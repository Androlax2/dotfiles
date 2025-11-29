#!/bin/bash

# ---- CONFIG ----
COMMAND="hyprlock"
THRESHOLD=10   # pixels from the corner
DEBOUNCE=2     # seconds between triggers
# ----------------

HEIGHT=$(hyprctl -j monitors | jq '.[0].height')

while true; do
    # Check if a fullscreen Steam game is running
    activewindow=$(hyprctl -j activewindow)
    class=$(echo "$activewindow" | jq -r '.class')
    fullscreen=$(echo "$activewindow" | jq -r '.fullscreen')
    
    # Skip hot corner if Steam or Steam game is fullscreen
    # fullscreen can be: 0 (no), 1 (maximize), 2 (fullscreen)
    if [[ "$fullscreen" -ge 1 ]] && [[ "$class" =~ ^(steam|steam_app_) ]]; then
        sleep 0.5
        continue
    fi
    
    pos=$(hyprctl -j cursorpos)
    x=$(echo "$pos" | jq '.x')
    y=$(echo "$pos" | jq '.y')

    # bottom-left corner detection
    if [ "$x" -lt "$THRESHOLD" ] && [ "$y" -gt "$(($HEIGHT - $THRESHOLD))" ]; then
        eval "$COMMAND"
        sleep $DEBOUNCE
    fi

    sleep 0.05
done

