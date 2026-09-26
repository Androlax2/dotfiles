-- Scratchpads on native special workspaces, toggled straight from the binds (no
-- shell round trip): the first toggle spawns the window with exec rules that put
-- it on its special workspace silently, so the current workspace never changes.
--   term   dropdown kitty, 60 % x 45 %, top-centred 6 px under the bar (32 + 6 + 6)
--   music  Apple Music in its own Zen profile ("music", compact mode), 1400x900 centred.
--          A cold start takes seconds, so a spinner placeholder window opens first and is
--          closed once the Zen window is there.

local function has_window(predicate)
    for _, window in ipairs(hl.get_windows()) do
        if predicate(window) then
            return true
        end
    end
    return false
end

local function has_tag(window, wanted)
    for _, tag in ipairs(window.tags or {}) do
        if tag == wanted then
            return true
        end
    end
    return false
end

local scratchpads = {
    term = {
        present = function(window) return window.class == "kitty-scratch" end,
        spawn = function()
            hl.exec_cmd("kitty --class kitty-scratch", {
                workspace = "special:term silent",
                float = true,
                size = "(monitor_w*0.6) (monitor_h*0.45)",
                move = "(monitor_w*0.2) 44",
            })
        end,
    },
    music = {
        present = function(window) return has_tag(window, "music") end,
        placeholder = function(window) return window.class == "dotfiles.music-placeholder" end,
        spawn = function()
            local rules = { workspace = "special:music silent", float = true, center = true, size = "1400 900" }
            hl.exec_cmd("~/.config/hypr/scripts/music-placeholder.py", rules)
            hl.exec_cmd("~/.config/hypr/scripts/music-launch.sh", rules)
        end,
    },
}

local function close_windows(predicate)
    for _, window in ipairs(hl.get_windows()) do
        if predicate(window) then
            hl.dispatch(hl.dsp.window.close({ window = window }))
        end
    end
end

-- An empty special workspace is closed by Hyprland before the window maps, so the
-- first toggle only spawns; the special is shown from the window.open event below, by
-- the scratchpad's window or by its placeholder, whichever opens first.
local pending = {}

local function toggle(name)
    local scratchpad = scratchpads[name]
    if has_window(scratchpad.present) then
        hl.dispatch(hl.dsp.workspace.toggle_special(name))
        return
    end
    if not pending[name] then
        pending[name] = true
        scratchpad.spawn()
    end
end

-- `hyprctl eval 'TOGGLE_SCRATCHPAD("music")'` for scripts (tools/capture.sh).
TOGGLE_SCRATCHPAD = toggle

-- keyd swaps Ctrl and Super, so CTRL here is the physical Super/Cmd key; code:49 is
-- the key left of 1 (² on AZERTY, ` on QWERTY).
local bind = require("binds").bind
bind("CTRL + code:49", function() toggle("term") end, { description = "Dropdown terminal" })
bind("CTRL + M", function() toggle("music") end, { description = "Apple Music" })

-- Zen ignores --class and --name on Wayland, so the music window is recognised by
-- its command line and tagged; the tag is what `present` and the rules below use.
local function is_music_window(window)
    if window.class ~= "zen" then
        return false
    end
    local cmdline = io.open("/proc/" .. window.pid .. "/cmdline", "r")
    if not cmdline then
        return false
    end
    local args = cmdline:read("a")
    cmdline:close()
    return args:find("-P\0music\0", 1, true) ~= nil
end

hl.on("window.open", function(window)
    if is_music_window(window) then
        hl.dispatch(hl.dsp.window.tag({ window = window, tag = "+music" }))
        close_windows(scratchpads.music.placeholder)
    end
    for name, scratchpad in pairs(scratchpads) do
        local opened = scratchpad.present(window) or (scratchpad.placeholder ~= nil and scratchpad.placeholder(window))
        if pending[name] and opened then
            pending[name] = nil
            hl.dispatch(hl.dsp.workspace.toggle_special(name))
        end
    end
end)

-- The music window keeps its frame even if the page asks for fullscreen.
hl.on("window.fullscreen", function(window)
    if window.fullscreen ~= 0 and has_tag(window, "music") then
        hl.dispatch(hl.dsp.window.fullscreen_state({ window = window, internal = 0, client = 0 }))
        hl.dispatch(hl.dsp.window.resize({ window = window, x = 1400, y = 900, exact = true }))
        hl.dispatch(hl.dsp.window.center({ window = window }))
    end
end)

hl.window_rule({ match = { tag = "music" }, opacity = "1 1" })
hl.window_rule({ match = { class = "^(dotfiles\\.music-placeholder)$" }, opacity = "1 1" })
