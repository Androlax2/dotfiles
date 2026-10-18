-- Scratchpads on native special workspaces, toggled straight from the binds (no
-- shell round trip): the first toggle spawns the window with exec rules that put
-- it on its special workspace silently, so the current workspace never changes.
--   term      dropdown kitty, 60 % x 45 %, top-centred 6 px under the bar (32 + 6 + 6)
--   music     Apple Music, 1400x900 centred
--   prs       GitHub pull requests waiting for my review, 1600x1000 centred
--   claude    Claude Code on the web (claude.ai/code), 1600x1000 centred
--   greyproxy greyproxy's dashboard: pending requests to approve, 1400x900 centred
-- Each web scratchpad is a site in its own Zen profile (compact mode, its own logins),
-- recognised by its command line and tagged with its name. A cold start takes seconds, so
-- a spinner placeholder window opens first and is closed once the Zen window is there.
-- The last three belong to the Claude cockpit: their keys only act on workspace 6, and the
-- session manager opens them too (OPEN_IN_SCRATCHPAD below), a pull request straight in prs.

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
}

-- greyproxy's dashboard; the Makefile's GREYPROXY_DASHBOARD_PORT (tailnet publishing) is the same port.
local GREYPROXY_DASHBOARD = "http://localhost:43080/dashboard"
local CLAUDE_WORKSPACE = 6 -- apps/claude.lua (WORKSPACE) and waybar/scripts/on-claude-workspace.sh carry the same number

local function web_scratchpad(name, title, url, width, height, cockpit)
    scratchpads[name] = {
        size = { width, height },
        home = url,
        cockpit = cockpit,
        present = function(window) return has_tag(window, name) end,
        placeholder = function(window) return window.class == "dotfiles." .. name .. "-placeholder" end,
        spawn = function(page)
            local rules = { workspace = "special:" .. name .. " silent", float = true, center = true, size = width .. " " .. height }
            hl.exec_cmd(string.format("~/.local/bin/web-placeholder %s %q", name, title), rules)
            hl.exec_cmd(string.format("~/.config/hypr/scripts/web-launch.sh %s %q", name, page or url), rules)
        end,
    }
end

web_scratchpad("music", "Apple Music", "https://music.apple.com", 1400, 900, false)
web_scratchpad("prs", "Pull requests", "https://github.com/pulls?q=is%3Aopen+is%3Apr+review-requested%3A%40me", 1600, 1000, true)
web_scratchpad("claude", "Claude", "https://claude.ai/code", 1600, 1000, true)
web_scratchpad("greyproxy", "greyproxy", GREYPROXY_DASHBOARD, 1400, 900, true)

local function on_claude_workspace()
    local workspace = hl.get_active_workspace()
    return workspace ~= nil and workspace.id == CLAUDE_WORKSPACE
end

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
    if scratchpad.cockpit and not on_claude_workspace() then
        return -- a cockpit key outside the cockpit does nothing
    end
    if has_window(scratchpad.present) then
        hl.dispatch(hl.dsp.workspace.toggle_special(name))
        return
    end
    if not pending[name] then
        pending[name] = true
        scratchpad.spawn()
    end
end

local function special_shown(name)
    local monitor = hl.get_active_monitor()
    local shown = monitor and monitor.active_special_workspace
    return shown ~= nil and shown.name == "special:" .. name
end

-- Show a web scratchpad, on a page when one is given: a running profile takes the page
-- through Zen's remoting (web-launch.sh starts profiles without --no-remote for this), a
-- profile not yet running starts on it. For the session manager: Enter on a pull request.
local function open_in(name, page)
    local scratchpad = scratchpads[name]
    if has_window(scratchpad.present) then
        if page then
            hl.exec_cmd(string.format("zen-browser -P %s %q", name, page))
        end
        if not special_shown(name) then
            hl.dispatch(hl.dsp.workspace.toggle_special(name))
        end
        return
    end
    if not pending[name] then
        pending[name] = true
        scratchpad.spawn(page)
    end
end

-- `hyprctl eval 'TOGGLE_SCRATCHPAD("music")'` for scripts (tools/capture.sh);
-- `hyprctl eval 'OPEN_IN_SCRATCHPAD("prs", "https://github.com/…/pull/7")'` for claude-sessions-sidebar.
TOGGLE_SCRATCHPAD = toggle
OPEN_IN_SCRATCHPAD = open_in

-- keyd swaps Ctrl and Super, so CTRL here is the physical Super/Cmd key; code:49 is
-- the key left of 1 (² on AZERTY, ` on QWERTY).
local bind = require("binds").bind
bind("CTRL + code:49", function() toggle("term") end, { description = "Dropdown terminal" })
bind("CTRL + M", function() toggle("music") end, { description = "Apple Music" })
-- SUPER + CTRL is physical Ctrl + Cmd, like the walker pickers in bindings.lua. These three
-- are cockpit keys: they act on the Claude workspace only (the key is still swallowed elsewhere).
-- Pull requests: the list of the repository of the session in the cockpit's pane (claude-sessions prs).
bind("SUPER + CTRL + P", hl.dsp.exec_cmd("claude-sessions prs"), { description = "Pull requests of the viewed session's repository" })
bind("SUPER + CTRL + A", function() toggle("claude") end, { description = "Claude on the web" })
bind("SUPER + CTRL + G", function() toggle("greyproxy") end, { description = "greyproxy dashboard" })

-- Escape hides a cockpit scratchpad when its window has the focus. The bind does not consume
-- the key (Escape is every application's key, Hyprland must not keep it), so it is checked
-- against the focused window and does nothing anywhere else.
local function cockpit_scratchpad_of(window)
    if window == nil then
        return nil
    end
    for name, scratchpad in pairs(scratchpads) do
        if scratchpad.cockpit and has_tag(window, name) then
            return name
        end
    end
    return nil
end

hl.bind("Escape", function()
    local name = cockpit_scratchpad_of(hl.get_active_window())
    if name and special_shown(name) then
        hl.dispatch(hl.dsp.workspace.toggle_special(name))
    end
end, { non_consuming = true })

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

-- A cockpit scratchpad belongs to the Claude workspace: leaving it hides the window (the key
-- brings it back there). The dropdown terminal and Apple Music follow you as before.
hl.on("workspace.active", function(workspace)
    if workspace ~= nil and workspace.id == CLAUDE_WORKSPACE then
        return
    end
    for name, scratchpad in pairs(scratchpads) do
        if scratchpad.cockpit and special_shown(name) then
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
