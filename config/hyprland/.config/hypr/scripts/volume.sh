#!/usr/bin/env bash
# Volume keys: change the default sink, then refresh the bar's icon.
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

pkill -x -RTMIN+8 waybar 2>/dev/null || true
