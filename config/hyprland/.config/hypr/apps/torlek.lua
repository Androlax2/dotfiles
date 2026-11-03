-- The Torlek TUI opens as `kitty --class torlek torlek tui`: no see-through background behind it.
hl.window_rule({ match = { class = "^(torlek)$" }, opacity = "1 1" })
