import contextlib
import configparser
import fcntl
import io
import json
import os
from pathlib import Path
import runpy
import shlex
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock


REPOSITORY = Path(__file__).resolve().parents[2]
SCRIPT = REPOSITORY / "config/bin/.local/bin/cache-maintenance-report"


class TurboReportTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.home = Path(self.temporary.name) / "home"
        self.home.mkdir()
        self.runtime = Path(self.temporary.name) / "runtime"
        self.runtime.mkdir(mode=0o700)
        self.cache = self.home / "IdeaProjects/hartprint/.turbo/cache"
        self.environment = dict(os.environ, HOME=str(self.home),
                                XDG_RUNTIME_DIR=str(self.runtime))

    def run_report(self, *arguments):
        return subprocess.run([sys.executable, str(SCRIPT), *arguments],
                              env=self.environment, capture_output=True, text=True)

    def create_entry(self, name="entry.tar.zst", access_days=40, modified_days=40):
        self.cache.mkdir(parents=True, exist_ok=True)
        entry = self.cache / name
        entry.write_bytes(b"cache fixture" * 1024)
        now = time.time()
        os.utime(entry, (now - access_days * 86400, now - modified_days * 86400))
        return entry

    def test_missing_cache_reports_zero(self):
        result = self.run_report()
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual(report["observed_files"], 0)
        self.assertEqual(report["old_files"], 0)

    def test_old_files_are_provisional_not_safe_deletion_candidates(self):
        entry = self.create_entry()
        allocated = entry.stat().st_blocks * 512
        result = self.run_report()
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual(report["mode"], "report-only")
        self.assertFalse(report["verified_safe_for_deletion"])
        self.assertEqual(report["old_files"], 1)
        self.assertEqual(report["estimated_old_allocated_bytes"], allocated)
        self.assertEqual(report["old_entries"], ["entry.tar.zst"])

    def test_recent_access_or_modification_excludes_file(self):
        for access_days, modified_days in [(1, 40), (40, 1), (1, 1)]:
            with self.subTest(access_days=access_days, modified_days=modified_days):
                self.create_entry(access_days=access_days, modified_days=modified_days)
                result = self.run_report()
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(json.loads(result.stdout)["old_files"], 0)

    def test_large_inventory_bounds_journal_output_without_hiding_totals(self):
        for index in range(25):
            self.create_entry(name=f"{index:02d}.tar.zst")
        result = self.run_report()
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual(report["old_files"], 25)
        self.assertEqual(len(report["old_entries"]), 20)
        self.assertEqual(report["old_entries_omitted"], 5)

    def test_exact_cutoff_is_not_old(self):
        self.assertTrue(SCRIPT.exists(), "report implementation is missing")
        implementation = runpy.run_path(str(SCRIPT))
        now = 4000000
        cutoff = now - 30 * 86400
        for access, modification, expected in [
            (cutoff, cutoff - 1, False), (cutoff - 1, cutoff, False),
            (cutoff - 1, cutoff - 1, True),
        ]:
            with self.subTest(access=access, modification=modification):
                metadata = os.stat_result((0, 0, 0, 0, 0, 0, 0, access, modification, 0))
                self.assertEqual(implementation["is_old"](metadata, now), expected)

    def test_no_file_content_or_timestamps_are_changed(self):
        entry = self.create_entry()
        before = entry.stat()
        result = self.run_report()
        self.assertEqual(result.returncode, 0, result.stderr)
        after = entry.stat()
        self.assertEqual((before.st_atime_ns, before.st_mtime_ns, before.st_size),
                         (after.st_atime_ns, after.st_mtime_ns, after.st_size))
        self.assertEqual(list(self.cache.iterdir()), [entry])
        self.assertEqual(entry.read_bytes(), b"cache fixture" * 1024)

    def test_symlink_ancestors_are_rejected(self):
        self.create_entry()
        for ancestor in [self.home, self.home / "IdeaProjects", self.cache.parent, self.cache]:
            with self.subTest(ancestor=ancestor):
                moved = ancestor.with_name(ancestor.name + "-real")
                ancestor.rename(moved)
                try:
                    ancestor.symlink_to(moved, target_is_directory=True)
                    result = self.run_report()
                    self.assertNotEqual(result.returncode, 0)
                    self.assertEqual(result.stdout, "")
                finally:
                    ancestor.unlink()
                    moved.rename(ancestor)

    def test_symlink_entry_is_rejected_without_following(self):
        self.create_entry()
        (self.cache / "link").symlink_to("/nonexistent-cache-target")
        result = self.run_report()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("regular file", result.stderr)
        self.assertEqual(result.stdout, "")

    def test_nested_directory_is_rejected(self):
        self.create_entry()
        (self.cache / "unexpected").mkdir()
        result = self.run_report()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("regular file", result.stderr)

    def test_no_delete_or_other_arguments_are_accepted(self):
        for argument in ["--apply", "--delete", "/other/cache"]:
            with self.subTest(argument=argument):
                result = self.run_report(argument)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("arguments", result.stderr)

    @unittest.skipIf(os.geteuid() == 0, "root bypasses ordinary filesystem permissions")
    def test_unreadable_cache_fails_visibly(self):
        self.create_entry()
        self.cache.chmod(0)
        try:
            result = self.run_report()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Permission", result.stderr)
        finally:
            self.cache.chmod(0o700)

    def test_lock_contention_fails_visibly(self):
        with (self.runtime / "cache-maintenance-report.lock").open("w") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            result = self.run_report()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("already running", result.stderr)

    def test_missing_runtime_directory_fails(self):
        self.runtime.rmdir()
        result = self.run_report()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")

    def test_disappearing_file_fails_without_partial_report(self):
        self.assertTrue(SCRIPT.exists(), "report implementation is missing")
        self.create_entry()
        implementation = runpy.run_path(str(SCRIPT))
        original_stat = os.stat

        def racing_stat(path, *arguments, **options):
            if options.get("dir_fd") is not None and str(path) == "entry.tar.zst":
                raise FileNotFoundError("cache entry disappeared")
            return original_stat(path, *arguments, **options)

        output, errors = io.StringIO(), io.StringIO()
        with mock.patch.dict(os.environ, self.environment), mock.patch.object(sys, "argv", [str(SCRIPT)]):
            with mock.patch("os.stat", side_effect=racing_stat):
                with contextlib.redirect_stdout(output), contextlib.redirect_stderr(errors):
                    status = implementation["main"]()
        self.assertNotEqual(status, 0)
        self.assertEqual(output.getvalue(), "")
        self.assertIn("disappeared", errors.getvalue())


