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

# Stricter check used by hyprsunset-if-not-gaming.sh: only real game windows
# count. No fullscreen heuristic here -- a maximized terminal or fullscreen
# YouTube must not kill the night light. To add a game, find its class with:
#   hyprctl activewindow -j | jq -r '.class'
GAME_CLASS_REGEX='^(steam_app_[0-9]+|Star Citizen|Dofus\.x64|retroarch)$'

is_playing_game() {
    class=$(hyprctl activewindow -j 2>/dev/null | jq -r '.class' 2>/dev/null)
    [[ "$class" =~ $GAME_CLASS_REGEX ]]
}

is_gaming
