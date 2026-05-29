#!/usr/bin/env bash
# Waybar custom module: print "HDR" when DP-1 is in HDR mode, nothing
# otherwise. Refresh is signal-driven (SIGRTMIN+10), triggered by
# ~/.local/bin/hdr-toggle.sh after it flips the monitor preset.
preset=$(hyprctl monitors 2>/dev/null | awk '/colorManagementPreset/ {print $2; exit}')
[[ "$preset" == "hdr" ]] && echo "HDR"
exit 0
