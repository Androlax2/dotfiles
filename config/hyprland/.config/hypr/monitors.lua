-- See https://wiki.hypr.land/Configuring/Basics/Monitors/

-- Optimized for retina-class 2x displays, like 13" 2.8K, 27" 5K, 32" 6K.
-- hl.env("GDK_SCALE", "2")

-- DP-1 runs in one of two modes, selected by the HDR flag below. The
-- hdr-toggle.sh script in ~/.local/bin/ flips the flag and reloads Hyprland,
-- so keep the line exactly in the form `local HDR = false`.
--
-- SDR mode: brighter overall panel output, no HDR pipeline. Good for
--   desktop, browsing, terminal, video that isn't HDR-mastered.
-- HDR mode: HDR10 (BT.2020 PQ). Highlights bright, panel reserves
--   headroom so SDR content ends up dimmer; sdrbrightness boosts SDR
--   reference luminance but can't reach SDR-mode peak. Good for games
--   with native HDR (DXVK_HDR=1, e.g. Elden Ring with our setup).

local HDR = true

if HDR then
    hl.monitor({
        output = "DP-1",
        mode = "3840x1600@144.05000",
        position = "0x0",
        scale = 1,
        bitdepth = 10,
        cm = "hdr",
        sdrbrightness = 3.0,
        vrr = 1,
    })
else
    hl.monitor({
        output = "DP-1",
        mode = "3840x1600@144.05000",
        position = "0x0",
        scale = 1,
        bitdepth = 10,
        vrr = 1,
    })
end

-- hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })
