#!/bin/bash

# Configuration - notify X minutes before events
NOTIFY_MINUTES=10

notified_file="/tmp/calendar_notified"
touch "$notified_file"

while true; do
    # Get events starting in the next NOTIFY_MINUTES minutes
    end_time=$(date -d "+${NOTIFY_MINUTES} minutes" "+%H:%M")
    
    # Get upcoming events with their start time
    while IFS= read -r event; do
        [[ -z "$event" ]] && continue
        [[ "$event" =~ ^error ]] && continue
        [[ "$event" =~ ^Today ]] && continue
        
        # Create unique ID for this event (time + title hash)
        event_id=$(echo "$event" | md5sum | cut -d' ' -f1)
        
        # Check if we already notified for this event
        if ! grep -q "$event_id" "$notified_file" 2>/dev/null; then
            # Extract time and title
            event_time=$(echo "$event" | awk '{print $1}')
            event_title=$(echo "$event" | cut -d' ' -f2-)
            
            # Send notification
            notify-send "󰃭 Upcoming Event" "$event_time - $event_title" -u normal -t 30000
            
            # Mark as notified
            echo "$event_id" >> "$notified_file"
        fi
    done < <(khal list now ${NOTIFY_MINUTES}m --format "{start-time} {title}" 2>/dev/null)
    
    # Clean old notifications (keep file small)
    tail -100 "$notified_file" > "${notified_file}.tmp" 2>/dev/null && mv "${notified_file}.tmp" "$notified_file"
    
    # Check every minute
    sleep 60
done
