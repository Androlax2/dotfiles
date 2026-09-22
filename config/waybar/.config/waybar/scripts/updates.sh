#!/usr/bin/env bash
# Waybar custom module: pending package updates. Reads the "days:count" cache that
# update-check.timer refreshes hourly (~/.config/update-timer-task.sh); empty text hides
# the module when nothing is pending. Class "stale" once the last upgrade is a week old.
STALE_DAYS=7
cache="/tmp/pending_updates_count"

[[ -f "$cache" ]] || { echo '{"text": ""}'; exit 0; }
IFS=: read -r days count < "$cache"
(( count > 0 )) || { echo '{"text": ""}'; exit 0; }

class=""
(( days >= STALE_DAYS )) && class="stale"
echo "{\"text\": \"${count}\", \"tooltip\": \"${count} package updates pending\\nLast upgrade ${days} days ago\\nClick to upgrade\", \"class\": \"${class}\"}"
