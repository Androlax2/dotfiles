import configparser
import contextlib
import fcntl
import hashlib
import io
import json
import os
from pathlib import Path
import shlex
import shutil
import runpy
import subprocess
import struct
import sys
import tempfile
import time
import unittest
from unittest import mock
import zlib


REPOSITORY = Path(__file__).resolve().parents[2]
SYSTEM = REPOSITORY / "system/etc/systemd"
REVIEW = REPOSITORY / "config/bin/.local/bin/disk-space-review"


def read_policy(relative):
    policy = configparser.ConfigParser(interpolation=None, strict=False)
    path = SYSTEM / relative
    if not path.is_file():
        raise AssertionError(f"missing retention policy: {path}")
    policy.read(path)
    return policy


class NativeRetentionTests(unittest.TestCase):
    def package_command(self):
        service = read_policy("system/paccache.service.d/retention.conf")["Service"]
        return shlex.split(service["ExecStart"])

    def test_journal_retention_uses_native_age_and_rotation(self):
        journal = read_policy("journald.conf.d/60-disk-maintenance.conf")["Journal"]
        self.assertEqual(journal["MaxRetentionSec"], "30day")
        self.assertEqual(journal["MaxFileSec"], "1day")
        self.assertNotIn("SystemMaxUse", journal)

    def test_package_cleanup_targets_only_approved_cache(self):
        self.assertEqual(self.package_command(), [
            "/usr/bin/paccache", "--remove", "--cachedir", "/var/cache/pacman/pkg",
            "--keep", "2", "--min-atime", "30 days ago", "--min-mtime", "30 days ago",
        ])
        service = read_policy("system/paccache.service.d/retention.conf")["Service"]
        self.assertEqual(service["ProtectSystem"], "strict")
        self.assertEqual(service["ReadWritePaths"], "/var/cache/pacman/pkg")

    def test_vendor_cleanup_commands_and_schedules_are_reset_before_overriding(self):
        service = SYSTEM / "system/paccache.service.d/retention.conf"
        timer = SYSTEM / "system/paccache.timer.d/schedule.conf"
        for path, key in [(service, "ExecStart"), (timer, "OnCalendar")]:
            with self.subTest(key=key):
                values = [line.split("=", 1)[1] for line in path.read_text().splitlines() if line.startswith(key + "=")]
                self.assertEqual(len(values), 2)
                self.assertEqual(values[0], "")
                self.assertNotEqual(values[1], "")

    def test_native_package_timer_is_daily_and_persistent(self):
        timer = read_policy("system/paccache.timer.d/schedule.conf")["Timer"]
        self.assertEqual(timer["OnCalendar"], "daily")
        self.assertEqual(timer["Persistent"], "true")
        self.assertEqual(timer["RandomizedDelaySec"], "1h")

    def run_fixture(self, recent_access=None, recent_modification=None, dry_run=False):
        with tempfile.TemporaryDirectory() as temporary:
            cache = Path(temporary)
            old = time.time() - 45 * 86400
            for version in range(1, 6):
                archive = cache / f"fixture-{version}.0-1-any.pkg.tar.zst"
                archive.write_bytes(b"disposable cache fixture")
                os.utime(archive, (time.time() if version == recent_access else old,
                                   time.time() if version == recent_modification else old))
            command = self.package_command()
            command[command.index("--cachedir") + 1] = str(cache)
            if dry_run:
                command[command.index("--remove")] = "--dryrun"
            result = subprocess.run(command, capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            return sorted(path.name for path in cache.iterdir())

    def test_old_excess_archives_are_removed_but_two_versions_survive(self):
        self.assertEqual(self.run_fixture(), ["fixture-4.0-1-any.pkg.tar.zst", "fixture-5.0-1-any.pkg.tar.zst"])

    def test_recent_access_alone_preserves_an_old_version(self):
        self.assertEqual(self.run_fixture(recent_access=1), [
            "fixture-1.0-1-any.pkg.tar.zst", "fixture-4.0-1-any.pkg.tar.zst", "fixture-5.0-1-any.pkg.tar.zst",
        ])

    def test_recent_modification_alone_preserves_an_old_version(self):
        self.assertEqual(self.run_fixture(recent_modification=2), [
            "fixture-2.0-1-any.pkg.tar.zst", "fixture-4.0-1-any.pkg.tar.zst", "fixture-5.0-1-any.pkg.tar.zst",
        ])

    def test_dry_run_preserves_every_archive(self):
        self.assertEqual(len(self.run_fixture(dry_run=True)), 5)


class MonthlyReviewTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.home = Path(self.temporary.name) / "home"
        self.runtime = Path(self.temporary.name) / "runtime"
        self.state = Path(self.temporary.name) / "state"
        self.runtime.mkdir(mode=0o700)
        self.state.mkdir(mode=0o700)
        for root in ["Games", ".local/share/Steam/steamapps/common", "IdeaProjects/.parallel"]:
            (self.home / root).mkdir(parents=True, exist_ok=True)
        self.environment = dict(os.environ, HOME=str(self.home),
                                XDG_RUNTIME_DIR=str(self.runtime), XDG_STATE_HOME=str(self.state))
        self.report = self.state / "disk-maintenance/latest.json"

    def run_review(self, *arguments):
        self.assertTrue(REVIEW.exists(), "monthly review implementation is missing")
        return subprocess.run([sys.executable, str(REVIEW), *arguments], env=self.environment,
                              capture_output=True, text=True, timeout=30)

    def game(self, name="game", blocks=1):
        game = self.home / "Games" / name
        game.mkdir()
        (game / "asset").write_bytes(b"a" * (4096 * blocks))
        return game

    def repository(self, name="project", upstream=False, tracked=False):
        project = self.home / "IdeaProjects" / name
        metadata = project / ".git"
        (metadata / "objects").mkdir(parents=True)
        (metadata / "refs/heads").mkdir(parents=True)
        (metadata / "info").mkdir()

        def object_id(kind, body):
            content = f"{kind} {len(body)}\0".encode() + body
            identifier = hashlib.sha1(content).hexdigest()
            folder = metadata / "objects" / identifier[:2]
            folder.mkdir(exist_ok=True)
            (folder / identifier[2:]).write_bytes(zlib.compress(content))
            return identifier

        tree_body = b""
        if tracked:
            tracked_file = project / "tracked.txt"
            tracked_file.write_bytes(b"original\n")
            blob = object_id("blob", b"original\n")
            tree_body = b"100644 tracked.txt\0" + bytes.fromhex(blob)
            metadata_stat = tracked_file.stat()
            fields = [int(metadata_stat.st_ctime), 0, int(metadata_stat.st_mtime), 0,
                      metadata_stat.st_dev, metadata_stat.st_ino, 0o100644,
                      metadata_stat.st_uid, metadata_stat.st_gid, metadata_stat.st_size]
            entry = struct.pack("!10I20sH", *(field & 0xffffffff for field in fields), bytes.fromhex(blob), 11) + b"tracked.txt\0"
            entry += b"\0" * ((-len(entry)) % 8)
            index = struct.pack("!4sII", b"DIRC", 2, 1) + entry
            (metadata / "index").write_bytes(index + hashlib.sha1(index).digest())
        tree = object_id("tree", tree_body)
        initial = object_id("commit", f"tree {tree}\nauthor Fixture <fixture@example.invalid> 1 +0000\ncommitter Fixture <fixture@example.invalid> 1 +0000\n\ninitial\n".encode())
        head = initial
        config = "[core]\nrepositoryformatversion = 0\nbare = false\n"
        if upstream:
            head = object_id("commit", f"tree {tree}\nparent {initial}\nauthor Fixture <fixture@example.invalid> 2 +0000\ncommitter Fixture <fixture@example.invalid> 2 +0000\n\nahead\n".encode())
            remote = metadata / "refs/remotes/origin"
            remote.mkdir(parents=True)
            (remote / "main").write_text(initial + "\n")
            config += '[remote "origin"]\nurl = https://example.invalid/never-fetch\nfetch = +refs/heads/*:refs/remotes/origin/*\n[branch "main"]\nremote = origin\nmerge = refs/heads/main\n'
        (metadata / "HEAD").write_text("ref: refs/heads/main\n")
        (metadata / "refs/heads/main").write_text(head + "\n")
        (metadata / "config").write_text(config)
        return project

    def test_ranks_games_and_preserves_content_and_timestamps(self):
        small = self.game("small", 1)
        large = self.game("large", 8)
        before = (large / "asset").stat()
        result = self.run_review()
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(self.report.read_text())
        self.assertEqual([entry["path"] for entry in report["games"]], [str(large), str(small)])
        self.assertTrue(report["advisory_only"])
        self.assertTrue(report["complete"])
        self.assertEqual((large / "asset").stat().st_mtime_ns, before.st_mtime_ns)
        self.assertEqual((large / "asset").read_bytes(), b"a" * (4096 * 8))
        self.assertEqual(self.report.stat().st_mode & 0o777, 0o600)

    def test_parallel_checkout_is_not_double_counted_as_a_direct_project(self):
        checkout = self.home / "IdeaProjects/.parallel/dotfiles/session"
        checkout.mkdir(parents=True)
        result = self.run_review()
        self.assertEqual(result.returncode, 0, result.stderr)
        paths = [entry["path"] for entry in json.loads(self.report.read_text())["checkouts"]]
        self.assertEqual(paths, [str(checkout)])

    def test_ranked_reports_are_bounded_without_hiding_counts(self):
        for number in range(25):
            self.game(f"game-{number}")
        result = self.run_review()
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(self.report.read_text())
        self.assertEqual((len(report["games"]), report["measured_games"], report["omitted_games"]), (20, 25, 5))

    def test_missing_roots_are_explicitly_unavailable_not_measured_as_zero(self):
        (self.home / "Games").rmdir()
        result = self.run_review()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(str(self.home / "Games"), json.loads(self.report.read_text())["unavailable_roots"])

    def test_symlink_is_not_followed_and_marks_report_incomplete(self):
        game = self.game()
        (game / "outside").symlink_to(self.state, target_is_directory=True)
        result = self.run_review()
        self.assertNotEqual(result.returncode, 0)
        report = json.loads(self.report.read_text())
        self.assertFalse(report["complete"])
        self.assertTrue(report["errors"])
        self.assertEqual(len(report["games"]), 1)
        self.assertFalse(report["games"][0]["complete"])
        self.assertGreater(report["games"][0]["allocated_bytes"], 0)

    def test_root_symlink_is_not_followed(self):
        root = self.home / "Games"
        root.rmdir()
        root.symlink_to(self.state, target_is_directory=True)
        result = self.run_review()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(json.loads(self.report.read_text())["complete"])

    def test_hardlinks_are_counted_once_within_an_entry(self):
        game = self.game()
        os.link(game / "asset", game / "copy")
        expected = game.stat().st_blocks * 512 + (game / "asset").stat().st_blocks * 512
        result = self.run_review()
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(self.report.read_text())
        self.assertEqual(report["games"][0]["allocated_bytes"], expected)
        self.assertIn("non-additive", report["warning"])

    def test_untracked_files_and_local_ahead_commits_are_reported_without_git_writes(self):
        project = self.repository(upstream=True)
        (project / "unfinished.txt").write_text("unfinished")
        before = {str(path): path.read_bytes() for path in (project / ".git").rglob("*") if path.is_file()}
        result = self.run_review()
        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads(self.report.read_text())["checkouts"][0]["git"]
        self.assertTrue(summary["dirty"])
        self.assertTrue(summary["has_upstream"])
        self.assertEqual(summary["ahead"], 1)
        after = {str(path): path.read_bytes() for path in (project / ".git").rglob("*") if path.is_file()}
        self.assertEqual(after, before)

    def test_ignored_files_do_not_imply_project_is_safe_to_delete(self):
        project = self.repository()
        (project / ".git/info/exclude").write_text("secret.env\n")
        (project / "secret.env").write_text("personal fixture")
        result = self.run_review()
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(self.report.read_text())
        self.assertFalse(report["checkouts"][0]["git"]["has_upstream"])
        self.assertIn("ignored", report["warning"])
        self.assertNotIn("safe_to_delete", report["checkouts"][0]["git"])

    def test_gitfile_outside_roots_is_unknown_not_followed(self):
        project = self.home / "IdeaProjects/linked"
        project.mkdir()
        (project / ".git").write_text(f"gitdir: {self.state}\n")
        result = self.run_review()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(self.report.read_text())["checkouts"][0]["git"]["status"], "unknown")

    def test_worktree_filters_are_never_executed(self):
        project = self.repository(tracked=True)
        marker = self.state / "filter-executed"
        with (project / ".git/config").open("a") as config:
            config.write(f'[filter "hostile"]\nclean = "touch {marker}; cat"\nrequired = true\n')
        (project / ".git/info/attributes").write_text("tracked.txt filter=hostile\n")
        (project / "tracked.txt").write_text("modified worktree\n")
        result = self.run_review()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(marker.exists())
        summary = json.loads(self.report.read_text())["checkouts"][0]["git"]
        self.assertIsNone(summary["dirty"])
        self.assertEqual(summary["status"], "unknown")

    def test_delete_arguments_are_rejected_without_writing_a_report(self):
        for argument in ["--delete", "--apply", "/somewhere"]:
            with self.subTest(argument=argument):
                result = self.run_review(argument)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(self.report.exists())

    def test_concurrent_report_keeps_previous_report(self):
        self.report.parent.mkdir()
        self.report.write_text("previous")
        with (self.runtime / "disk-space-review.lock").open("w") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            result = self.run_review()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("already running", result.stderr)
        self.assertEqual(self.report.read_text(), "previous")

    def test_symlink_report_is_rejected_without_changing_target(self):
        self.report.parent.mkdir()
        target = self.state / "personal"
        target.write_text("keep me")
        self.report.symlink_to(target)
        result = self.run_review()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(target.read_text(), "keep me")
        self.assertTrue(self.report.is_symlink())

    def test_control_characters_in_names_are_json_escaped(self):
        self.game("game\n\x1b[31m")
        result = self.run_review()
        self.assertEqual(result.returncode, 0, result.stderr)
        raw = self.report.read_text()
        self.assertNotIn("\x1b", raw)
        self.assertIn("\\u001b", raw)

    def test_monthly_unit_has_timeout_and_no_deletion_command(self):
        folder = REPOSITORY / "config/systemd/.config/systemd/user"
        policy = configparser.ConfigParser(interpolation=None)
        self.assertTrue((folder / "disk-space-review.service").is_file())
        policy.read(folder / "disk-space-review.service")
        self.assertEqual(shlex.split(policy["Service"]["ExecStart"]),
                         ["/usr/bin/python3", "%h/.local/bin/disk-space-review"])
        self.assertEqual(policy["Service"]["TimeoutStartSec"], "15min")
        policy.read(folder / "disk-space-review.timer")
        self.assertEqual(policy["Timer"]["OnCalendar"], "monthly")
        self.assertEqual(policy["Timer"]["Persistent"], "true")

    def invoke_with_patch(self, patcher):
        self.assertTrue(REVIEW.exists(), "monthly review implementation is missing")
        implementation = runpy.run_path(str(REVIEW))
        output, errors = io.StringIO(), io.StringIO()
        with mock.patch.dict(os.environ, self.environment), mock.patch.object(sys, "argv", [str(REVIEW)]):
            with patcher, contextlib.redirect_stdout(output), contextlib.redirect_stderr(errors):
                return implementation["main"](), errors.getvalue()

    def test_failed_git_read_is_unknown_not_clean(self):
        self.repository()
        failure = subprocess.TimeoutExpired("git", 10)
        status, _ = self.invoke_with_patch(mock.patch("subprocess.run", side_effect=failure))
        self.assertEqual(status, 0)
        summary = json.loads(self.report.read_text())["checkouts"][0]["git"]
        self.assertEqual(summary["status"], "unknown")
        self.assertIsNone(summary["dirty"])

    def test_git_status_one_is_an_error_for_inventory_not_a_clean_worktree(self):
        self.repository()
        original_run = subprocess.run

        def failed_inventory(command, *arguments, **options):
            if "ls-files" in command:
                return subprocess.CompletedProcess(command, 1)
            return original_run(command, *arguments, **options)

        status, _ = self.invoke_with_patch(mock.patch("subprocess.run", side_effect=failed_inventory))
        self.assertEqual(status, 0)
        summary = json.loads(self.report.read_text())["checkouts"][0]["git"]
        self.assertEqual(summary["status"], "unknown")
        self.assertIsNone(summary["dirty"])

    def test_cross_device_entry_is_not_measured(self):
        self.game()
        original_stat = os.stat

        def mounted_stat(path, *arguments, **options):
            result = original_stat(path, *arguments, **options)
            if path == "asset" and options.get("dir_fd") is not None:
                fields = list(result)
                fields[2] += 1
                return os.stat_result(fields)
            return result

        status, _ = self.invoke_with_patch(mock.patch("os.stat", side_effect=mounted_stat))
        self.assertNotEqual(status, 0)
        report = json.loads(self.report.read_text())
        self.assertFalse(report["complete"])
        self.assertEqual(len(report["games"]), 1)
        self.assertFalse(report["games"][0]["complete"])

    def test_interrupted_publication_preserves_previous_report(self):
        self.report.parent.mkdir()
        self.report.write_text("previous")
        with self.assertRaises(KeyboardInterrupt):
            self.invoke_with_patch(mock.patch("os.replace", side_effect=KeyboardInterrupt))
        self.assertEqual(self.report.read_text(), "previous")

    def test_symlink_state_directory_is_rejected(self):
        (self.state / "disk-maintenance").symlink_to(self.home, target_is_directory=True)
        result = self.run_review()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.home / "latest.json").exists())


class UnitValidationTests(unittest.TestCase):
    def test_native_package_dropins_validate_against_vendor_units_without_installation(self):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            for name in ["paccache.service", "paccache.timer"]:
                shutil.copyfile(Path("/usr/lib/systemd/system") / name, folder / name)
                shutil.copytree(SYSTEM / "system" / (name + ".d"), folder / (name + ".d"))
            environment = dict(os.environ, SYSTEMD_UNIT_PATH=str(folder) + ":/usr/lib/systemd/system")
            result = subprocess.run(["/usr/bin/systemd-analyze", "--man=no", "verify",
                                     str(folder / "paccache.service"), str(folder / "paccache.timer")],
                                    env=environment, capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
