#!/bin/bash

# Configuration - time window to show upcoming events
HOURS_AHEAD=2

# Get events in the next X hours
next_event=$(khal list now ${HOURS_AHEAD}h --format "{start-time} {title}" 2>/dev/null | grep -v "^error" | grep -v "^Today" | grep -v "^$" | head -1)

if [[ -z "$next_event" ]]; then
    # No event - output nothing (hides the module)
    echo ""
else
    # Build agenda for tooltip (next 7 days)
    agenda=$(khal list now 7d 2>/dev/null | grep -v "^error" | head -20)
    agenda_escaped=$(echo "$agenda" | sed 's/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g')
    
    # Event coming up - show it
    echo "{\"text\": \"󰃭 $next_event\", \"tooltip\": \"󰃭 Upcoming Events\\n───────────────\\n$agenda_escaped\", \"class\": \"has-events\"}"
fi
