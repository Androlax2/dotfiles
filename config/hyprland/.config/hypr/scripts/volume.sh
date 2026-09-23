#!/usr/bin/env bash
# Volume keys: change the default sink, then show the OSD and refresh the bar's icon.
# Usage: volume.sh raise|lower|mute
set -uo pipefail

case ${1:-} in
raise) wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+ ;;
lower) wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%- ;;
mute) wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle ;;
*)
    echo "usage: volume.sh raise|lower|mute" >&2
    exit 1
    ;;
esac

state=$(wpctl get-volume @DEFAULT_AUDIO_SINK@) # "Volume: 0.45" or "Volume: 0.45 [MUTED]"
level=$(awk '{printf "%d", $2 * 100 + 0.5}' <<<"$state")
if [[ $state == *MUTED* ]]; then
    ~/.local/bin/osd-show mute
else
    ~/.local/bin/osd-show volume "$level"
fi
pkill -RTMIN+8 waybar 2>/dev/null || true
