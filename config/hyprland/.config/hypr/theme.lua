-- https://wiki.hypr.land/Configuring/Basics/Variables/

-- Tokyo Night (night): blue -> purple hairline on the focused window, bg_highlight on the rest.
-- Tokens and rules in assets/waybar-design/DESIGN.md.
local colors = require("colors") -- generated from the palette, see tools/palette.sh
local activeBorderColor = { colors = { "rgba(" .. colors.blue .. "cc)", "rgba(" .. colors.magenta .. "cc)" }, angle = 45 }
local inactiveBorderColor = "rgba(" .. colors.bgHighlight .. "99)"

hl.config({
    general = {
        gaps_in = 5,
        -- Top gap equals Waybar's 6 px top margin, so the space under the bar matches the space above it.
        gaps_out = { top = 6, right = 15, bottom = 15, left = 15 },

        border_size = 1,

        col = {
            active_border = activeBorderColor,
            inactive_border = inactiveBorderColor,
        },

        -- Set to true enable resizing windows by clicking and dragging on borders and gaps
        resize_on_border = true,

        -- Please see https://wiki.hypr.land/Configuring/Advanced-and-Cool/Tearing/ before you turn this on
        allow_tearing = false,

        layout = "dwindle",
    },

    group = {
        col = {
            border_active = activeBorderColor,
            border_inactive = inactiveBorderColor,
            border_locked_active = activeBorderColor,
            border_locked_inactive = inactiveBorderColor,
        },

        groupbar = {
            font_size = 12,
            font_family = "monospace",
            font_weight_active = "ultraheavy",
            font_weight_inactive = "normal",

            indicator_height = 0,
            indicator_gap = 5,
            height = 22,
            gaps_in = 0,
            gaps_out = 0,

            text_color = "rgb(" .. colors.fg .. ")",
            col = {
                active = "rgb(" .. colors.bgActive .. ")",
                inactive = "rgba(" .. colors.bgActive .. "66)",
            },

            gradients = true,
        },
    },

    decoration = {
        rounding = 10,
        rounding_power = 2,

        -- Change transparency of focused and unfocused windows
        active_opacity = 1.0,
        inactive_opacity = 1.0,

        -- Soft drop shadow; the gamemode bind turns it off while gaming.
        shadow = {
            enabled = true,
            range = 20,
            render_power = 3,
            color = "rgba(00000059)",
            offset = { 0, 4 },
        },

        -- Barely-there dim so the focused window reads without a loud border.
        dim_inactive = true,
        dim_strength = 0.08,

        -- Only the Waybar islands are blurred: apps/waybar.lua turns blur off for every window.
        blur = {
            enabled = true,
            size = 6,
            passes = 3,
            new_optimizations = true,

            -- vibrancy = 0.1696,
        },
    },
})
