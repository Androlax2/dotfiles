#!/usr/bin/env bash
# Toggle a screen recording with wl-screenrec.
#   First run: pick an area with slurp (Esc = the focused monitor), record with the
#   default sink's audio, write the state file the bar's REC chip reads.
#   Second run: stop, optionally compress, notify (click reveals the file in Files).
# Options: --silent (no audio), --no-compress (skip the ffmpeg pass).
set -uo pipefail

silent=false
compress=true
while [[ $# -gt 0 ]]; do
    case $1 in
    --silent) silent=true ;;
    --no-compress) compress=false ;;
    *)
        echo "Usage: $0 [--silent] [--no-compress]" >&2
        exit 1
        ;;
    esac
    shift
done

save_dir="$HOME/Videos/Recordings"
state_file="${XDG_RUNTIME_DIR:-/tmp}/recording" # "<start epoch>\t<file>", read by waybar/scripts/recording.sh
mkdir -p "$save_dir"

refresh_bar() { pkill -RTMIN+13 waybar 2>/dev/null || true; }

# ── Stop if already recording ──────────────────────────────────────────────
if pgrep -x wl-screenrec >/dev/null; then
    file=$(cut -f2 "$state_file" 2>/dev/null || true)
    pkill -INT wl-screenrec
    for _ in $(seq 1 15); do
        pgrep -x wl-screenrec >/dev/null || break
        sleep 0.2
    done
    if pgrep -x wl-screenrec >/dev/null; then
        pkill -9 wl-screenrec
        sleep 0.5
    fi
    rm -f "$state_file"
    refresh_bar

    [[ -n $file && -f $file ]] || file=$(ls -t "$save_dir"/*.mp4 2>/dev/null | head -n1)
    if [[ -z $file ]]; then
        notify-send -a "Recording" -u critical "Recording failed" "No file was written"
        exit 1
    fi

    if [[ $compress == true ]] && command -v ffmpeg >/dev/null; then
        compressed="${file%.mp4}_compressed.mp4"
        if ffmpeg -i "$file" -c:v libx264 -crf 23 -preset medium -c:a copy -movflags +faststart "$compressed" -y 2>/dev/null; then
            mv "$compressed" "$file"
        else
            rm -f "$compressed"
        fi
    fi

    printf '%s' "$file" | wl-copy
    ~/.config/capture-notify.sh "$file" "Recording saved"
    exit 0
fi

# ── Start ──────────────────────────────────────────────────────────────────
# shellcheck source=slurp-colors
source ~/.config/slurp-colors # SLURP_ARGS, generated from the palette
# Esc in slurp records the whole focused monitor.
region=$(slurp "${SLURP_ARGS[@]}") ||
    region=$(hyprctl monitors -j | jq -r '.[] | select(.focused) | "\(.x),\(.y) \(.width)x\(.height)"')

file="$save_dir/$(date +'%Y-%m-%d_%H-%M-%S').mp4"

if [[ $silent == false ]]; then
    # Record what is played, not the microphone: the default sink's monitor source.
    sink_monitor="$(pactl get-default-sink).monitor"
    wl-screenrec -g "$region" --audio --audio-device "$sink_monitor" -f "$file" &
else
    wl-screenrec -g "$region" -f "$file" &
fi
sleep 0.2
if ! pgrep -x wl-screenrec >/dev/null; then
    notify-send -a "Recording" -u critical "Recording failed" "wl-screenrec did not start"
    exit 1
fi

printf '%s\t%s\n' "$(date +%s)" "$file" > "$state_file"
refresh_bar
