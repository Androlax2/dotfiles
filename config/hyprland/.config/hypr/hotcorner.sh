#!/bin/bash

# ---- CONFIG ----
COMMAND="hyprlock"
THRESHOLD=15   # Increased slightly for easier triggering
DEBOUNCE=3     # Seconds between triggers

source ~/.config/hypr/scripts/is-gaming.sh

while true; do
    if is_gaming; then
        sleep 2
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
