-- Hyprland config, split by concern. Each require() runs in its own Lua
-- scope, so an error in one file does not stop the others from loading.
-- https://wiki.hypr.land/Configuring/

----------------
--- MONITORS ---
----------------

require("monitors")

-----------------
--- AUTOSTART ---
-----------------

require("autostart")

-----------------------------
--- ENVIRONMENT VARIABLES ---
-----------------------------

require("envs")

-------------------
--- PERMISSIONS ---
-------------------

-- See https://wiki.hypr.land/Configuring/Advanced-and-Cool/Permissions/
-- Please note permission changes here require a Hyprland restart and are not
-- applied on-the-fly for security reasons

-- hl.config({ ecosystem = { enforce_permissions = true } })

-- hl.permission("/usr/(bin|local/bin)/grim", "screencopy", "allow")
-- hl.permission("/usr/(lib|libexec|lib64)/xdg-desktop-portal-hyprland", "screencopy", "allow")
-- hl.permission("/usr/(bin|local/bin)/hyprpm", "plugin", "allow")

---------------------
--- LOOK AND FEEL ---
---------------------

require("theme")
require("animations")

-- See https://wiki.hypr.land/Configuring/Layouts/Dwindle-Layout/
-- and https://wiki.hypr.land/Configuring/Layouts/Master-Layout/
hl.config({
    dwindle = {
        preserve_split = true, -- You probably want this
        force_split = 2,
    },

    master = {
        new_status = "master",
        orientation = "right",
    },

    misc = {
        force_default_wallpaper = 0,
        disable_hyprland_logo = true,
        disable_splash_rendering = true,
        focus_on_activate = true,
        anr_missed_pings = 3,
    },
})

-------------
--- INPUT ---
-------------

require("input")

-- See https://wiki.hypr.land/Configuring/Advanced-and-Cool/Gestures/
hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })

-- Example per-device config
-- See https://wiki.hypr.land/Configuring/Advanced-and-Cool/Devices/
hl.device({ name = "epic-mouse-v1", sensitivity = -0.5 })

-------------------
--- KEYBINDINGS ---
-------------------

require("bindings")

------------------------------
--- WINDOWS AND WORKSPACES ---
------------------------------

require("windows")

---------------
--- PLUGINS ---
---------------

require("plugins")
