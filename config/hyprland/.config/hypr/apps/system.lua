-- Floating windows
hl.window_rule({
    match = { tag = "floating-window" },
    float = true,
    center = true,
    size = "875 600",
})
hl.window_rule({ match = { class = "(blueberry.py|com.omarchy.Impala|com.omarchy.Wiremix|com.omarchy.Omarchy|org.gnome.NautilusPreviewer|com.gabm.satty|Omarchy|About|TUI.float)" }, tag = "+floating-window" })
hl.window_rule({
    match = {
        class = "(xdg-desktop-portal-gtk|sublime_text|DesktopEditors|org.gnome.Nautilus)",
        title = "^(Open.*Files?|Open [F|f]older.*|Save.*Files?|Save.*As|Save|All Files|.*wants to [open|save].*|[C|c]hoose.*)",
    },
    tag = "+floating-window",
})
hl.window_rule({ match = { class = "org.gnome.Calculator" }, float = true })

-- Fullscreen screensaver
hl.window_rule({ match = { class = "Screensaver" }, fullscreen = true })

-- No transparency on media windows
hl.window_rule({ match = { class = "^(zoom|vlc|mpv|org.kde.kdenlive|com.obsproject.Studio|com.github.PintaProject.Pinta|imv|org.gnome.NautilusPreviewer)$" }, opacity = "1 1" })

-- Popped window rounding
hl.window_rule({ match = { tag = "pop" }, rounding = 8 })
