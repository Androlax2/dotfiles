#!/bin/bash

# Get the next event's details including description/location which may contain meeting links
HOURS_AHEAD=2

# Find ics files for upcoming events
calendar_dir="$HOME/.local/share/calendars/icloud"

# Get next event info
next_event_info=$(khal list now ${HOURS_AHEAD}h --format "{uid}" 2>/dev/null | grep -v "^error" | grep -v "^Today" | grep -v "^$" | head -1)

if [[ -z "$next_event_info" ]]; then
    notify-send "Calendar" "No upcoming events" -t 2000
    exit 0
fi

# Search for meeting links in calendar files
meeting_link=""

# Look for common meeting URLs in ics files
for file in "$calendar_dir"/*/*.ics; do
    if grep -q "$next_event_info" "$file" 2>/dev/null; then
        # Extract meeting links (Google Meet, Zoom, Teams, etc.)
        meeting_link=$(grep -oP '(https?://(meet\.google\.com|zoom\.us|teams\.microsoft\.com|whereby\.com)[^\s"<>]+)' "$file" 2>/dev/null | head -1)
        if [[ -n "$meeting_link" ]]; then
            break
        fi
    fi
done

if [[ -n "$meeting_link" ]]; then
    xdg-open "$meeting_link"
    notify-send "Calendar" "Opening meeting link..." -t 2000
else
    # No meeting link found, open khal interactive
    khal interactive
fi
