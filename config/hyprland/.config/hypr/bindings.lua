-- Application bindings
-- See https://wiki.hypr.land/Configuring/Basics/Binds/
local mainMod = "ALT"

-- Set programs that you use
local terminal = "kitty"
local fileManager = "io.elementary.files" ---@diagnostic disable-line: unused-local
local menu = "walker"
local bind = require("binds").bind

-- --- GAME MODE LOGIC ---
-- ALT + G enters Game Mode (game-mode.lua also offers it when a game window opens).
local game_mode = require("game-mode")
bind(mainMod .. " + G", game_mode.enter, { description = "Game mode (eye candy off)" })

hl.define_submap("gamemode", function()
    -- ALT + G exits: the reload restores the config values changed on entry.
    hl.bind(mainMod .. " + G", function()
        hl.dispatch(hl.dsp.exec_cmd("hyprctl reload"))
        hl.dispatch(hl.dsp.submap("reset"))
    end)

    -- Multimedia (Volume/Brightness)
    hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("~/.config/hypr/scripts/volume.sh raise"), { locked = true, repeating = true })
    hl.bind("XF86AudioLowerVolume", hl.dsp.exec_cmd("~/.config/hypr/scripts/volume.sh lower"), { locked = true, repeating = true })
    hl.bind("XF86AudioMute", hl.dsp.exec_cmd("~/.config/hypr/scripts/volume.sh mute"), { locked = true, repeating = true })
    hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd("~/.config/hypr/brightness-adjust-focused.sh +10"), { locked = true })
    hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd("~/.config/hypr/brightness-adjust-focused.sh -10"), { locked = true })
    hl.bind("XF86AudioNext", hl.dsp.exec_cmd("playerctl next"), { locked = true })
    hl.bind("XF86AudioPause", hl.dsp.exec_cmd("playerctl play-pause"), { locked = true })
    hl.bind("XF86AudioPlay", hl.dsp.exec_cmd("playerctl play-pause"), { locked = true })
    hl.bind("XF86AudioPrev", hl.dsp.exec_cmd("playerctl previous"), { locked = true })
end)
-- --- END OF GAME MODE ---

bind(mainMod .. " + Q", hl.dsp.exec_cmd(terminal), { description = "Terminal" })
bind(mainMod .. " + W", hl.dsp.window.close(), { description = "Close window" })
bind(mainMod .. " + F", hl.dsp.window.fullscreen_state({ internal = 3, client = -1 }), { description = "Fullscreen" })
bind("CTRL + SPACE", hl.dsp.exec_cmd(menu), { description = "Launcher" })
bind(mainMod .. " + J", hl.dsp.layout("togglesplit"), { description = "Toggle window split" })

-- Capture, macOS layout: CTRL is the physical Super key (keyd swap), code:12/13/14 = 3/4/5 on AZERTY.
bind("CTRL + SHIFT + code:12", hl.dsp.exec_cmd("~/.config/screenshot.sh full"), { description = "Screenshot the monitor" })
bind("CTRL + SHIFT + code:13", hl.dsp.exec_cmd("~/.config/screenshot.sh area"), { description = "Screenshot an area or a window" })
bind("CTRL + SHIFT + code:14", hl.dsp.exec_cmd("~/.config/screen-record.sh"), { description = "Start / stop recording" })
bind("SUPER + CTRL + SPACE", hl.dsp.exec_cmd("walker -m symbols"), { description = "Emoji picker" })

-- Launcher modes (walker + elephant) on physical Ctrl+Cmd+letter, like the emoji picker above:
-- keyd swaps Ctrl and Super, and Cmd+Shift+C/V would steal kitty's copy and paste.
bind("SUPER + CTRL + Q", hl.dsp.exec_cmd("walker -m menus:power"), { description = "Power menu" })
bind("SUPER + CTRL + W", hl.dsp.exec_cmd("~/.config/walker/scripts/wifi.sh"), { description = "Wi-Fi picker" })
bind("SUPER + CTRL + B", hl.dsp.exec_cmd("~/.config/walker/scripts/bluetooth.sh"), { description = "Bluetooth picker" })
bind("SUPER + CTRL + V", hl.dsp.exec_cmd("walker -m clipboard"), { description = "Clipboard history" })
bind("SUPER + CTRL + E", hl.dsp.exec_cmd("walker -m symbols"), { description = "Emoji picker" })
bind("SUPER + CTRL + C", hl.dsp.exec_cmd("walker -m calc"), { description = "Calculator" })
bind("SUPER + CTRL + K", hl.dsp.exec_cmd("~/.config/walker/scripts/keybinds.sh"), { description = "Keybinding cheat sheet" })

