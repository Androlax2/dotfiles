-- https://wiki.hypr.land/Configuring/Advanced-and-Cool/Animations/
-- One curve for everything; leaves not listed here inherit from their parent.

hl.config({
    animations = {
        enabled = true,
    },
})

hl.curve("ease", { type = "bezier", points = { { 0.2, 0.8 }, { 0.2, 1 } } })

hl.animation({ leaf = "windows",    enabled = true, speed = 3, bezier = "ease", style = "popin 85%" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 3, bezier = "ease", style = "popin 85%" })
hl.animation({ leaf = "fade",       enabled = true, speed = 2, bezier = "ease" })
hl.animation({ leaf = "workspaces", enabled = true, speed = 1.5, bezier = "ease", style = "fade" })
hl.animation({ leaf = "layers",     enabled = true, speed = 2, bezier = "ease", style = "fade" })
-- Scratchpads slide down from under the bar.
hl.animation({ leaf = "specialWorkspace", enabled = true, speed = 3, bezier = "ease", style = "slidefadevert -100%" })
