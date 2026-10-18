// Prefs of every web scratchpad profile (music, prs, claude, greyproxy): a compact-mode window
// on one site. ~/.config/hypr/scripts/web-launch.sh copies this to ~/.zen/<profile>/user.js on
// the first run and appends the site as browser.startup.homepage.
user_pref("browser.startup.page", 1);
user_pref("browser.sessionstore.resume_from_crash", false);
user_pref("browser.shell.checkDefaultBrowser", false);
user_pref("browser.aboutwelcome.enabled", false);
user_pref("zen.welcome-screen.seen", true);
user_pref("zen.view.compact.enable-at-startup", true);
user_pref("zen.theme.accent-color", "#7aa2f7");
user_pref("ui.systemUsesDarkTheme", 1);
user_pref("layout.css.prefers-color-scheme.content-override", 0);
user_pref("browser.theme.toolbar-theme", 0);
user_pref("browser.theme.content-theme", 0);
user_pref("media.hardwaremediakeys.enabled", true);
