-- Picture-in-picture overlays
hl.window_rule({ match = { title = "(Picture.?in.?[Pp]icture)" }, tag = "+pip" })
hl.window_rule({
    match = { tag = "pip" },
    float = true,
    pin = true,
    size = "600 338",
    keep_aspect_ratio = true,
    border_size = 0,
    opacity = "1 1",
    move = "(monitor_w-window_w-40) (monitor_h*0.04)",
})
