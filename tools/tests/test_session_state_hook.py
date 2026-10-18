"""The Claude Code hook (config/claude/.claude/hooks/session-state.sh): one FIFO line per event."""

import json
import os
import shutil
import subprocess
import time
import unittest

from support import HOOK, SESSION_A, Sandbox


class HookTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.fifo = self.runtime / "osd.fifo"
        os.mkfifo(self.fifo)
        # Collect what arrives, and hold the FIFO open for writing like the daemon does so the
        # reader sees no EOF between hook runs. The reader must open first: opening a FIFO for
        # writing blocks until someone reads.
        self.reader = subprocess.Popen(["cat", str(self.fifo)], stdout=subprocess.PIPE, text=True)
        self.addCleanup(self.reader.kill)
        self.holder = subprocess.Popen(["sleep", "30"], stdout=open(self.fifo, "w"))
        self.addCleanup(self.holder.kill)
        self.shim("notify-send", 'echo 42')
        self.shim("gdbus", "")

    def run_hook(self, payload: dict, **env) -> subprocess.CompletedProcess:
        return subprocess.run(["bash", str(HOOK)], input=json.dumps(payload), capture_output=True, text=True,
                              env={**self.env, "KITTY_PID": "4242", "TMUX_PANE": "%3", **env})

    def received(self) -> list[dict]:
        time.sleep(0.3)
        self.holder.kill()
        output = self.reader.communicate(timeout=5)[0]
        return [json.loads(line.partition(" ")[2]) for line in output.splitlines() if line.startswith("session ")]

    def test_events_map_to_states(self):
        cases = [
            ({"hook_event_name": "SessionStart", "source": "startup"}, "waiting", None),
            ({"hook_event_name": "UserPromptSubmit", "prompt": "add the chip\nmore"}, "working", None),
            ({"hook_event_name": "PostToolUse", "tool_name": "Bash"}, "working", None),
            ({"hook_event_name": "Notification", "message": "perm", "notification_type": "permission_prompt"}, "needs-you", True),
            ({"hook_event_name": "Notification", "message": "idle", "notification_type": "idle_prompt"}, "waiting", True),
            ({"hook_event_name": "Notification", "message": "done", "notification_type": "push_notification"}, "waiting", None),
            ({"hook_event_name": "Stop"}, "waiting", None),
            ({"hook_event_name": "SessionEnd", "reason": "exit"}, "gone", None),
        ]
        for payload, _, _ in cases:
            self.run_hook({"session_id": SESSION_A, "cwd": "/x/proj", **payload})
        lines = self.received()
        self.assertEqual([line["state"] for line in lines], [state for _, state, _ in cases])
        self.assertEqual([line.get("sticky") for line in lines], [sticky for _, _, sticky in cases])
        self.assertEqual(lines[1]["task"], "add the chip")
        self.assertEqual(lines[0]["tmux_pane"], "%3")

    def test_informational_notification_carries_no_state(self):
        self.run_hook({"hook_event_name": "Notification", "session_id": SESSION_A, "cwd": "/x/proj", "message": "ok", "notification_type": "auth_success"})
        line = self.received()[0]
        self.assertNotIn("state", line)
        self.assertEqual(line["message"], "ok")

    def test_unknown_event_fails_loudly(self):
        result = self.run_hook({"hook_event_name": "PreCompact", "session_id": SESSION_A, "cwd": "/x"})
        self.assertEqual(result.returncode, 1)
        self.assertIn("unexpected hook event", result.stderr)

    def test_nested_claude_reports_nothing(self):
        # A stand-in process named "claude" that a tool started (CLAUDECODE=1 in its environment),
        # between the hook and its real ancestors.
        fake_claude = self.root / "claude"
        shutil.copy(shutil.which("bash"), fake_claude)
        subprocess.run([str(fake_claude), "-c", f"bash {HOOK}; :"], input=json.dumps(
            {"hook_event_name": "Stop", "session_id": SESSION_A, "cwd": "/x/.issues-tmp"}), text=True,
            env={**self.env, "CLAUDECODE": "1"})
        self.assertEqual(self.received(), [])

    def test_a_session_started_from_a_shell_reports_even_under_the_daemon(self):
        # Since Claude Code 2.1.300 a session sits under processes named claude (launcher, daemon)
        # that carry no CLAUDECODE mark: two of them must not look like nesting.
        fake_claude = self.root / "claude"
        shutil.copy(shutil.which("bash"), fake_claude)
        # The daemon scrubs the session's environment: the pane and the kitty window come from
        # the launcher above (the outer fake claude here).
        env = {key: value for key, value in self.env.items() if key != "CLAUDECODE"}
        env.update({"TMUX_PANE": "%9", "KITTY_PID": "4242"})
        subprocess.run([str(fake_claude), "-c", f"env -u TMUX_PANE -u KITTY_PID {fake_claude} -c 'bash {HOOK}'; :"], input=json.dumps(
            {"hook_event_name": "Stop", "session_id": SESSION_A, "cwd": "/x/proj"}), text=True, env=env)
        lines = self.received()
        self.assertEqual([line["state"] for line in lines], ["waiting"])
        self.assertEqual((lines[0]["tmux_pane"], lines[0]["kitty_pid"]), ("%9", "4242"))

    def test_each_event_leaves_a_line_in_the_hook_log(self):
        self.run_hook({"hook_event_name": "UserPromptSubmit", "session_id": SESSION_A, "cwd": "/x/proj",
                       "prompt": "the secret plan", "source": "user"})
        self.received()
        line = (self.home / ".cache/claude/hook.log").read_text().splitlines()[-1]
        self.assertRegex(line, r" UserPromptSubmit aaaaaaaa working: sent to the osd daemon$")
        self.assertNotIn("secret", line)

    def test_fallback_toast_when_the_fifo_is_unreachable(self):
        os.remove(self.fifo)
        self.run_hook({"hook_event_name": "Stop", "session_id": SESSION_A, "cwd": "/x/proj"})
        self.assertEqual(len(self.calls("notify-send")), 1)
        self.assertEqual((self.home / ".cache/claude" / f"toast-{SESSION_A}.id").read_text().strip(), "42")
        self.run_hook({"hook_event_name": "UserPromptSubmit", "session_id": SESSION_A, "cwd": "/x/proj", "prompt": "x"})
        self.assertEqual(len(self.calls("gdbus")), 1)
        self.assertFalse((self.home / ".cache/claude" / f"toast-{SESSION_A}.id").exists())


if __name__ == "__main__":
    unittest.main()
