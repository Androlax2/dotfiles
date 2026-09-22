#!/usr/bin/env bash
# Waybar custom module: "NIGHT" while hyprsunset is actually tinting -- daemon
# running and inside the 4000K profile window. Keep the hours in sync with
# ~/.config/hypr/hyprsunset.conf (21:00 -> 4000K, 7:30 -> identity).
pgrep -x hyprsunset >/dev/null || exit 0
now=$(date +%H%M)
if [ "$now" -ge 2100 ] || [ "$now" -lt 0730 ]; then
    echo "NIGHT"
fi
