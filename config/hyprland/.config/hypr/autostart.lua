-- Autostart necessary processes (like notifications daemons, status bars, etc.)
-- See https://wiki.hypr.land/Configuring/Basics/Autostart/
hl.on("hyprland.start", function()
    -- The bar first, so it is on screen before the heavier startup jobs compete for the CPU.
    -- Its log goes to the runtime dir so a slow start can be timed against Hyprland's own log.
    hl.exec_cmd('waybar > "$XDG_RUNTIME_DIR/waybar.log" 2>&1')
    hl.exec_cmd("swaync")
    hl.exec_cmd("~/.config/osd/osd.py")
    hl.exec_cmd("~/.config/mpris-card/card.py") -- resident so the card shows instantly
    -- No -n: it would pop a Hyprland notification with a progress bar on every login.
    hl.exec_cmd("hyprpm reload")
    -- From the session, not a root unit: OpenRGB's udev rules give i2c/hidraw access to the
    -- logged-in seat, which is what detects the RAM sticks; root at boot missed them.
    hl.exec_cmd("openrgb --noautoconnect --profile 'Vive le bleu'")
    hl.exec_cmd("~/.config/hypr/scripts/hyprsunset-if-not-gaming.sh")
    hl.exec_cmd("hypridle")
    -- Routes the media keys and the bar's mpris island to the last active player.
    hl.exec_cmd("playerctld daemon")
    hl.exec_cmd("elephant")
    hl.exec_cmd("walker --gapplication-service")
    hl.exec_cmd("~/.config/hypr/hotcorner.sh")
    hl.exec_cmd("~/.config/hypr/wallpaper-rotator.sh")
    hl.exec_cmd("~/.config/hypr/scripts/dofus-mouse-remap.sh")
    hl.exec_cmd("synology-drive")
    hl.exec_cmd("librepods --hide")
end)
