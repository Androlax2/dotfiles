"""The hot corner (hypr/hotcorner.sh): locks in the bottom-left corner, cheaply."""

import subprocess
import time

from support import REPO, Sandbox

SCRIPT = REPO / "config/hyprland/.config/hypr/hotcorner.sh"
HYPR_SCRIPTS = REPO / "config/hyprland/.config/hypr/scripts"


class HotCornerTests(Sandbox):
    def setUp(self):
        super().setUp()
        scripts = self.home / ".config/hypr/scripts"
        scripts.mkdir(parents=True)
        (scripts / "is-gaming.sh").symlink_to(HYPR_SCRIPTS / "is-gaming.sh")
        self.shim("lock", "")

    def watch(self, cursor: str, seconds: float = 1.6):
        self.shim("hyprctl", f"""case "$*" in
  cursorpos) echo "{cursor}" ;;
  "monitors -j") echo '[{{"x": 0, "y": 0, "width": 3840, "height": 1600}}]' ;;
  "activewindow -j") echo '{{"class": "kitty", "fullscreen": 0}}' ;;
esac""")
        corner = subprocess.Popen(["bash", str(SCRIPT)], env={**self.env, "HOTCORNER_COMMAND": str(self.shims / "lock")})
        time.sleep(seconds)
        corner.kill()
        corner.wait()

    def test_the_corner_locks(self):
        self.watch("3, 1597")
        self.assertGreaterEqual(len(self.calls("lock")), 1)

    def test_elsewhere_nothing_happens(self):
        self.watch("1920, 800")
        self.assertEqual(self.calls("lock"), [])

    def test_the_layout_is_not_read_on_every_poll(self):
        self.watch("1920, 800")
        calls = self.calls("hyprctl")
        polls = [call for call in calls if call == "hyprctl cursorpos"]
        self.assertGreaterEqual(len(polls), 6)
        self.assertEqual(len([call for call in calls if call == "hyprctl monitors -j"]), 1)
