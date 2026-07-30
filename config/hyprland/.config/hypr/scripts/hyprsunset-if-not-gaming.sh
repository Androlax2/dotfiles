#!/bin/bash
# Keeps hyprsunset off while gaming. When not gaming, the daemon runs and
# its profiles (hyprsunset.conf) decide when the tint actually applies.

source ~/.config/hypr/scripts/is-gaming.sh

while true; do
    if is_gaming; then
        pkill -x hyprsunset
    else
        pgrep -x hyprsunset >/dev/null || hyprsunset &
    fi
    sleep 10
done
