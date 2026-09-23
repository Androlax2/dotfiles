#!/usr/bin/env bash
# Bluetooth picker in walker's dmenu mode: paired devices, connected ones marked;
# choosing a device toggles its connection.
set -uo pipefail

notify() { notify-send -a "Bluetooth" -i bluetooth "$@"; }
strip_colours() { sed 's/\x1b\[[0-9;]*m//g'; }

mapfile -t paired < <(bluetoothctl devices Paired | strip_colours | sed -E 's/^Device ([0-9A-F:]+) (.*)$/\1|\2/')
if ((${#paired[@]} == 0)); then
    notify "No paired devices" "Pair one with blueman-manager first"
    exit 1
fi
connected=$(bluetoothctl devices Connected | strip_colours | awk '{print $2}')

lines=()
for device in "${paired[@]}"; do
    IFS='|' read -r mac name <<<"$device"
    marker="  "
    grep -qxF "$mac" <<<"$connected" && marker="󰄬 "
    lines+=("$marker$name")
done

choice=$(printf '%s\n' "${lines[@]}" | walker -d -p "Bluetooth") || exit 0
[[ -z $choice ]] && exit 0
name=${choice#󰄬 }
name=${name#  }
mac=$(printf '%s\n' "${paired[@]}" | awk -F'|' -v name="$name" '$2 == name {print $1; exit}')
[[ -z $mac ]] && exit 1

if grep -qxF "$mac" <<<"$connected"; then
    bluetoothctl disconnect "$mac" >/dev/null 2>&1 && notify "Disconnected $name" || notify -u critical "Could not disconnect $name"
else
    bluetoothctl connect "$mac" >/dev/null 2>&1 && notify "Connected $name" || notify -u critical "Could not connect $name"
fi
