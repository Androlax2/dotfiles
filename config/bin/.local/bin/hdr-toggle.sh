#!/usr/bin/env bash
# Toggle the LG ultrawide between SDR and HDR on Hyprland by flipping the `local HDR`
# flag in ~/.config/hypr/monitors.lua, then reloading Hyprland.
#
# monitors.lua must contain exactly one line of the form:
#   local HDR = false
# (or `= true`). The monitor spec for each mode lives in that file.
#
# Why this exists: in HDR mode the LG Ultrawide 38" reserves panel
# headroom for highlights, so global brightness drops noticeably even
# at sdrbrightness 4.0. SDR mode lets the panel run at its full
# everyday peak. Switch to HDR before launching games with native HDR
# (Elden Ring with DXVK_HDR=1 etc.), back to SDR for desktop work.

set -euo pipefail

CONF="$HOME/.config/hypr/monitors.lua"

if [[ ! -f "$CONF" ]]; then
	echo "missing: $CONF" >&2
	exit 1
fi

# Detect current mode from the flag, then flip it.
if grep -qE '^local HDR = true$' "$CONF"; then
	current=HDR
	target=SDR
	sed -i 's/^local HDR = true$/local HDR = false/' "$CONF"
elif grep -qE '^local HDR = false$' "$CONF"; then
	current=SDR
	target=HDR
	sed -i 's/^local HDR = false$/local HDR = true/' "$CONF"
else
	echo "could not find a 'local HDR = true|false' line in $CONF" >&2
	exit 1
fi

hyprctl reload > /dev/null

# Give Hyprland a moment to apply the new mode before querying.
sleep 0.3

# Refresh the Waybar HDR indicator (custom/hdr module is wired to
# SIGRTMIN+10 in config.jsonc). Ignore failure if Waybar isn't running.
pkill -RTMIN+10 waybar 2>/dev/null || true

preset=$(hyprctl monitors 2>/dev/null | awk '/colorManagementPreset/ {print $2; exit}')
brightness=$(hyprctl monitors 2>/dev/null | awk '/sdrBrightness/ {print $2; exit}')

echo "was:  $current"
echo "now:  $target  (colorManagementPreset=${preset:-?}  sdrBrightness=${brightness:-?})"
