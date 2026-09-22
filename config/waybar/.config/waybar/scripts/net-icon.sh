#!/usr/bin/env bash
# Waybar image module: network icon path on line 1, tooltip on line 2. Image modules run their
# script synchronously at bar startup, so this reads link state from sysfs (instant) instead
# of asking NetworkManager, which answers slowly while it is still connecting at login.
ICONS="$HOME/.config/waybar/icons"

is_up() { [[ "$(cat "/sys/class/net/$1/operstate" 2>/dev/null)" == "up" ]]; }

for dev in /sys/class/net/en* /sys/class/net/eth*; do
    [[ -e "$dev" ]] || continue
    name=${dev##*/}
    if is_up "$name"; then
        ip=$(ip -4 -o addr show dev "$name" 2>/dev/null | awk '{print $4; exit}')
        printf '%s/ethernet.svg\n%s  %s\n' "$ICONS" "$name" "$ip"
        exit 0
    fi
done

for dev in /sys/class/net/wl*; do
    [[ -e "$dev" ]] || continue
    name=${dev##*/}
    if is_up "$name"; then
        # iw reads the link directly from the driver, no daemon involved
        ssid=$(iw dev "$name" link 2>/dev/null | awk -F': ' '/SSID/ {print $2; exit}')
        signal=$(iw dev "$name" link 2>/dev/null | awk '/signal/ {print $2; exit}')
        ip=$(ip -4 -o addr show dev "$name" 2>/dev/null | awk '{print $4; exit}')
        printf '%s/wifi.svg\n%s (%s dBm)  %s\n' "$ICONS" "${ssid:-wifi}" "${signal:-?}" "$ip"
        exit 0
    fi
done

printf '%s/wifi-off.svg\ndisconnected\n' "$ICONS"
