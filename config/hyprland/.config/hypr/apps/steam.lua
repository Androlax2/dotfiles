hl.window_rule({
    match = { class = "steam" },
    workspace = "5",
    opacity = "1 1",
    idle_inhibit = "fullscreen",
})
hl.window_rule({ match = { class = "^(steam_app_.*)$" }, no_blur = true, idle_inhibit = "fullscreen" })
hl.window_rule({ match = { class = "^(steam_app_*)$" }, immediate = true })
hl.window_rule({ match = { class = "^(steam)$" }, idle_inhibit = "fullscreen" })
