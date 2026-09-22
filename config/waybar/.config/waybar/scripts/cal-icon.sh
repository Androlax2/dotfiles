#!/usr/bin/env bash
# Waybar image module: the calendar icon while calendar.sh has an event to show. It reads the
# marker calendar.sh writes rather than asking khal itself: image scripts run synchronously at
# bar startup and khal takes ~0.2 s. calendar.sh signals SIGRTMIN+11 when the marker changes.
marker="${XDG_RUNTIME_DIR:-/tmp}/waybar-next-event"
[[ -s "$marker" ]] && echo "$HOME/.config/waybar/icons/calendar.svg"
exit 0
