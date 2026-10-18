-- Claude Code sessions. ~/.config/osd/osd.py keeps the registry (hooks run inside greywall with
-- no Hyprland socket) and calls CLAUDE_WINDOW_STATE below with `hyprctl eval`: the terminal a
-- session runs in carries exactly one claude-<state> tag, and the rules colour its border by state.
-- The claude-sessions manager (ALT + C) is the one window that lists and shows every session.
local colors = require("colors")
local STATE_TAGS = { "claude-working", "claude-waiting", "claude-needs-you" }
-- Workspace 6: right after Steam (5); the games moved to 7 to make room.
-- The same number lives in waybar/scripts/on-claude-workspace.sh (what every script asks) and in
-- scratchpads.lua (CLAUDE_WORKSPACE): change all three together.
local WORKSPACE = 6

hl.window_rule({ match = { tag = "claude-needs-you", focus = true }, border_color = "rgba(" .. colors.red .. "ee)" })
hl.window_rule({ match = { tag = "claude-needs-you", focus = false }, border_color = "rgba(" .. colors.red .. "aa)" })
hl.window_rule({ match = { tag = "claude-waiting", focus = true }, border_color = "rgba(" .. colors.yellow .. "ee)" })
hl.window_rule({ match = { tag = "claude-waiting", focus = false }, border_color = "rgba(" .. colors.yellow .. "aa)" })
-- Working keeps the theme's focused gradient; only the unfocused border says "a session lives here".
hl.window_rule({ match = { tag = "claude-working", focus = false }, border_color = "rgba(" .. colors.blue .. "99)" })

-- The cockpit: no gaps, border, rounding or shadow, and the bar hides itself here (waybar's
-- workspace-listener), so the manager fills the screen like an app of its own.
hl.workspace_rule({
    workspace = tostring(WORKSPACE),
    persistent = true,
    gaps_in = 0,
    gaps_out = 0,
    no_border = true,
    no_rounding = true,
    no_shadow = true,
})
-- Fully opaque: a wall of session output over the wallpaper was hard to read at the global 0.95/0.9.
hl.window_rule({ match = { class = "^claude-sessions$" }, workspace = tostring(WORKSPACE), opacity = "1 1" })

-- `hyprctl eval 'CLAUDE_WINDOW_STATE(4242, "waiting")'`; nil clears every state tag.
function CLAUDE_WINDOW_STATE(pid, state)
    local window = hl.get_window("pid:" .. pid)
    if not window then
        error("no window for pid " .. pid)
    end
    for _, tag in ipairs(STATE_TAGS) do
        hl.dispatch(hl.dsp.window.tag({ window = window, tag = "-" .. tag }))
    end
    if state then
        hl.dispatch(hl.dsp.window.tag({ window = window, tag = "+claude-" .. state }))
    end
end

-- The "still there?" screen before the lock on this workspace (scripts/claude-idle.sh) sits over
-- a blurred desktop; it is translucent itself, so what is behind it stays readable.
hl.layer_rule({ match = { namespace = "^claude-idle$" }, blur = true, ignore_alpha = 0.3 })

-- The approval window for a command Claude asks to run outside the sandbox (claude-run-approve),
-- when the cockpit is not on screen: floating, centred, above the work.
hl.window_rule({ match = { class = "^claude-run$" }, float = true, center = true, size = "1100 640" })

-- Steps Claude hands over to do by hand (claude-guide): a narrow floating window at the right edge,
-- next to the browser; an ordinary window otherwise, moved to any workspace like the rest.
hl.window_rule({
    match = { class = "^claude-guide$" },
    float = true,
    size = "560 (monitor_h*0.85)",
    move = "(monitor_w-window_w-24) (monitor_h*0.08)",
})
