#!/usr/bin/env bash
# macOS-style screenshots. Usage: screenshot.sh full|area
#   full  the focused monitor
#   area  slurp selection; clicking a window captures exactly that window (its
#         geometry is fed to slurp), dragging selects freely.
# Saved to ~/Pictures/Screenshots/<datetime>.png and copied to the clipboard. The
# notification (capture-notify.sh) reveals the file in Files on click and offers Annotate.
set -uo pipefail

mode=${1:?usage: screenshot.sh full|area}
save_dir="$HOME/Pictures/Screenshots"
mkdir -p "$save_dir"
file="$save_dir/$(date +'%Y-%m-%d_%H-%M-%S').png"

# shellcheck source=slurp-colors
source ~/.config/slurp-colors # SLURP_ARGS, generated from the palette

case $mode in
full)
    monitor=$(hyprctl monitors -j | jq -r '.[] | select(.focused) | .name')
    grim -o "$monitor" "$file" || exit 1
    ;;
area)
    workspace=$(hyprctl activeworkspace -j | jq -r .id)
    region=$(hyprctl clients -j |
        jq -r --argjson ws "$workspace" '.[] | select(.workspace.id == $ws and .mapped and .size[0] > 0) | "\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"' |
        slurp "${SLURP_ARGS[@]}") || exit 0
    grim -g "$region" "$file" || exit 1
    ;;
*)
    echo "usage: screenshot.sh full|area" >&2
    exit 1
    ;;
esac

wl-copy < "$file"
~/.config/capture-notify.sh "$file" "Screenshot saved"
