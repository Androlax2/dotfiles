#!/bin/bash

# Configuration
HOURS_AHEAD=2

# Get events in the next X hours
next_event=$(khal list now ${HOURS_AHEAD}h --format "{start-time} {title}" 2>/dev/null | grep -v "^error" | grep -v "^Today" | grep -v "^$" | head -1)

if [[ -z "$next_event" ]]; then
    echo ""
else
    # Build nicely formatted agenda for tooltip
    tooltip="<b>󰃭  Upcoming Events</b>\\n\\n"
    
    while IFS= read -r line; do
        if [[ "$line" =~ ^(Today|Tomorrow|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday) ]]; then
            tooltip+="<span color='#7aa2f7'><b>$line</b></span>\\n"
        elif [[ -n "$line" ]]; then
            tooltip+="    $line\\n"
        fi
    done < <(khal list now 7d 2>/dev/null | grep -v "^error" | head -20)
    
    # Escape quotes for JSON
    tooltip_escaped=$(echo "$tooltip" | sed 's/"/\\"/g')
    
    echo "{\"text\": \"󰃭 $next_event\", \"tooltip\": \"$tooltip_escaped\", \"class\": \"has-events\"}"
fi
