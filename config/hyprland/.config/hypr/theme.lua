-- https://wiki.hypr.land/Configuring/Basics/Variables/

-- TokyoNight
local activeBorderColor = { colors = { "rgba(33ccffee)", "rgba(00ff99ee)" }, angle = 45 }
local inactiveBorderColor = "rgb(1A1B26)"

hl.config({
    general = {
        gaps_in = 5,
        gaps_out = 15,

        border_size = 2,

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

            text_color = "rgb(C0CAF5)",
            col = {
                active = "rgb(2f3344)",
                inactive = "rgba(2f334466)",
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

        shadow = {
            enabled = false,
            range = 2,
            render_power = 3,
            color = "rgba(1a1a1aee)",
        },

        blur = {
            enabled = false,
            size = 3,
            passes = 3,

            -- vibrancy = 0.1696,
        },
    },
})
