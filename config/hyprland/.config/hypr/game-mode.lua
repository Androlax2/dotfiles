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

-- `hyprctl eval "ENTER_GAME_MODE()"` from scripts/game-mode-prompt.sh.
ENTER_GAME_MODE = game_mode.enter

-- Same classes as GAME_CLASS_REGEX in scripts/is-gaming.sh.
local game_classes = { "^steam_app_%d+$", "^Star Citizen$", "^starcitizen%.exe$", "^Dofus%.x64$", "^retroarch$" }

hl.on("window.open", function(window)
    if hl.get_current_submap() == "gamemode" then
        return
    end
    for _, pattern in ipairs(game_classes) do
        if window.class:match(pattern) then
            hl.dispatch(hl.dsp.exec_cmd("~/.config/hypr/scripts/game-mode-prompt.sh '" .. window.class .. "'"))
            return
        end
    end
end)

return game_mode
