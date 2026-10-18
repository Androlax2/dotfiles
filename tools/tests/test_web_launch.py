"""web-launch.sh: the web scratchpads' Zen profiles get the shared prefs on every start."""

import subprocess

from support import REPO, Sandbox

WEB_LAUNCH = REPO / "config/hyprland/.config/hypr/scripts/web-launch.sh"
PREFS = REPO / "config/zen/.config/zen/scratchpad.user.js"


class WebLaunchTests(Sandbox):
    def setUp(self):
        super().setUp()
        (self.home / ".config/zen").mkdir(parents=True)
        (self.home / ".config/zen/scratchpad.user.js").symlink_to(PREFS)
        # -CreateProfile makes the folder; a start just logs its arguments.
        self.shim("zen-browser", '[[ $1 == -CreateProfile ]] && mkdir -p "${2#* }"; exit 0')

    def launch(self, url):
        subprocess.run(["bash", str(WEB_LAUNCH), "prs", url], env=self.env, check=True)
        return (self.home / ".zen/prs/user.js").read_text()

    def test_a_profile_keeps_its_home_page_and_gets_the_one_tab_prefs(self):
        first = self.launch("https://github.com/pulls")
        self.assertIn('user_pref("browser.startup.homepage", "https://github.com/pulls");', first)
        again = self.launch("https://github.com/o/r/pull/7")  # started straight on a pull request
        self.assertIn('user_pref("browser.startup.homepage", "https://github.com/pulls");', again)
        self.assertNotIn("pull/7", again)
        self.assertIn('user_pref("browser.link.open_newwindow.override.external", 1);', again)
        self.assertEqual(self.calls("zen-browser")[-1], "zen-browser -P prs https://github.com/o/r/pull/7")
