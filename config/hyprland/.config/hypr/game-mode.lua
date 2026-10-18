-- Game mode: eye candy off and the "gamemode" submap on (banners and approval windows
-- wait meanwhile). Entered by itself when a game window opens, or with ALT + G
-- (bindings.lua); the reload on exit restores the config values changed here.
--
-- The game's process is registered with Feral GameMode (gamemoded -r <pid>): CPU and GPU in
-- their performance modes, and game-performance pausing the background work, for as long as it
-- runs. When it exits, GameMode's end hook (game-performance off) calls EXIT_GAME_MODE.
local game_mode = {}

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
    -- Every game window registers its process: GameMode counts each and ends with the last.
    if window.pid and window.pid > 0 then
        hl.dispatch(hl.dsp.exec_cmd("gamemoded -r " .. math.floor(window.pid)))
    end
    if hl.get_current_submap() ~= "gamemode" then
        game_mode.enter()
        hl.dispatch(hl.dsp.exec_cmd("notify-send -a 'Game mode' -i input-gaming -t 4000 'Game mode on' "
            .. "'Performance CPU and GPU, background work paused until the game exits.'"))
    end
end)

return game_mode
