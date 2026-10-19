-- Game mode: eye candy off and the "gamemode" submap on (banners and approval windows
-- wait meanwhile). Entered by itself when a game window opens, or with ALT + G
-- (bindings.lua); the reload on exit restores the config values changed here.
--
-- The game's process is registered with Feral GameMode (gamemoded --request=<pid>): CPU and GPU
-- in their performance modes, and game-performance pausing the background work, for as long as
-- it runs. When it exits, GameMode's end hook (game-performance off) calls EXIT_GAME_MODE.
local game_mode = {}

-- The processes registered with GameMode in this game mode. --request toggles: a game's second
-- window (the same process) would unregister it, so each process registers once.
local registered_pids = {}

function game_mode.enter()
    hl.config({
        animations = { enabled = false },
        decoration = {
            rounding = 0,
            shadow = { enabled = false },
            blur = { enabled = false },
        },
    })
    hl.dispatch(hl.dsp.submap("gamemode"))
end

function game_mode.exit()
    registered_pids = {}
    hl.dispatch(hl.dsp.exec_cmd("hyprctl reload"))
    hl.dispatch(hl.dsp.submap("reset"))
end

-- `hyprctl eval "EXIT_GAME_MODE()"` from game-performance off, when the game has exited.
EXIT_GAME_MODE = function()
    if hl.get_current_submap() == "gamemode" then
        game_mode.exit()
    end
end

-- Same classes as GAME_CLASS_REGEX in scripts/is-gaming.sh.
local game_classes = { "^steam_app_%d+$", "^Star Citizen$", "^starcitizen%.exe$", "^Dofus%.x64$", "^retroarch$" }

-- Launchers that carry a game class (Battle.net shares the steam_app_<id> class of its
-- Steam shortcut with the game, see apps/battle-net.lua): no prompt for them.
local launcher_titles = { "^Battle%.net$" }

local function matches_any(value, patterns)
    for _, pattern in ipairs(patterns) do
        if value:match(pattern) then
            return true
        end
    end
    return false
end

hl.on("window.open", function(window)
    if matches_any(window.title or "", launcher_titles) or not matches_any(window.class or "", game_classes) then
        return
    end
    -- Every game's process registers once: GameMode counts each and ends with the last. The PID
    -- is attached to the option: "gamemoded -r <pid>", with a space, is -r without one, which
    -- registers gamemoded itself and waits forever, so game mode never ended with the game.
    local pid = window.pid and math.floor(window.pid) or 0
    if pid > 0 and not registered_pids[pid] then
        registered_pids[pid] = true
        hl.dispatch(hl.dsp.exec_cmd("gamemoded --request=" .. pid))
    end
    if hl.get_current_submap() ~= "gamemode" then
        game_mode.enter()
        hl.dispatch(hl.dsp.exec_cmd("notify-send -a 'Game mode' -i input-gaming -t 4000 'Game mode on' "
            .. "'Performance CPU and GPU, background work paused until the game exits.'"))
    end
end)

return game_mode
