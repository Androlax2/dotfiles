#!/usr/bin/env bash
# Take the README screenshots into assets/: the bar, the launcher, a notification
# stack, two tiled windows, the music scratchpad and the bar with the REC chip.
# Runs on an empty persistent workspace (1-5) so nothing else shows, then returns.
set -uo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
out=$repo/assets
mkdir -p "$out"

monitor=$(hyprctl monitors -j | jq -r '.[] | select(.focused)')
width=$(jq -r .width <<<"$monitor")
height=$(jq -r .height <<<"$monitor")
previous_workspace=$(hyprctl activeworkspace -j | jq -r .id)
spawned=()

focus() { hyprctl dispatch "hl.dsp.focus({ workspace = $1 })" >/dev/null; }
shot() { grim -g "$1" "$out/$2.png" && echo "assets/$2.png"; }
spawn() {
    setsid "$@" >/dev/null 2>&1 </dev/null &
    spawned+=($!)
    sleep 1.2
}
cleanup() {
    for pid in "${spawned[@]}"; do kill "$pid" 2>/dev/null; done
    focus "$previous_workspace"
}
trap cleanup EXIT

scratch=$(hyprctl workspaces -j | jq -r '[.[] | select(.id > 0 and .id <= 5 and .windows > 0) | .id] as $busy | [range(1; 6)] - $busy | .[0] // 5')
focus "$scratch"
sleep 0.5

# 1. The bar alone
shot "0,0 ${width}x44" bar

# 2. Two tiled windows (kitty with the palette and btop), 16:9 crop from the top left
spawn kitty --hold -e sh -c "cat ~/.config/waybar/colors.css"
spawn kitty -e btop
sleep 2
shot "0,0 $((width / 2))x$((width / 2 * 9 / 16))" layout

# 3. The launcher over them
walker &
sleep 0.8
shot "$((width / 2 - 520)),$((height / 2 - 360)) 1040x720" launcher
walker -q
sleep 0.4

# 4. Notifications, top right
notify-send -a "Files" -i io.elementary.files "Download complete" "arch-installer.iso saved to Downloads · 1.2 GB"
notify-send -a "Backup" -i drive-harddisk -h int:value:62 "Backing up home" "1.3 GB of 2.1 GB sent to the NAS"
notify-send -a "Security check" -i dialog-warning -u critical "Security check: 2 findings" "new: tcp 0.0.0.0:8080"
sleep 1.2
shot "$((width - 560)),0 560x420" notification
swaync-client -C >/dev/null 2>&1

# 5. The music scratchpad
hyprctl eval 'TOGGLE_SCRATCHPAD("music")' >/dev/null
sleep 6
shot "$((width / 2 - 760)),$((height / 2 - 500)) 1520x1000" music
hyprctl eval 'TOGGLE_SCRATCHPAD("music")' >/dev/null
sleep 0.5

# 6. The bar with a recording running (silent, whole monitor: slurp is skipped)
if command -v wl-screenrec >/dev/null; then
    tmp_recording="$XDG_RUNTIME_DIR/capture-recording.mp4"
    wl-screenrec -g "0,0 ${width}x${height}" -f "$tmp_recording" >/dev/null 2>&1 &
    recorder=$!
    printf '%s\t%s\n' "$(date +%s)" "$tmp_recording" > "$XDG_RUNTIME_DIR/recording"
    pkill -RTMIN+13 waybar
    sleep 2.5
    shot "$((width - 1300)),0 1300x44" recording
    kill -INT "$recorder"; sleep 0.8
    rm -f "$XDG_RUNTIME_DIR/recording" "$tmp_recording"
    pkill -RTMIN+13 waybar
fi
