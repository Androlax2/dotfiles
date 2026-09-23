#!/usr/bin/env bash
# Single palette source. Tokens live in config/waybar/.config/waybar/colors.css
# (`@define-color name value;`); every other themed file is rendered from a template
# in tools/palette/ that uses {{name}} (#rrggbb or the rgba() value), {{name_hex}}
# (rrggbb, no #) and {{name_rgb}} ("r, g, b" for rgba() forms).
#
#   tools/palette.sh              render every target in tools/palette/targets.txt
#   tools/palette.sh --check      exit 1 with a diff when a target differs from its template
#   tools/palette.sh --adopt FILE print FILE with palette values replaced by placeholders
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
tokens_file=$repo/config/waybar/.config/waybar/colors.css
manifest=$repo/tools/palette/targets.txt

declare -A value
names=()
while read -r name raw; do
    if [[ $raw == @* ]]; then
        raw=${value[${raw#@}]:?"palette: $name refers to unknown token ${raw#@}"}
    fi
    value[$name]=$raw
    names+=("$name")
done < <(sed -n 's/^@define-color \([A-Za-z0-9]*\) \(.*\);.*$/\1 \2/p' "$tokens_file")

hex_to_rgb() { printf '%d, %d, %d' "0x${1:0:2}" "0x${1:2:2}" "0x${1:4:2}"; }

forward_script() {
    local name hex
    for name in "${names[@]}"; do
        if [[ ${value[$name]} =~ ^#([0-9a-f]{6})$ ]]; then
            hex=${BASH_REMATCH[1]}
            printf 's/{{%s}}/#%s/g\ns/{{%s_hex}}/%s/g\ns/{{%s_rgb}}/%s/g\n' \
                "$name" "$hex" "$name" "$hex" "$name" "$(hex_to_rgb "$hex")"
        else
            printf 's/{{%s}}/%s/g\n' "$name" "${value[$name]//\//\\/}"
        fi
    done
}

# Reverse mapping for --adopt. Aliases come last in colors.css, so the first token
# with a given value wins. Exact rgba() tokens first, then "#hex", then the bare
# "r, g, b" and "hex" forms.
reverse_script() {
    local name hex
    for name in "${names[@]}"; do
        [[ ${value[$name]} =~ ^#([0-9a-f]{6})$ ]] || printf 's/%s/{{%s}}/g\n' "${value[$name]//\//\\/}" "$name"
    done
    for name in "${names[@]}"; do
        [[ ${value[$name]} =~ ^#([0-9a-f]{6})$ ]] || continue
        hex=${BASH_REMATCH[1]}
        printf 's/#%s\\b/{{%s}}/gI\n' "$hex" "$name"
        printf 's/#ff%s\\b/#ff{{%s_hex}}/gI\n' "$hex" "$name" # qt6ct's #aarrggbb form
    done
    for name in "${names[@]}"; do
        [[ ${value[$name]} =~ ^#([0-9a-f]{6})$ ]] || continue
        hex=${BASH_REMATCH[1]}
        printf 's/rgba(%s,/rgba({{%s_rgb}},/g\n' "$(hex_to_rgb "$hex")" "$name"
        printf 's/\\(^\\|[^#{a-zA-Z0-9]\\)%s/\\1{{%s_hex}}/gI\n' "$hex" "$name"
    done
}

render() { sed -f <(forward_script) "$1"; }

case ${1:-} in
--adopt)
    sed -f <(reverse_script) "${2:?usage: palette.sh --adopt FILE}"
    ;;
--check | "")
    check=${1:-}
    drift=0
    while IFS=$'\t' read -r template target; do
        [[ -z $template || $template == \#* ]] && continue
        rendered=$(render "$repo/tools/palette/$template"; printf x)
        rendered=${rendered%x}
        if [[ -n $check ]]; then
            if ! diff -u --label "rendered: $template" --label "repo: $target" <(printf '%s' "$rendered") "$repo/$target"; then
                drift=1
            fi
        else
            printf '%s' "$rendered" > "$repo/$target"
            echo "wrote $target"
        fi
    done < "$manifest"
    exit $drift
    ;;
*)
    echo "usage: palette.sh [--check | --adopt FILE]" >&2
    exit 2
    ;;
esac
