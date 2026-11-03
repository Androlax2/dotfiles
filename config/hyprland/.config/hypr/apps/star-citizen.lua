-- Not loaded: the old apps.conf did not source star-citizen.conf either.
-- Add require("apps/star-citizen") to windows.lua to enable.

-- Star Citizen
hl.window_rule({
    match = { class = "^(starcitizen.exe)$" },
    workspace = "6",
    opacity = "1 1",
    idle_inhibit = "always",
    no_blur = true,
    immediate = true,
})

-- RSI Launcher
hl.window_rule({ match = { class = "^(rsi launcher.exe)$" }, workspace = "6" })
