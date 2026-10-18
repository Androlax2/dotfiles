-- Autostart necessary processes (like notifications daemons, status bars, etc.)
-- See https://wiki.hypr.land/Configuring/Basics/Autostart/
hl.on("hyprland.start", function()
    -- The bar first, so it is on screen before the heavier startup jobs compete for the CPU.
    -- Its log goes to the runtime dir so a slow start can be timed against Hyprland's own log.
    hl.exec_cmd('waybar > "$XDG_RUNTIME_DIR/waybar.log" 2>&1')
    hl.exec_cmd("swaync")
    -- The OSD daemon, the Claude session manager, elephant and walker are systemd user units
    -- (journal logs, restart on failure): hand them the session's environment first, then start
    -- them. Apps launched from walker inherit it, so it carries everything envs.lua sets (theme,
    -- cursor, Wayland backends) and PATH, as an app started from a Hyprland bind would get.
    local session_environment = table.concat({
        "WAYLAND_DISPLAY", "DISPLAY", "HYPRLAND_INSTANCE_SIGNATURE", "PATH",
        "XDG_CURRENT_DESKTOP", "XDG_SESSION_DESKTOP", "XDG_SESSION_TYPE",
        "XCURSOR_THEME", "XCURSOR_SIZE", "HYPRCURSOR_THEME", "HYPRCURSOR_SIZE",
        "GDK_BACKEND", "QT_QPA_PLATFORM", "QT_QPA_PLATFORMTHEME", "SDL_VIDEODRIVER",
        "MOZ_ENABLE_WAYLAND", "ELECTRON_OZONE_PLATFORM_HINT", "OZONE_PLATFORM",
    }, " ")
    -- restart, not start: the units are enabled at login, so systemd may have started them before
    -- Hyprland existed, without its display; walker's apps then fail to open.
    hl.exec_cmd("systemctl --user import-environment " .. session_environment
        .. " && systemctl --user restart osd.service elephant.service walker.service"
        .. " && systemctl --user start claude-sessions.service")
    hl.exec_cmd("~/.config/mpris-card/card.py") -- resident so the card shows instantly
    -- No -n: it would pop a Hyprland notification with a progress bar on every login.
    hl.exec_cmd("hyprpm reload")
    -- From the session, not a root unit: OpenRGB's udev rules give i2c/hidraw access to the
    -- logged-in seat, which is what detects the RAM sticks; root at boot missed them.
    hl.exec_cmd("openrgb --noautoconnect --profile 'Vive le bleu'")
    hl.exec_cmd("~/.local/bin/trcc gui --resume")
    hl.exec_cmd("~/.config/hypr/scripts/hyprsunset-if-not-gaming.sh")
    hl.exec_cmd("hypridle")
    -- Routes the media keys and the bar's mpris island to the last active player.
    hl.exec_cmd("playerctld daemon")
    hl.exec_cmd("~/.config/hypr/hotcorner.sh")
    hl.exec_cmd("~/.config/hypr/wallpaper-rotator.sh")
    hl.exec_cmd("~/.config/hypr/scripts/dofus-mouse-remap.sh")
    hl.exec_cmd("synology-drive")
    hl.exec_cmd("librepods --hide")
end)
