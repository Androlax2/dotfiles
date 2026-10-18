-- Now-playing card (mpris-card, apps/desktop): blurred island like the bar.
hl.layer_rule({ match = { namespace = "mpris-card" }, blur = true, ignore_alpha = 0.2 })
