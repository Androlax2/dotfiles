hl.window_rule({ match = { class = "^(jetbrains-.*)$" }, opacity = "1 1" })
hl.window_rule({ match = { class = "^jetbrains-idea$" }, workspace = "1" })

-- Fix splash screen showing in weird places and prevent annoying focus takeovers
hl.window_rule({ match = { class = "^(jetbrains-.*)$", title = "^(splash)$", float = true }, tag = "+jetbrains-splash" })
hl.window_rule({
    match = { tag = "jetbrains-splash" },
    center = true,
    no_focus = true,
    border_size = 0,
})

-- Center popups/find windows
hl.window_rule({ match = { class = "^(jetbrains-.*)", title = "^()$", float = true }, tag = "+jetbrains" })
hl.window_rule({ match = { tag = "jetbrains" }, center = true })

-- Enabling this makes it possible to provide input in popup dialogs (search window, new file, etc.)
hl.window_rule({
    match = { tag = "jetbrains" },
    stay_focused = true,
    border_size = 0,
})

-- For some reason tag:jetbrains does not work for size rule
hl.window_rule({ match = { class = "^(jetbrains-.*)", title = "^()$", float = true }, min_size = "(monitor_w*0.5) (monitor_h*0.5)" })

-- Disable window flicker when autocomplete or tooltips appear
hl.window_rule({ match = { class = "^(jetbrains-.*)$", title = "^(win.*)$", float = true }, no_initial_focus = true })

-- Disable mouse focus
hl.window_rule({ match = { class = "^(jetbrains-.*)$" }, no_follow_mouse = true })
