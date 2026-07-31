#!/bin/bash
# Keeps hyprsunset off while gaming. When not gaming, the daemon runs and
# its profiles (hyprsunset.conf) decide when the tint actually applies.

# Single instance: autostart plus manual relaunches must not stack watchers.
exec 9>"$XDG_RUNTIME_DIR/hyprsunset-if-not-gaming.lock"
flock -n 9 || exit 0

source ~/.config/hypr/scripts/is-gaming.sh

while true; do
    if is_playing_game; then
        pkill -x hyprsunset
    else
        pgrep -x hyprsunset >/dev/null || hyprsunset &
    fi
    sleep 10
done
