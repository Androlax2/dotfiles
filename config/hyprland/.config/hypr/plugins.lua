-- Plugins are installed and loaded by hyprpm ("hyprpm reload -n" in autostart.lua). Loading a plugin
-- re-runs the config, so each block below is guarded and applies once its plugin is in.

-- hypr-dynamic-cursors, used only for macOS-style "shake to find". The motion modes stay off
-- (mode = none). Tuned to behave like macOS: detect on the second or third swing (threshold), start at 2x (base), grow while
-- the shaking continues (speed, plus influence for harder shakes), cap at 4x (limit) and let go
-- the instant the shaking stops (timeout = 0). The 400 ms ease in both directions is hard-coded
-- in the plugin.
if hl.plugin.dynamic_cursors then
    hl.config({
        plugin = {
            dynamic_cursors = {
                enabled = true,
                mode = "none",
                shake = {
                    enabled = true,
                    threshold = 3.5,
                    base = 2.0,
                    speed = 5.0,
                    influence = 0.5,
                    limit = 4.0,
                    timeout = 0,
                    effects = false,
                    ipc = false,
                },
            },
        },
    })
end
