#!/bin/bash

SECRETS_FILE="$HOME/.secrets/airpods"
if [ ! -f "$SECRETS_FILE" ]; then
  echo "Error: $SECRETS_FILE not found" >&2
  exit 1
fi
source "$SECRETS_FILE"

# Convert colon-separated MAC to regex matching both : and _ separators
AIRPODS_RE="^bluez_output\.$(echo "$AIRPODS_MAC" | sed 's/:/(:|_)/g')"

while true; do
  if bluetoothctl info "$AIRPODS_MAC" 2>/dev/null | grep -q "Connected: yes"; then
    airpods_sink=$(pactl list short sinks 2>/dev/null | awk -v re="$AIRPODS_RE" '$2 ~ re { print $2; exit }')
    if [ -n "$airpods_sink" ]; then
      current_sink=$(pactl get-default-sink 2>/dev/null)
      if [ "$current_sink" != "$airpods_sink" ]; then
        pactl set-default-sink "$airpods_sink" >/dev/null 2>&1
        pactl list short sink-inputs 2>/dev/null | awk '{print $1}' | while read -r input_id; do
          [ -n "$input_id" ] && pactl move-sink-input "$input_id" "$airpods_sink" >/dev/null 2>&1
        done
      fi
    fi
  fi
  sleep 1
done
