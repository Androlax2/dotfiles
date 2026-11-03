-- Blur the Waybar islands only. Blur is enabled in theme.lua for the layer, then
-- switched off for every window so the 0.95 / 0.9 opacity windows look as before.
-- ignore_alpha skips the transparent bar background and the anti-aliased island corners.
hl.layer_rule({ match = { namespace = "waybar" }, blur = true, ignore_alpha = 0.2 })
hl.window_rule({ match = { class = ".*" }, no_blur = true })
