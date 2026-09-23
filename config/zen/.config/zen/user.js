// Zen prefs for the main profile, linked in by `make zen`.
// Tokyo Night chrome (userChrome.css), dark content, blue accent. Compact mode is
// left to the user's own setting.
user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);
user_pref("ui.systemUsesDarkTheme", 1);
user_pref("layout.css.prefers-color-scheme.content-override", 0);
user_pref("browser.theme.toolbar-theme", 0);
user_pref("browser.theme.content-theme", 0);
user_pref("zen.theme.accent-color", "#7aa2f7");
user_pref("zen.theme.color-prefs.use-workspace-colors", false);
