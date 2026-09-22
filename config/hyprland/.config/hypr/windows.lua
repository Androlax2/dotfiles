-- See https://wiki.hypr.land/Configuring/Basics/Window-Rules/

-- Just dash of opacity by default
hl.window_rule({
    match = { class = ".*" },
    suppress_event = "maximize",
    opacity = "0.95 0.9",
})

-- Fix some dragging issues with XWayland
hl.window_rule({
    match = {
        class = "^$",
        title = "^$",
        xwayland = true,
        float = true,
        fullscreen = false,
        pin = false,
    },
    no_focus = true,
})

-- App-specific tweaks
require("apps/pip")
require("apps/browser")
require("apps/terminals")
require("apps/webcam-overlay")
require("apps/bitwarden")
require("apps/jetbrains")
require("apps/hyprshot")
require("apps/system")
require("apps/walker")
require("apps/waybar")
require("apps/dunst")
require("apps/steam")
require("apps/libreoffice")
require("apps/ganymede")
require("apps/dofus")
