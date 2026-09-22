#!/usr/bin/env bash
# Waybar image module: the notification bell. bell-off while Do Not Disturb is on, bell-dot
# while notifications wait in the centre, bell otherwise. swaync-client talks to swaync over
# D-Bus, so calls get a timeout: image scripts run synchronously at bar startup.
ICONS="$HOME/.config/waybar/icons"

if [[ "$(timeout 0.3 swaync-client -D 2>/dev/null)" == "true" ]]; then
    printf '%s/bell-off.svg\nDo not disturb\n' "$ICONS"
    exit 0
fi
count=$(timeout 0.3 swaync-client -c 2>/dev/null) || exit 0
if (( count > 0 )); then
    printf '%s/bell-dot.svg\n%s notification(s)\n' "$ICONS" "$count"
else
    printf '%s/bell.svg\nno notifications\n' "$ICONS"
fi
