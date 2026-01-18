#!/bin/bash

CHANGE=$1
if [ -z "$CHANGE" ]; then exit 1; fi

# --- 1. Star Citizen / Gaming Check ---
WINDOW_DATA=$(hyprctl activewindow -j)
IS_FULLSCREEN=$(echo "$WINDOW_DATA" | jq -r '.fullscreen')

if [ "$IS_FULLSCREEN" -ne 0 ]; then
    exit 0
fi

# --- 2. Monitor Detection ---
MONITOR=$(hyprctl monitors -j | jq -r '.[] | select(.focused == true) | .name')
case $MONITOR in
    "DP-1") BUS=8 ;; 
    *) exit 1 ;;
esac

CACHE_FILE="/tmp/brightness_cache_$MONITOR"

# --- 3. Instant Brightness Logic ---
if [ -f "$CACHE_FILE" ]; then
    CURRENT=$(cat "$CACHE_FILE")
else
    CURRENT=$(ddcutil getvcp 10 --bus "$BUS" --terse --sleep-multiplier 0 | cut -d' ' -f4)
fi

# Calculate new value
NEW=$((CURRENT + CHANGE))

# Clamp 0-100
[ $NEW -lt 0 ] && NEW=0
[ $NEW -gt 100 ] && NEW=100

# Update cache immediately
echo "$NEW" > "$CACHE_FILE"

# --- 4. The Notification Fix ---
# Ensure NEW is a number before sending to notify-send
if [[ "$NEW" =~ ^[0-9]+$ ]]; then
    notify-send -h string:x-canonical-private-synchronous:brightness \
                -h int:value:"$NEW" \
                -t 1000 \
                "Brightness" "$NEW%"
fi

# Send to hardware in background
ddcutil setvcp 10 $NEW --bus "$BUS" --sleep-multiplier 0 --skip-ddc-checks &
