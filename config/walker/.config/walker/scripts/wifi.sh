#!/usr/bin/env bash
# Wi-Fi picker in walker's dmenu mode: lists networks, connects to the chosen one,
# asking for a password when the network is secured and not saved yet.
set -uo pipefail

notify() { notify-send -a "Wi-Fi" -i network-wireless "$@"; }

nmcli dev wifi rescan 2>/dev/null
mapfile -t networks < <(nmcli -t -f IN-USE,SSID,SIGNAL,SECURITY dev wifi list | awk -F: '$2 != "" && !seen[$2]++')
if ((${#networks[@]} == 0)); then
    notify "No networks found"
    exit 1
fi

lines=()
for network in "${networks[@]}"; do
    IFS=: read -r in_use ssid signal security <<<"$network"
    marker="  "
    [[ $in_use == "*" ]] && marker="󰄬 "
    lock=""
    [[ -n $security ]] && lock=" 󰌾"
    lines+=("$marker$ssid  ·  $signal%$lock")
done

choice=$(printf '%s\n' "${lines[@]}" | walker -d -p "Wi-Fi") || exit 0
[[ -z $choice ]] && exit 0
ssid=$(sed -E 's/^(󰄬 |  )//; s/  ·  [0-9]+%.*$//' <<<"$choice")

if nmcli -t -f NAME connection show | grep -qxF "$ssid"; then
    nmcli connection up id "$ssid" >/dev/null 2>&1 && notify "Connected to $ssid" || notify -u critical "Could not connect to $ssid"
    exit 0
fi

security=$(nmcli -t -f SSID,SECURITY dev wifi list | awk -F: -v ssid="$ssid" '$1 == ssid {print $2; exit}')
if [[ -n $security ]]; then
    password=$(walker -d -x -p "Password for $ssid" </dev/null) || exit 0
    [[ -z $password ]] && exit 0
    nmcli dev wifi connect "$ssid" password "$password" >/dev/null 2>&1
else
    nmcli dev wifi connect "$ssid" >/dev/null 2>&1
fi
if [[ $? -eq 0 ]]; then notify "Connected to $ssid"; else notify -u critical "Could not connect to $ssid"; fi
