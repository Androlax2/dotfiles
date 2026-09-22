#!/usr/bin/env bash
# Waybar image module: prints the network icon path on line 1 and the tooltip on line 2.
# Wired wins over wifi when both are up; no output would hide the icon, so the
# disconnected state prints its own icon instead.
ICONS="$HOME/.config/waybar/icons"

devices=$(nmcli -t -f DEVICE,TYPE,STATE device 2>/dev/null)

wired=$(awk -F: '$2 == "ethernet" && $3 == "connected" { print $1; exit }' <<< "$devices")
if [[ -n "$wired" ]]; then
    ip=$(nmcli -t -f IP4.ADDRESS device show "$wired" | head -1 | cut -d: -f2)
    printf '%s/ethernet.svg\n%s  %s\n' "$ICONS" "$wired" "$ip"
    exit 0
fi

wifi=$(awk -F: '$2 == "wifi" && $3 == "connected" { print $1; exit }' <<< "$devices")
if [[ -n "$wifi" ]]; then
    IFS=: read -r ssid signal < <(nmcli -t -f ACTIVE,SSID,SIGNAL device wifi | awk -F: '$1 == "yes" { print $2 ":" $3; exit }')
    ip=$(nmcli -t -f IP4.ADDRESS device show "$wifi" | head -1 | cut -d: -f2)
    printf '%s/wifi.svg\n%s (%s%%)  %s\n' "$ICONS" "$ssid" "$signal" "$ip"
    exit 0
fi

printf '%s/wifi-off.svg\ndisconnected\n' "$ICONS"