class MaintenanceUnitTests(unittest.TestCase):
    def read_unit(self, scope, name):
        folder = (REPOSITORY / "config/systemd/.config/systemd/user" if scope == "user"
                  else REPOSITORY / "system/etc/systemd/system")
        path = folder / name
        self.assertTrue(path.exists(), f"unit is missing: {path}")
        unit = configparser.ConfigParser(interpolation=None)
        unit.read(path)
        return unit

    def test_user_service_invokes_report_without_deletion_arguments(self):
        service = self.read_unit("user", "cache-maintenance-report.service")["Service"]
        self.assertEqual(shlex.split(service["ExecStart"]),
                         ["/usr/bin/python3", "%h/.local/bin/cache-maintenance-report"])
        self.assertEqual(service["ProtectHome"], "read-only")
        self.assertEqual(service["ReadWritePaths"], "%t")

    def test_system_service_retains_two_versions_and_recent_packages_in_dry_run(self):
        service = self.read_unit("system", "package-cache-report.service")["Service"]
        self.assertEqual(shlex.split(service["ExecStart"]), [
            "/usr/bin/paccache", "--dryrun", "--verbose", "--cachedir", "/var/cache/pacman/pkg", "--keep", "2",
            "--min-atime", "30 days ago", "--min-mtime", "30 days ago",
        ])
        self.assertEqual(service["ProtectHome"], "true")
        self.assertEqual(service["PrivateNetwork"], "true")
        self.assertNotIn("ReadWritePaths", service)

    def test_both_services_are_low_priority_read_only_oneshots(self):
        for scope, name in [("user", "cache-maintenance-report"),
                            ("system", "package-cache-report")]:
            with self.subTest(scope=scope):
                service = self.read_unit(scope, name + ".service")["Service"]
                self.assertEqual(service["Type"], "oneshot")
                self.assertEqual(service["Nice"], "19")
                self.assertEqual(service["IOSchedulingClass"], "idle")
                self.assertEqual(service["ProtectSystem"], "strict")
                self.assertEqual(service["NoNewPrivileges"], "true")
                self.assertEqual(service["PrivateTmp"], "true")
                self.assertEqual(service["StandardOutput"], "journal")

    def test_timers_run_matching_services_daily_and_catch_up(self):
        for scope, name in [("user", "cache-maintenance-report"),
                            ("system", "package-cache-report")]:
            with self.subTest(scope=scope):
                unit = self.read_unit(scope, name + ".timer")
                self.assertEqual(unit["Timer"]["OnCalendar"], "daily")
                self.assertEqual(unit["Timer"]["Persistent"], "true")
                self.assertEqual(unit["Timer"]["RandomizedDelaySec"], "1h")
                self.assertEqual(unit["Timer"].get("Unit", name + ".service"), name + ".service")
                self.assertEqual(unit["Install"]["WantedBy"], "timers.target")
                self.read_unit(scope, name + ".service")


if __name__ == "__main__":
    unittest.main()
