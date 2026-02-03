#!/bin/bash

# Configuration
HOURS_AHEAD=2

# Get next event with end time to calculate refresh interval
next_event=$(khal list now ${HOURS_AHEAD}h --format "{start-time} {title}" 2>/dev/null | grep -v "^error" | grep -v "^Today" | grep -v "^$" | head -1)
next_event_end=$(khal list now ${HOURS_AHEAD}h --format "{end}" 2>/dev/null | grep -v "^error" | grep -v "^Today" | grep -v "^$" | head -1)

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

# Signal waybar to refresh when event ends
# Calculate seconds until next event ends and schedule a refresh
if [[ -n "$next_event_end" ]]; then
    # Parse the end time and calculate seconds remaining
    end_epoch=$(date -d "$next_event_end" +%s 2>/dev/null)
    now_epoch=$(date +%s)
    if [[ -n "$end_epoch" && "$end_epoch" -gt "$now_epoch" ]]; then
        seconds_until_end=$((end_epoch - now_epoch + 5))  # Add 5s buffer
        # Schedule a signal to waybar (in background, detached)
        (sleep "$seconds_until_end" && pkill -RTMIN+9 waybar) &>/dev/null &
        disown
    fi
fi
