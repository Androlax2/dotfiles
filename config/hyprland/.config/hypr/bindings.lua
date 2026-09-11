-- Application bindings
-- See https://wiki.hypr.land/Configuring/Basics/Binds/
local mainMod = "ALT"

-- Set programs that you use
local terminal = "kitty"
local fileManager = "dolphin" ---@diagnostic disable-line: unused-local
local menu = "walker"

-- --- GAME MODE LOGIC ---
-- Press ALT + G to enter Game Mode. This turns off eye-candy and enters the submap.
hl.bind(mainMod .. " + G", function()
    hl.config({
        animations = { enabled = false },
        decoration = {
            rounding = 0,
            shadow = { enabled = false },
            blur = { enabled = false },
        },
    })
    hl.dispatch(hl.dsp.submap("gamemode"))
end)

hl.define_submap("gamemode", function()
    -- ALT + G exits: the reload restores the config values changed on entry.
    hl.bind(mainMod .. " + G", function()
        hl.dispatch(hl.dsp.exec_cmd("hyprctl reload"))
        hl.dispatch(hl.dsp.submap("reset"))
    end)

    -- Multimedia (Volume/Brightness)
    hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+"), { locked = true, repeating = true })
    hl.bind("XF86AudioLowerVolume", hl.dsp.exec_cmd("wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"), { locked = true, repeating = true })
    hl.bind("XF86AudioMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"), { locked = true, repeating = true })
    hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd("~/.config/hypr/brightness-adjust-focused.sh +10"), { locked = true })
    hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd("~/.config/hypr/brightness-adjust-focused.sh -10"), { locked = true })
end)
-- --- END OF GAME MODE ---

hl.bind(mainMod .. " + Q", hl.dsp.exec_cmd(terminal))
hl.bind(mainMod .. " + W", hl.dsp.window.close(), { description = "Close window" })
hl.bind(mainMod .. " + F", hl.dsp.window.fullscreen_state({ internal = 3, client = -1 }))
hl.bind("CTRL + SPACE", hl.dsp.exec_cmd(menu))
hl.bind(mainMod .. " + J", hl.dsp.layout("togglesplit"), { description = "Toggle window split" })

hl.bind("CTRL + SHIFT + code:13", hl.dsp.exec_cmd("hyprshot -m region --clipboard-only"), { description = "Screenshot a region" })
hl.bind("CTRL + SHIFT + code:14", hl.dsp.exec_cmd("~/.config/screen-record.sh --silent"), { description = "Start recording" })
hl.bind("SUPER + CTRL + SPACE", hl.dsp.exec_cmd("walker -m symbols"), { description = "Emoji picker" })

-- TAB between workspaces
hl.bind(mainMod .. " + TAB", hl.dsp.focus({ workspace = "e+1" }), { description = "Next workspace" })
hl.bind(mainMod .. " + SHIFT + TAB", hl.dsp.focus({ workspace = "e-1" }), { description = "Previous workspace" })

-- Move focus with SUPER + arrow keys
hl.bind("SUPER + left", hl.dsp.focus({ direction = "l" }))
hl.bind("SUPER + right", hl.dsp.focus({ direction = "r" }))
hl.bind("SUPER + up", hl.dsp.focus({ direction = "u" }))
hl.bind("SUPER + down", hl.dsp.focus({ direction = "d" }))

-- Move window placement
hl.bind("SUPER + SHIFT + left", hl.dsp.window.move({ direction = "l" }))
hl.bind("SUPER + SHIFT + right", hl.dsp.window.move({ direction = "r", group_aware = true }))
hl.bind("SUPER + SHIFT + down", hl.dsp.window.move({ direction = "d", group_aware = true }))
hl.bind("SUPER + SHIFT + up", hl.dsp.window.move({ direction = "u", group_aware = true }))

-- Switch workspaces with mainMod + [0-9], move the active window there with
-- mainMod + SHIFT + [0-9]. Keycodes because AZERTY: code:10..19 is the number row.
for i = 1, 10 do
    local key = "code:" .. (9 + i)
    hl.bind(mainMod .. " + " .. key, hl.dsp.focus({ workspace = i }))
    hl.bind(mainMod .. " + SHIFT + " .. key, hl.dsp.window.move({ workspace = i }))
end

-- Example special workspace (scratchpad)
hl.bind(mainMod .. " + SHIFT + S", hl.dsp.workspace.toggle_special("magic"))
-- hl.bind(mainMod .. " + SHIFT + S", hl.dsp.window.move({ workspace = "special:magic" }))

-- Scroll through existing workspaces with mainMod + scroll
hl.bind(mainMod .. " + mouse_down", hl.dsp.focus({ workspace = "e+1" }))
hl.bind(mainMod .. " + mouse_up", hl.dsp.focus({ workspace = "e-1" }))

-- Move/resize windows with mainMod + LMB/RMB and dragging
-- hl.bind(mainMod .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
-- hl.bind(mainMod .. " + mouse:273", hl.dsp.window.resize(), { mouse = true })

-- Laptop multimedia keys for volume and LCD brightness
hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+"), { locked = true, repeating = true })
hl.bind("XF86AudioLowerVolume", hl.dsp.exec_cmd("wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"), { locked = true, repeating = true })
hl.bind("XF86AudioMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"), { locked = true, repeating = true })
hl.bind("XF86AudioMicMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"), { locked = true, repeating = true })
hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd("~/.config/hypr/brightness-adjust-focused.sh +10"), { locked = true })
hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd("~/.config/hypr/brightness-adjust-focused.sh -10"), { locked = true })

-- Requires playerctl
hl.bind("XF86AudioNext", hl.dsp.exec_cmd("playerctl next"), { locked = true })
hl.bind("XF86AudioPause", hl.dsp.exec_cmd("playerctl --all-players play-pause"), { locked = true })
hl.bind("XF86AudioPlay", hl.dsp.exec_cmd("playerctl --all-players play-pause"), { locked = true })
hl.bind("XF86AudioPrev", hl.dsp.exec_cmd("playerctl previous"), { locked = true })

-- Toggle the LG ultrawide between SDR and HDR (see ~/.local/bin/hdr-toggle.sh).
-- SDR for desktop work (brighter overall), HDR for native-HDR games.
hl.bind(mainMod .. " + SHIFT + H", hl.dsp.exec_cmd("~/.local/bin/hdr-toggle.sh"))
