-- Scratchpads on native special workspaces, toggled straight from the binds (no
-- shell round trip): the first toggle spawns the window with exec rules that put
-- it on its special workspace silently, so the current workspace never changes.
--   term      dropdown kitty, 60 % x 45 %, top-centred 6 px under the bar (32 + 6 + 6)
--   music     Apple Music, 1400x900 centred
--   vitals    the machine monitor's TUI (vitals, its own repository), 1400x900 centred; also the bar's warning dot
-- Each web scratchpad is a site in its own Zen profile (compact mode, its own logins),
-- recognised by its command line and tagged with its name. A cold start takes seconds, so
-- a spinner placeholder window opens first and is closed once the Zen window is there.

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
    vitals = {
        present = function(window) return window.class == "vitals" end,
        spawn = function()
            hl.exec_cmd("kitty --class vitals -T vitals ~/.local/bin/vitals", {
                workspace = "special:vitals silent",
                float = true,
                center = true,
                size = "1400 900",
            })
        end,
    },
}

local function web_scratchpad(name, title, url, width, height)
    scratchpads[name] = {
        size = { width, height },
        home = url,
        present = function(window) return has_tag(window, name) end,
        placeholder = function(window) return window.class == "dotfiles." .. name .. "-placeholder" end,
        spawn = function()
            local rules = { workspace = "special:" .. name .. " silent", float = true, center = true, size = width .. " " .. height }
            hl.exec_cmd(string.format("~/.local/bin/web-placeholder %s %q", name, title), rules)
            hl.exec_cmd(string.format("~/.config/hypr/scripts/web-launch.sh %s %q", name, url), rules)
        end,
    }
end

web_scratchpad("music", "Apple Music", "https://music.apple.com", 1400, 900)

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
bind("SUPER + CTRL + M", function() toggle("vitals") end, { description = "Machine monitor (vitals)" })

-- Zen ignores --class and --name on Wayland, so a web scratchpad's window is recognised by
-- the profile on its command line and tagged with the scratchpad's name; the tag is what
-- `present` and the rules below use.
local function zen_profile_of(window)
    if window.class ~= "zen" then
        return nil
    end
    local cmdline = io.open("/proc/" .. window.pid .. "/cmdline", "r")
    if not cmdline then
        return nil
    end
    local args = cmdline:read("a")
    cmdline:close()
    for name, scratchpad in pairs(scratchpads) do
        if scratchpad.size and args:find("-P\0" .. name .. "\0", 1, true) then
            return name
        end
    end
    return nil
end

hl.on("window.open", function(window)
    local profile = zen_profile_of(window)
    if profile then
        hl.dispatch(hl.dsp.window.tag({ window = window, tag = "+" .. profile }))
        close_windows(scratchpads[profile].placeholder)
    end
    for name, scratchpad in pairs(scratchpads) do
        local opened = scratchpad.present(window) or (scratchpad.placeholder ~= nil and scratchpad.placeholder(window))
        if pending[name] and opened then
            pending[name] = nil
            hl.dispatch(hl.dsp.workspace.toggle_special(name))
        end
    end
end)

-- A web scratchpad keeps its frame even if the page asks for fullscreen.
hl.on("window.fullscreen", function(window)
    if window.fullscreen == 0 then
        return
    end
    for name, scratchpad in pairs(scratchpads) do
        if scratchpad.size and has_tag(window, name) then
            hl.dispatch(hl.dsp.window.fullscreen_state({ window = window, internal = 0, client = 0 }))
            hl.dispatch(hl.dsp.window.resize({ window = window, x = scratchpad.size[1], y = scratchpad.size[2], exact = true }))
            hl.dispatch(hl.dsp.window.center({ window = window }))
        end
    end
end)

for name, scratchpad in pairs(scratchpads) do
    if scratchpad.size then
        hl.window_rule({ match = { tag = name }, opacity = "1 1" })
    end
end
hl.window_rule({ match = { class = "^(dotfiles\\.[a-z]+-placeholder)$" }, opacity = "1 1" })
-- The machine monitor is read, not glanced at: no see-through background behind its numbers.
hl.window_rule({ match = { class = "^(vitals)$" }, opacity = "1 1" })
