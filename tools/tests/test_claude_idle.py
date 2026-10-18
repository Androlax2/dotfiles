"""The Claude workspace's idle prompt (hypr/scripts/claude-idle.sh) around its screen."""

import subprocess
import time
import unittest

from support import REPO, Sandbox

SCRIPT = REPO / "config/hyprland/.config/hypr/scripts/claude-idle.sh"
HYPR_SCRIPTS = REPO / "config/hyprland/.config/hypr/scripts"


class IdlePromptTests(Sandbox):
    def setUp(self):
        super().setUp()
        # The script finds its neighbours through $HOME (the sandbox already links the bar's); the
        # screen itself (GTK) is a shim whose exit code plays the answer or the timeout.
        scripts = self.home / ".config/hypr/scripts"
        scripts.mkdir(parents=True)
        for name in ("is-gaming.sh",):
            (scripts / name).symlink_to(HYPR_SCRIPTS / name)
        self.prompt = scripts / "claude-idle-prompt.py"
        self.shim("loginctl", "")
        self.shim("pidof", "exit 1")

    def screen(self, body: str):
        self.prompt.write_text("#!/bin/sh\n" + body + "\n")
        self.prompt.chmod(0o755)

    def on_workspace(self, workspace: int):
        self.shim("hyprctl", f'[[ $* == *activeworkspace* ]] && echo \'{{"id": {workspace}}}\'; '
                             f'[[ $* == *activewindow* ]] && echo \'{{"class": "kitty", "fullscreen": 0}}\'; exit 0')

    def run_script(self, action: str, **env) -> subprocess.CompletedProcess:
        return subprocess.run(["bash", str(SCRIPT), action], capture_output=True, text=True,
                              env={**self.env, "CLAUDE_IDLE_GRACE_S": "1", "CLAUDE_IDLE_SCREEN_OFF_AFTER_S": "0", **env})

    def test_elsewhere_the_prompt_does_nothing(self):
        self.on_workspace(2)
        self.screen('echo "shown $1" > "$HOME/shown"; exit 1')
        self.run_script("prompt")
        self.assertFalse((self.home / "shown").exists())
        self.assertEqual(self.calls("loginctl"), [])

    def test_on_the_claude_workspace_an_unanswered_screen_locks(self):
        self.on_workspace(6)
        self.screen('echo "shown $1" > "$HOME/shown"; exit 1')
        self.run_script("prompt")
        self.assertEqual((self.home / "shown").read_text().strip(), "shown 1")
        self.assertEqual(self.calls("loginctl"), ["loginctl lock-session"])

    def test_an_answered_screen_keeps_the_session_unlocked(self):
        self.on_workspace(6)
        self.screen("exit 0")
        self.assertEqual(self.run_script("prompt").returncode, 0)
        self.assertEqual(self.calls("loginctl"), [])

    def test_resumed_activity_closes_the_screen_and_forgets_the_lock(self):
        self.on_workspace(6)
        self.screen("trap 'exit 0' TERM; sleep 10 & wait $!; exit 1")
        pending = subprocess.Popen(["bash", str(SCRIPT), "prompt"], env={**self.env, "CLAUDE_IDLE_GRACE_S": "10"})
        self.addCleanup(pending.kill)
        time.sleep(0.5)
        self.run_script("cancel")
        self.assertEqual(pending.wait(timeout=5), 0)
        self.assertEqual(self.calls("loginctl"), [])
        self.assertFalse((self.runtime / "claude-idle.pid").exists())


if __name__ == "__main__":
    unittest.main()