-- TAB between workspaces
bind(mainMod .. " + TAB", hl.dsp.focus({ workspace = "e+1" }), { description = "Next workspace" })
bind(mainMod .. " + SHIFT + TAB", hl.dsp.focus({ workspace = "e-1" }), { description = "Previous workspace" })

-- Move focus with SUPER + arrow keys
bind("SUPER + left", hl.dsp.focus({ direction = "l" }), { description = "Focus left" })
bind("SUPER + right", hl.dsp.focus({ direction = "r" }), { description = "Focus right" })
bind("SUPER + up", hl.dsp.focus({ direction = "u" }), { description = "Focus up" })
bind("SUPER + down", hl.dsp.focus({ direction = "d" }), { description = "Focus down" })

-- Move window placement
bind("SUPER + SHIFT + left", hl.dsp.window.move({ direction = "l" }), { description = "Move window left" })
bind("SUPER + SHIFT + right", hl.dsp.window.move({ direction = "r", group_aware = true }), { description = "Move window right" })
bind("SUPER + SHIFT + down", hl.dsp.window.move({ direction = "d", group_aware = true }), { description = "Move window down" })
bind("SUPER + SHIFT + up", hl.dsp.window.move({ direction = "u", group_aware = true }), { description = "Move window up" })

-- Switch workspaces with mainMod + [0-9], move the active window there with
-- mainMod + SHIFT + [0-9]. Keycodes because AZERTY: code:10..19 is the number row.
for i = 1, 10 do
    local key = "code:" .. (9 + i)
    bind(mainMod .. " + " .. key, hl.dsp.focus({ workspace = i }), { description = "Workspace " .. i })
    bind(mainMod .. " + SHIFT + " .. key, hl.dsp.window.move({ workspace = i }), { description = "Move window to workspace " .. i })
end

-- The dropdown terminal and Apple Music scratchpads are bound in scratchpads.lua.
bind(mainMod .. " + SHIFT + S", hl.dsp.workspace.toggle_special("magic"), { description = "Scratch workspace" })
-- hl.bind(mainMod .. " + SHIFT + S", hl.dsp.window.move({ workspace = "special:magic" }))

-- Scroll through existing workspaces with mainMod + scroll
bind(mainMod .. " + mouse_down", hl.dsp.focus({ workspace = "e+1" }), { description = "Next workspace" })
bind(mainMod .. " + mouse_up", hl.dsp.focus({ workspace = "e-1" }), { description = "Previous workspace" })

-- Move/resize windows with mainMod + LMB/RMB and dragging
-- hl.bind(mainMod .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
-- hl.bind(mainMod .. " + mouse:273", hl.dsp.window.resize(), { mouse = true })

-- Volume and monitor brightness keys (the scripts also drive the OSD, see ~/.config/osd)
bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("~/.config/hypr/scripts/volume.sh raise"), { locked = true, repeating = true, description = "Volume up" })
bind("XF86AudioLowerVolume", hl.dsp.exec_cmd("~/.config/hypr/scripts/volume.sh lower"), { locked = true, repeating = true, description = "Volume down" })
bind("XF86AudioMute", hl.dsp.exec_cmd("~/.config/hypr/scripts/volume.sh mute"), { locked = true, repeating = true, description = "Mute" })
bind("XF86AudioMicMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"), { locked = true, repeating = true, description = "Mute the microphone" })
bind("XF86MonBrightnessUp", hl.dsp.exec_cmd("~/.config/hypr/brightness-adjust-focused.sh +10"), { locked = true, description = "Brightness up" })
bind("XF86MonBrightnessDown", hl.dsp.exec_cmd("~/.config/hypr/brightness-adjust-focused.sh -10"), { locked = true, description = "Brightness down" })

-- Media keys go to playerctld's most recently active player, so the Zen tab (or the
-- Apple Music window) that last played gets them, not every MPRIS player at once.
bind("XF86AudioNext", hl.dsp.exec_cmd("playerctl next"), { locked = true, description = "Next track" })
bind("XF86AudioPause", hl.dsp.exec_cmd("playerctl play-pause"), { locked = true, description = "Play / pause" })
bind("XF86AudioPlay", hl.dsp.exec_cmd("playerctl play-pause"), { locked = true, description = "Play / pause" })
bind("XF86AudioPrev", hl.dsp.exec_cmd("playerctl previous"), { locked = true, description = "Previous track" })

-- Toggle the LG ultrawide between SDR and HDR (see ~/.local/bin/hdr-toggle.sh).
-- SDR for desktop work (brighter overall), HDR for native-HDR games.
bind(mainMod .. " + SHIFT + H", hl.dsp.exec_cmd("~/.local/bin/hdr-toggle.sh"), { description = "Toggle HDR" })
