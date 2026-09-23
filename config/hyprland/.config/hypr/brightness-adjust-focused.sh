#!/bin/bash
# Change the focused monitor's brightness over DDC/CI and show it on the OSD.
# Usage: brightness-adjust-focused.sh +10 | -10 | =100
# The I2C bus is read from sysfs under the monitor's DRM connector: bus numbers
# (and DP-N names) shuffle across kernel and driver updates, so nothing is hardcoded.

change=$1
if [ -z "$change" ]; then exit 1; fi

# Skip while a fullscreen window (a game) is focused.
if [ "$(hyprctl activewindow -j | jq -r '.fullscreen')" != "0" ]; then
    exit 0
fi

connector=$(hyprctl monitors -j | jq -r '.[] | select(.focused) | .name')
bus=$(ls /sys/class/drm/card*-"$connector"/ 2>/dev/null | sed -n 's/^i2c-\([0-9]*\)$/\1/p' | head -n1)
if [ -z "$bus" ]; then
    notify-send -a "Brightness" -u critical "No DDC bus for $connector"
    exit 1
fi

# Reading over DDC is slow, so the last value is cached per boot.
cache_file="${XDG_RUNTIME_DIR:-/tmp}/brightness-$connector"
if [ -f "$cache_file" ]; then
    current=$(cat "$cache_file")
else
    current=$(ddcutil getvcp 10 --bus "$bus" --terse --sleep-multiplier 0 | cut -d' ' -f4)
fi

case $change in
=*) new=${change#=} ;;
*) new=$((current + change)) ;;
esac
[ "$new" -lt 0 ] && new=0
[ "$new" -gt 100 ] && new=100

echo "$new" > "$cache_file"
~/.local/bin/osd-show brightness "$new"
ddcutil setvcp 10 "$new" --bus "$bus" --sleep-multiplier 0 --skip-ddc-checks &
