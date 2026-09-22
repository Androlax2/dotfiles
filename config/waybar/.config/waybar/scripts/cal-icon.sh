#!/usr/bin/env bash
# Waybar image module: the calendar icon while calendar.sh has an event to show, nothing otherwise
# (empty output hides the icon). Same khal window and filters as calendar.sh.
HOURS_AHEAD=2

next_event=$(khal list now "${HOURS_AHEAD}h" --format "{title}" 2>/dev/null \
    | grep -v -e "^error" -e "^Today" -e "^$" | head -1)

[[ -n "$next_event" ]] && echo "$HOME/.config/waybar/icons/calendar.svg"
exit 0
