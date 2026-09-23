#!/usr/bin/env bash
# Take the README screenshots into assets/: a floating terminal over the wallpaper,
# three tiled terminals, the bar, the launcher, a notification stack, two tiled
# windows, the music scratchpad and the bar with the REC chip.
# Runs on an empty persistent workspace (1-5) so nothing else shows, then returns.
set -uo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
out=$repo/assets
mkdir -p "$out"

monitor=$(hyprctl monitors -j | jq -r '.[] | select(.focused)')
width=$(jq -r .width <<<"$monitor")
height=$(jq -r .height <<<"$monitor")
previous_workspace=$(hyprctl activeworkspace -j | jq -r .id)
focus() { hyprctl dispatch "hl.dsp.focus({ workspace = $1 })" >/dev/null; }
shot() { grim -g "$1" "$out/$2.png" && echo "assets/$2.png"; }
# Full-screen shots at half size: the README does not need 3840 px wide images.
shot_screen() { grim -s 0.5 "$out/$1.png" && echo "assets/$1.png"; }
spawn() {
    setsid "$@" >/dev/null 2>&1 </dev/null &
    sleep 1.2
}
# Floating, centred, through Hyprland's spawn rules.
spawn_floating() {
    hyprctl eval "hl.exec_cmd([[$1]], { float = true, center = true, size = \"$2\" })" >/dev/null
    sleep 1.5
}
close_scratch_windows() {
    hyprctl clients -j | jq -r --argjson ws "$scratch" '.[] | select(.workspace.id == $ws) | .pid' | xargs -r kill 2>/dev/null
    sleep 0.6
}
cleanup() {
    close_scratch_windows
    focus "$previous_workspace"
}
trap cleanup EXIT

scratch=$(hyprctl workspaces -j | jq -r '[.[] | select(.id > 0 and .id <= 5 and .windows > 0) | .id] as $busy | [range(1; 6)] - $busy | .[0] // 5')
focus "$scratch"
sleep 0.5

# 0. A floating terminal with fastfetch over the wallpaper
spawn_floating "kitty --hold -e fastfetch" "1500 820"
sleep 1
shot_screen desktop
close_scratch_windows

# 0b. Tiled terminals: fish + fastfetch on the left, btop top right, yazi bottom right.
# Dwindle would put the third window beside btop (the right half is wider than tall),
# so its split is toggled to stack them.
# fastfetch queries the terminal and the replies arrive after it exits, so they are
# flushed before fish starts, or fish would echo them as "\033\033" at the top.
spawn kitty -e sh -c 'fastfetch; sleep 0.3; python3 -c "import sys, termios; termios.tcflush(sys.stdin, termios.TCIFLUSH)"; exec fish'
spawn kitty -e btop
spawn kitty -e yazi ~/dotfiles
hyprctl dispatch "hl.dsp.layout('togglesplit')" >/dev/null
sleep 2.5
shot_screen terminals
close_scratch_windows

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
