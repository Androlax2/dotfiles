#!/usr/bin/env bash
# Waybar custom module: "▶ Artist — Title" for the last active MPRIS player (via
# playerctld), one JSON line per change. Prints an empty text when nothing is
# playing or paused, which hides the module; the native mpris module keeps a
# Stopped Firefox player on screen, which is why this script exists.
# Icons are muted (comment), text is fg_dark; max 40 characters.

max_len=40

emit_line() {
    local status=$1 artist=$2 title=$3 icon text tooltip
    case $status in
    Playing) icon="󰐊" ;;
    Paused) icon="󰏤" ;;
    *)
        printf '{"text": ""}\n'
        return
        ;;
    esac
    if [[ -n $artist && -n $title ]]; then
        text="$artist — $title"
    else
        text="${artist}${title}"
    fi
    [[ -z $text ]] && { printf '{"text": ""}\n'; return; }
    tooltip=$text
    if ((${#text} > max_len)); then
        text="${text:0:max_len-1}…"
    fi
    text=$(printf '%s' "$text" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g')
    tooltip=$(printf '%s' "$tooltip" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g')
    jq -cn --arg text "<span color='#565f89'>$icon</span> $text" --arg tooltip "$tooltip" --arg class "${status,,}" \
        '{text: $text, tooltip: $tooltip, class: $class}'
}

if [[ ${1:-} == --test ]]; then
    while IFS='|' read -r status artist title; do emit_line "$status" "$artist" "$title"; done
    exit 0
fi

playerctl --player=playerctld --follow metadata --format '{{status}}|{{artist}}|{{title}}' 2>/dev/null |
    while IFS='|' read -r status artist title; do
        emit_line "$status" "$artist" "$title"
    done
