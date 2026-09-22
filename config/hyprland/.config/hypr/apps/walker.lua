-- The launcher is an island like the bar: no animation, blurred ground, transparent pixels ignored.
hl.layer_rule({ match = { namespace = "walker" }, no_anim = true, blur = true, ignore_alpha = 0.2 })
