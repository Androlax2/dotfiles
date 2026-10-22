// Prefs of every web scratchpad profile (music, prs, claude, greyproxy): a compact-mode window
// on one site, one tab, the fewest content processes. ~/.config/hypr/scripts/web-launch.sh copies this to ~/.zen/<profile>/user.js
// on every start (so a change here reaches existing profiles) and adds the site as
// browser.startup.homepage.
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
// One tab: a page sent from another app (cockpit' pull requests) replaces the current one
// instead of piling up tabs, and links asking for a new tab or window stay in this one.
user_pref("browser.link.open_newwindow.override.external", 1);
user_pref("browser.link.open_newwindow", 1);
// One site per profile, so the desktop defaults (3 prelaunched content processes, up to 4 per
// site) only cost RAM here: each scratchpad keeps one process for its site.
user_pref("dom.ipc.processPrelaunch.enabled", false);
user_pref("dom.ipc.processCount", 1);
user_pref("dom.ipc.processCount.webIsolated", 1);
