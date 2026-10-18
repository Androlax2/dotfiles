"""claude-version-link: finishes Claude Code's self-updates that greywall keeps from completing."""

import os
import subprocess

from support import BIN, Sandbox


class VersionLinkTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.versions = self.root / "versions"
        self.versions.mkdir()
        self.link = self.root / "claude"

    def version(self, name: str, starts: bool = True):
        path = self.versions / name
        path.write_text("#!/bin/sh\nexit 0\n" if starts else "#!/bin/sh\nexit 1\n")
        path.chmod(0o755)

    def run_link(self) -> subprocess.CompletedProcess:
        return subprocess.run([str(BIN / "claude-version-link")], capture_output=True, text=True,
                              env={**self.env, "CLAUDE_VERSIONS_DIR": str(self.versions), "CLAUDE_LINK": str(self.link)})

    def target(self) -> str:
        return os.path.basename(os.readlink(self.link))

    def test_moves_the_link_to_the_newest_version(self):
        for name in ("2.1.300", "2.1.302", "2.1.99"):
            self.version(name)
        self.link.symlink_to(self.versions / "2.1.300")
        self.run_link()
        self.assertEqual(self.target(), "2.1.302")

    def test_skips_a_version_that_does_not_start(self):
        self.version("2.1.300")
        self.version("2.1.301")
        self.version("2.1.302", starts=False)
        self.version("2.1.303.partial")
        self.link.symlink_to(self.versions / "2.1.300")
        self.run_link()
        self.assertEqual(self.target(), "2.1.301")

    def test_never_moves_backwards_from_a_pinned_newer_version(self):
        self.version("2.1.300")
        self.version("2.1.305", starts=False)
        self.link.symlink_to(self.versions / "2.1.305")
        self.run_link()
        self.assertEqual(self.target(), "2.1.305")

    def test_creates_the_link_when_there_is_none(self):
        self.version("2.1.302")
        self.assertEqual(self.run_link().returncode, 0)
        self.assertEqual(self.target(), "2.1.302")
