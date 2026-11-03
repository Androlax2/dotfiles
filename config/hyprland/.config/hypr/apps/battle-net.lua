-- Battle.net and World of Warcraft, launched through Steam (NonSteamLaunchers). Both windows
-- carry the shortcut's steam_app_<id> class, and the id changes when the shortcut is
-- recreated, so the title tells them apart. Game mode stays on the usual prompt
-- (game-mode.lua); Battle.net itself is excluded from it there.
local STEAM_APP = "^(steam_app_[0-9]+)$"

-- The launcher stays with Steam.
hl.window_rule({ match = { class = STEAM_APP, title = "^(Battle\\.net)$" }, workspace = "5", opacity = "1 1" })

-- The game gets the workspace Star Citizen uses and every gaming rule at once.
hl.window_rule({
    match = { class = STEAM_APP, title = "^(World of Warcraft)$" },
    workspace = "6",
    fullscreen = true,
    opacity = "1 1",
    no_blur = true,
    immediate = true,
    idle_inhibit = "always",
})

-- One World of Warcraft at a time. Battle.net starts another instance on every Play click,
-- and all instances share WTF/Config.wtf: whichever exits last writes back the settings it
-- started with, undoing what was changed in the one actually played (Sep 2026). Kill rather
-- than close: a graceful close would write the file too.
local function is_wow_window(window)
    return (window.class or ""):match("^steam_app_") ~= nil and (window.title or ""):match("^World of Warcraft") ~= nil
end

hl.on("window.open", function(window)
    if not is_wow_window(window) then
        return
    end
    for _, other in ipairs(hl.get_windows()) do
        if other.pid ~= window.pid and is_wow_window(other) then
            hl.dispatch(hl.dsp.window.kill({ window = window }))
            return
        end
    end
end)
