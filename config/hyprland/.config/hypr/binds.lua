-- hl.bind wrapper that remembers each described bind, so the keybinding cheat
-- sheet (walker/scripts/keybinds.sh) can run the chosen entry:
--   hyprctl eval 'RUN_BIND("Power menu")'
BIND_ACTIONS = BIND_ACTIONS or {}

local binds = {}

function binds.bind(keys, action, options)
    options = options or {}
    if options.description then
        BIND_ACTIONS[options.description] = action
    end
    hl.bind(keys, action, options)
end

function RUN_BIND(description)
    local action = BIND_ACTIONS[description]
    if action == nil then
        hl.notification.create({ text = "No bind described as " .. description, time = 3000 })
    elseif type(action) == "function" then
        action()
    else
        hl.dispatch(action)
    end
end

return binds
