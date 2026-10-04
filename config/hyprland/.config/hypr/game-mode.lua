-- Game mode: eye candy off and the "gamemode" submap on. Entered with ALT + G
-- (bindings.lua) or from the prompt shown when a game window opens; the reload
-- on exit restores the config values changed here.
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

-- `hyprctl eval "ENTER_GAME_MODE()"` from scripts/game-mode-prompt.sh.
ENTER_GAME_MODE = game_mode.enter

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
    if hl.get_current_submap() == "gamemode" or matches_any(window.title or "", launcher_titles) then
        return
    end
    if matches_any(window.class or "", game_classes) then
        hl.dispatch(hl.dsp.exec_cmd("~/.config/hypr/scripts/game-mode-prompt.sh '" .. window.class .. "'"))
    end
end)

return game_mode
