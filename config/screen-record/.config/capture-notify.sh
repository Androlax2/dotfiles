#!/usr/bin/env bash
# Notification for a saved capture, with the file as preview. Clicking it reveals the
# file in Files (FileManager1.ShowItems); images also get an "Annotate" button that
# opens swappy and re-copies the result. Runs detached so the caller returns at once.
# Usage: capture-notify.sh <file> <title>
set -uo pipefail

file=${1:?usage: capture-notify.sh <file> <title>}
title=${2:?usage: capture-notify.sh <file> <title>}

reveal() {
    busctl --user call org.freedesktop.FileManager1 /org/freedesktop/FileManager1 \
        org.freedesktop.FileManager1 ShowItems ass 1 "file://$file" "" >/dev/null 2>&1
}

actions=(-A "default=Show in Files")
case $file in
*.png | *.jpg | *.jpeg) actions+=(-A "annotate=Annotate") ;;
esac

(
    answer=$(notify-send -a "Capture" -i "$file" -t 10000 "${actions[@]}" "$title" "$(basename "$file")")
    case $answer in
    default) reveal ;;
    annotate)
        if swappy -f "$file" -o "$file"; then
            wl-copy < "$file"
        fi
        ;;
    esac
) >/dev/null 2>&1 </dev/null &
disown
