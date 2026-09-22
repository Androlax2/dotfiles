-- Notification cards and the notification center are islands like the bar: blurred ground,
-- transparent pixels ignored.
hl.layer_rule({ match = { namespace = "swaync-notification-window" }, blur = true, ignore_alpha = 0.2 })
hl.layer_rule({ match = { namespace = "swaync-control-center" }, blur = true, ignore_alpha = 0.2 })
