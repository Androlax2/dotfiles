"""The bar's Claude pieces: the sessions chip, the usage reader and the bar visibility keeper."""

import json
import subprocess
import unittest

from support import WAYBAR_SCRIPTS, Sandbox


class SessionsChipTests(Sandbox):
    def chip(self, snapshot) -> dict:
        path = self.root / "snapshot.json"
        path.write_text(json.dumps(snapshot))
        result = subprocess.run([str(WAYBAR_SCRIPTS / "claude-sessions.sh")], capture_output=True, text=True,
                                env={**self.env, "CLAUDE_SESSIONS_SNAPSHOT": str(path)})
        return json.loads(result.stdout)

    def test_counts_by_state_with_the_most_urgent_class(self):
        chip = self.chip({"sessions": [{"state": "needs-you", "project": "a", "since": 1}, {"state": "working", "project": "b", "since": 1}]})
        self.assertIn("● 1", chip["text"])
        self.assertIn("! 1", chip["text"])
        self.assertEqual(chip["class"], "needs-you")

    def test_idle_glyph_with_no_session(self):
        chip = self.chip({"sessions": []})
        self.assertEqual(chip["class"], "idle")
        self.assertEqual(chip["text"], "\U000F06A9")

    def test_tooltip_escapes_markup(self):
        chip = self.chip({"sessions": [{"state": "working", "project": "a", "task": "fix <totals>", "since": 1}]})
        self.assertIn("&lt;totals&gt;", chip["tooltip"])


class UsageTests(Sandbox):
    def usage(self, window: str, fixture: dict) -> dict:
        path = self.root / "usage.json"
        path.write_text(json.dumps(fixture))
        result = subprocess.run([str(WAYBAR_SCRIPTS / "claude-usage.sh"), window], capture_output=True, text=True,
                                env={**self.env, "CLAUDE_USAGE_FIXTURE": str(path)})
        return json.loads(result.stdout)

    def test_each_window_carries_its_glyph_and_threshold_class(self):
        fixture = {"five_hour": {"utilization": 42.3, "resets_at": "2026-10-16T21:00:00Z"}, "seven_day": {"utilization": 91, "resets_at": "2026-10-20T09:00:00Z"}}
        five = self.usage("five_hour", fixture)
        seven = self.usage("seven_day", fixture)
        self.assertIn("42%", five["text"])
        self.assertEqual(five["class"], "ok")
        self.assertIn("91%", seven["text"])
        self.assertEqual(seven["class"], "critical")

    def test_unknown_values_render_a_dash(self):
        self.assertEqual(self.usage("five_hour", {"five_hour": None, "seven_day": None})["class"], "unknown")


class BarVisibilityTests(Sandbox):
    def setUp(self):
        super().setUp()
        self.shim("hyprctl", '[[ $1 == -j && $2 == activeworkspace ]] && echo "{\\"id\\": $(cat "$SHIM_WS_FILE")}"; exit 0')
        self.shim("pkill", "")
        self.workspace = self.root / "ws"
        self.workspace.write_text("2\n")

    def visibility(self, command: str):
        subprocess.run([str(WAYBAR_SCRIPTS / "waybar-visibility.sh"), command], check=True, env={**self.env, "SHIM_WS_FILE": str(self.workspace)})

    def toggles(self) -> int:
        return len(self.calls("pkill"))

    def test_hides_once_on_the_claude_workspace_and_shows_once_back(self):
        self.visibility("reset")
        self.visibility("sync")
        self.assertEqual(self.toggles(), 0)
        self.workspace.write_text("6\n")
        self.visibility("sync")
        self.visibility("sync")
        self.assertEqual(self.toggles(), 1)
        self.workspace.write_text("3\n")
        self.visibility("sync")
        self.assertEqual(self.toggles(), 2)

    def test_a_pre_hide_is_not_toggled_twice_by_the_following_event(self):
        self.visibility("reset")
        self.visibility("hide")
        self.workspace.write_text("6\n")
        self.visibility("sync")
        self.assertEqual(self.toggles(), 1)
        self.assertEqual(self.calls("pkill"), ["pkill -x -USR1 waybar"])


if __name__ == "__main__":
    unittest.main()
