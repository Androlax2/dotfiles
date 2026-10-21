-- Volume/brightness OSD (osd, from agent-cockpit): blurred island like the bar.
hl.layer_rule({ match = { namespace = "osd" }, blur = true, ignore_alpha = 0.2 })
