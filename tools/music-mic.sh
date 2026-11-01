#!/usr/bin/env bash
# make music-mic-on / music-mic-off: a virtual microphone for game voice chat that carries the
# default microphone plus everything played into the "Music" output, Apple Music moved there. That
# output is also looped to the speakers, so the music is still heard locally.
# Every module's arguments name a music_mic_ device, which is how off finds them; the only state
# kept is the microphone to restore as the default source.
set -Eeuo pipefail

previous_source_file=${XDG_RUNTIME_DIR:?}/music-mic-previous-source

# The Apple Music scratchpad is a Zen window on its own profile (hypr/scripts/web-launch.sh starts
# it with -P music); its sound comes from that profile's processes.
route_apple_music() {
    local zen_pid stream
    zen_pid=$(pgrep -o -f -- '-P music ' || true)
    if [ -z "$zen_pid" ]; then
        echo "À faire : Apple Music n'est pas ouvert. Ouvre-le (CMD+M), lance un morceau, puis relance make music-mic-on."
        return
    fi
    local profile_pids streams
    profile_pids=$({ echo "$zen_pid"; pgrep -P "$zen_pid" || true; } | jq -s 'map(tostring)')
    streams=$(pactl -f json list sink-inputs | jq -r --argjson pids "$profile_pids" \
        '.[] | select(.properties["application.process.id"] as $pid | $pids | index($pid)) | .index')
    if [ -z "$streams" ]; then
        echo "À faire : Apple Music ne joue rien. Lance un morceau, puis relance make music-mic-on."
        return
    fi
    for stream in $streams; do
        pactl move-sink-input "$stream" music_mic_music
    done
    echo "OK : Apple Music passe dans le micro (tu l'entends toujours dans ton casque)."
}

loaded_modules() {
    pactl list modules short | awk '/music_mic_/ {print $1}'
}

turn_on() {
    if [ -n "$(loaded_modules)" ]; then
        echo "Le micro musique est déjà actif."
        route_apple_music
        return
    fi
    local microphone speakers
    microphone=$(pactl get-default-source)
    speakers=$(pactl get-default-sink)
    trap turn_off ERR
    pactl load-module module-null-sink sink_name=music_mic_music sink_properties=device.description=Music >/dev/null
    pactl load-module module-null-sink sink_name=music_mic_mix sink_properties=device.description=MicMix >/dev/null
    pactl load-module module-loopback source="$microphone" sink=music_mic_mix latency_msec=20 >/dev/null
    pactl load-module module-loopback source=music_mic_music.monitor sink=music_mic_mix latency_msec=20 >/dev/null
    pactl load-module module-loopback source=music_mic_music.monitor sink="$speakers" latency_msec=20 >/dev/null
    pactl load-module module-remap-source master=music_mic_mix.monitor source_name=music_mic_source source_properties=device.description=VirtualMic >/dev/null
    echo "$microphone" > "$previous_source_file"
    pactl set-default-source music_mic_source
    trap - ERR
    route_apple_music
    cat <<'STEPS'

Dans Arc Raiders (options audio) :
  1. Micro (entrée) : VirtualMic, ou "par défaut".
  2. Mode du micro : détection vocale / micro ouvert. En push-to-talk, la musique ne passe
     que quand tu appuies sur la touche.
Musique trop forte pour les autres : pavucontrol, onglet Lecture, baisse le loopback vers MicMix.
Pour arrêter : make music-mic-off
STEPS
}

turn_off() {
    local module
    for module in $(loaded_modules | sort -rn); do
        pactl unload-module "$module"
    done
    if [ -s "$previous_source_file" ]; then
        pactl set-default-source "$(cat "$previous_source_file")"
        rm "$previous_source_file"
    fi
    echo "Micro musique coupé : ton vrai micro est de nouveau celui par défaut. Si tu avais choisi VirtualMic dans le jeu, remets ton micro."
}

case ${1:-} in
    on) turn_on ;;
    off) turn_off ;;
    *) echo "usage: $0 on|off" >&2; exit 2 ;;
esac
