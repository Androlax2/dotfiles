#!/bin/bash

# Get the next event's details including description/location which may contain meeting links
HOURS_AHEAD=2

# Find ics files for upcoming events (search all calendars)
calendar_dir="$HOME/.local/share/calendars"

# Get next event title to match in ics files
next_event_title=$(khal list now ${HOURS_AHEAD}h --format "{title}" 2>/dev/null | grep -v "^error" | grep -v "^Today" | grep -v "^$" | head -1)

if [[ -z "$next_event_title" ]]; then
    notify-send "Calendar" "No upcoming events" -t 2000
    exit 0
fi

# Search for meeting links in calendar files
meeting_link=""

# Look for the event in ics files by title and extract meeting link
# Use find to recursively search all calendar directories
while IFS= read -r file; do
    [[ -f "$file" ]] || continue
    
    # Check if this file contains the event title
    if grep -qi "SUMMARY:.*$(echo "$next_event_title" | sed 's/[]\[^$.*/]/\\&/g')" "$file" 2>/dev/null; then
        # Extract meeting links from the entire file (Google Meet, Zoom, Teams, etc.)
        # Check LOCATION, DESCRIPTION, and any URL fields
        meeting_link=$(grep -oE 'https?://(meet\.google\.com|zoom\.us|[a-z0-9]+\.zoom\.us|teams\.microsoft\.com|whereby\.com)/[^[:space:]"<>\\]+' "$file" 2>/dev/null | head -1)
        
        if [[ -n "$meeting_link" ]]; then
            # Clean up any trailing backslashes or escaped characters
            meeting_link=$(echo "$meeting_link" | sed 's/\\n.*//; s/\\//g')
            break
        fi
    fi
done < <(find "$calendar_dir" -name "*.ics" -type f 2>/dev/null)

if [[ -n "$meeting_link" ]]; then
    xdg-open "$meeting_link"
    notify-send "Calendar" "Opening meeting link..." -t 2000
else
    # No meeting link found, open khal interactive
    notify-send "Calendar" "No meeting link found for: $next_event_title" -t 3000
    kitty --class floating khal interactive &
fi
