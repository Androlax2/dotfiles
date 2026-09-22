-- Notifications are islands like the bar: blurred ground, transparent pixels ignored.
hl.layer_rule({ match = { namespace = "notifications" }, blur = true, ignore_alpha = 0.2 })
