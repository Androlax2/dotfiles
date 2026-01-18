#!/bin/bash

# ---- CONFIG ----
COMMAND="hyprlock"
THRESHOLD=15   # Increased slightly for easier triggering
DEBOUNCE=3     # Seconds between triggers

while true; do
    # 1. Get Active Window Data
    WINDOW_DATA=$(hyprctl activewindow -j)
    CLASS=$(echo "$WINDOW_DATA" | jq -r '.class')
    FULLSCREEN=$(echo "$WINDOW_DATA" | jq -r '.fullscreen')

    # 2. THE GAME GUARD
    # Logic: If window is fullscreen OR matches common game patterns, skip.
    # Add any other classes (like 'gamescope') to the regex below.
    if [ "$FULLSCREEN" -ne 0 ] || [[ "$CLASS" =~ ^(steam|steam_app_|Star Citizen|heroic|lutris|retroarch)$ ]]; then
        sleep 2  # Sleep longer when gaming to save CPU
        continue
    fi

    # 3. Get Cursor and Monitor Data
    CURSOR=$(hyprctl cursorpos -j)
    X=$(echo "$CURSOR" | jq -r '.x')
    Y=$(echo "$CURSOR" | jq -r '.y')

    # Get height of the monitor the cursor is currently on
    MONITOR_HEIGHT=$(hyprctl monitors -j | jq -r ".[] | select($X >= .x and $X < (.x + .width) and $Y >= .y and $Y < (.y + .height)) | .height")
    
    # Fallback to a safe height if jq fails
    MONITOR_HEIGHT=${MONITOR_HEIGHT:-1080}

    # 4. Bottom-Left Corner Detection
    if [ "$X" -lt "$THRESHOLD" ] && [ "$Y" -gt "$((MONITOR_HEIGHT - THRESHOLD))" ]; then
        eval "$COMMAND"
        
        # After locking/unlocking, wait the debounce period
        sleep "$DEBOUNCE"
    fi

    sleep 0.1
done
