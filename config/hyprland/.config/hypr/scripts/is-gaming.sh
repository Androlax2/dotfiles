#!/bin/bash
# Save as: ~/.config/hypr/scripts/is-gaming.sh
# Common function to check if gaming

is_gaming() {
    active_window=$(hyprctl activewindow -j 2>/dev/null)
    
    FULLSCREEN=$(echo "$active_window" | jq -r '.fullscreen' 2>/dev/null)
    CLASS=$(echo "$active_window" | jq -r '.class' 2>/dev/null)
    
    # Check fullscreen
    if [ "$FULLSCREEN" != "0" ] && [ "$FULLSCREEN" != "false" ] && [ "$FULLSCREEN" != "null" ]; then
        return 0
    fi
    
    # Check gaming apps
    if [[ "$CLASS" =~ ^(steam|steam_app_|Star Citizen|heroic|lutris|retroarch)$ ]]; then
        return 0
    fi
    
    return 1
}

is_gaming
