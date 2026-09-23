hl.window_rule({
    match = { class = "^([Bb]itwarden)$" }, -- "bitwarden" on native Wayland, "Bitwarden" under XWayland
    no_screen_share = true,
    float = true,
    center = true,
    size = "1100 720",
})
