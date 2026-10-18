"""make handoff-install (tools/handoff-install.sh): checksums, a diff, a confirmation, then cleanup."""

import hashlib
import subprocess

from support import REPO, Sandbox

SCRIPT = REPO / "tools/handoff-install.sh"


class HandoffInstallTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.repo = self.root / "repo"
        (self.repo / "config/claude").mkdir(parents=True)
        (self.repo / "config/claude/hook.sh").write_text("old\n")
        self.group = self.repo / ".handoff/claude"
        self.group.mkdir(parents=True)
        (self.group / "hook.sh").write_text("new\n")
        self.manifest("hook.sh\tconfig/claude/hook.sh\techo ran >> after.log\n")
        self.sums()

    def manifest(self, text: str):
        (self.group / "MANIFEST").write_text(text)

    def sums(self):
        digest = hashlib.sha256((self.group / "hook.sh").read_bytes()).hexdigest()
        (self.group / "SHA256SUMS").write_text(f"{digest}  hook.sh\n")

    def install(self, answer: str = "y\n") -> subprocess.CompletedProcess:
        return subprocess.run(["bash", str(SCRIPT)], input=answer, capture_output=True, text=True,
                              env={**self.env, "HANDOFF_REPO": str(self.repo)})

    def test_installs_on_yes_runs_the_follow_up_once_and_cleans_up(self):
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("+new", result.stdout)
        self.assertIn("then runs: echo ran >> after.log", result.stdout)
        self.assertEqual((self.repo / "config/claude/hook.sh").read_text(), "new\n")
        self.assertEqual((self.repo / "after.log").read_text(), "ran\n")
        self.assertFalse(self.group.exists())

    def test_no_leaves_everything_as_it_was(self):
        self.install("n\n")
        self.assertEqual((self.repo / "config/claude/hook.sh").read_text(), "old\n")
        self.assertTrue(self.group.exists())

    def test_a_checksum_mismatch_installs_nothing(self):
        (self.group / "hook.sh").write_text("tampered\n")
        result = self.install()
        self.assertEqual(result.returncode, 1)
        self.assertIn("checksum mismatch", result.stdout)
        self.assertEqual((self.repo / "config/claude/hook.sh").read_text(), "old\n")

    def test_a_destination_outside_the_repository_is_refused(self):
        self.manifest("hook.sh\t../../.bashrc\n")
        result = self.install()
        self.assertEqual(result.returncode, 1)
        self.assertIn("refusing", result.stdout)

    def test_an_installed_group_is_just_removed(self):
        (self.repo / "config/claude/hook.sh").write_text("new\n")
        result = self.install("")
        self.assertIn("already installed", result.stdout)
        self.assertFalse(self.group.exists())
