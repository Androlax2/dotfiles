-- Settings dialogs float centred instead of tiling; file pickers get a wider frame.
-- Border and rounding come from theme.lua, so they match tiled windows.
hl.window_rule({
    match = { class = "^(org\\.pulseaudio\\.pavucontrol|pavucontrol|\\.blueman-manager-wrapped|blueman-manager|nm-connection-editor)$" },
    float = true,
    center = true,
    size = "960 640",
})
hl.window_rule({
    match = { class = "^(xdg-desktop-portal-gtk|xdg-desktop-portal-hyprland)$" },
    float = true,
    center = true,
    size = "1100 700",
})
