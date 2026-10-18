"""Shared pieces for the Claude session tests: a scratch home and runtime dir, and fake commands
on PATH that log what they were called with."""

import os
from pathlib import Path
import stat
import tempfile
import time
import unittest

REPO = Path(__file__).resolve().parents[2]
BIN = REPO / "config/bin/.local/bin"
OSD = REPO / "config/osd/.config/osd"
WAYBAR_SCRIPTS = REPO / "config/waybar/.config/waybar/scripts"
# SESSION_STATE_HOOK points the hook tests at another copy, such as the one in .handoff/claude
# before it is installed (config/claude is read-only for Claude inside the sandbox).
HOOK = Path(os.environ.get("SESSION_STATE_HOOK", REPO / "config/claude/.claude/hooks/session-state.sh"))
SESSION_A = "aaaaaaaa-0000-4000-8000-000000000000"
SESSION_B = "bbbbbbbb-0000-4000-8000-000000000000"


class Sandbox(unittest.TestCase):
    """A temporary HOME, XDG_RUNTIME_DIR and XDG_STATE_HOME, plus a shims directory first on PATH.
    Only the scripts under test see this environment; the host stays untouched."""

    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="claude-sessions-test-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.home = self.root / "home"
        self.runtime = self.root / "runtime"
        self.state = self.root / "state"
        self.shims = self.root / "shims"
        self.log = self.root / "calls.log"
        for path in (self.home, self.runtime, self.state, self.shims):
            path.mkdir()
        self.log.touch()
        # The scripts find their siblings through $HOME; link the real ones in.
        scripts = self.home / ".config/waybar/scripts"
        scripts.mkdir(parents=True)
        for script in WAYBAR_SCRIPTS.iterdir():
            (scripts / script.name).symlink_to(script)
        (self.home / ".config/osd").mkdir()
        for name in ("osd.py", "claude_registry.py"):
            (self.home / ".config/osd" / name).symlink_to(OSD / name)
        (self.home / ".local/bin").mkdir(parents=True)
        for name in ("claude-sessions", "claude-sessions-sidebar", "osd-show"):
            (self.home / ".local/bin" / name).symlink_to(BIN / name)
        self.env = {
            **os.environ,
            "HOME": str(self.home),
            "XDG_RUNTIME_DIR": str(self.runtime),
            "XDG_STATE_HOME": str(self.state),
            "PATH": f"{self.shims}:{self.home / '.local/bin'}:{os.environ['PATH']}",
            "SHIM_LOG": str(self.log),
            "CLAUDE_STATUS_URL": "",  # no network: status.claude.com is not fetched
        }

    def shim(self, name: str, body: str):
        """A fake command. `body` is bash; $SHIM_LOG receives one line per call by convention."""
        path = self.shims / name
        path.write_text("#!/usr/bin/env bash\n" + 'echo "' + name + ' $*" >> "$SHIM_LOG"\n' + body + "\n")
        path.chmod(path.stat().st_mode | stat.S_IXUSR)

    def calls(self, name: str) -> list[str]:
        return [line for line in self.log.read_text().splitlines() if line.startswith(name + " ")]

    def wait_for_calls(self, name: str, count: int, timeout: float = 3.0) -> list[str]:
        """For commands the code runs in the background: their log lines arrive a little later."""
        deadline = time.time() + timeout
        while time.time() < deadline:
            calls = self.calls(name)
            if len(calls) >= count:
                return calls
            time.sleep(0.05)
        return self.calls(name)

    def reset_log(self):
        self.log.write_text("")
