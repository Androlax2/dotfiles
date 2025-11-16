#!/bin/bash

# ---- CONFIG ----
COMMAND="hyprlock"
THRESHOLD=10   # pixels from the corner
DEBOUNCE=2     # seconds between triggers
# ----------------

HEIGHT=$(hyprctl -j monitors | jq '.[0].height')

while true; do
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

