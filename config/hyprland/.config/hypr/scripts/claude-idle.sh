#!/usr/bin/env bash
# hypridle listeners for the Claude workspace (6, hypr/apps/claude.lua). Reading a session's
# output is idle time to hypridle, so there the lock comes later than elsewhere (the standard
# listeners stand down on that workspace) and asks first:
#   claude-idle.sh prompt   on-timeout: the "still there?" screen (claude-idle-prompt.py) for
#                           GRACE_S, then the lock unless it was answered
#   claude-idle.sh cancel   on-resume: input happened, close the screen and forget the lock
# Any key, click or mouse move answers the screen itself; hypridle sees the same input as resumed
# activity and runs cancel. The screen goes dark SCREEN_OFF_AFTER_S after the lock.
# CLAUDE_IDLE_GRACE_S / CLAUDE_IDLE_SCREEN_OFF_AFTER_S override the delays (tests).
GRACE_S=${CLAUDE_IDLE_GRACE_S:-30}
SCREEN_OFF_AFTER_S=${CLAUDE_IDLE_SCREEN_OFF_AFTER_S:-30}
PID_FILE="${XDG_RUNTIME_DIR:-/tmp}/claude-idle.pid"
ON_CLAUDE_WORKSPACE="$HOME/.config/waybar/scripts/on-claude-workspace.sh"
PROMPT="$HOME/.config/hypr/scripts/claude-idle-prompt.py"

prompt() {
    "$ON_CLAUDE_WORKSPACE" || exit 0
    # shellcheck source=/dev/null
    source "$HOME/.config/hypr/scripts/is-gaming.sh" >/dev/null 2>&1 || true
    if is_gaming; then exit 0; fi
    claude-sessions log idle INFO "idle on the Claude workspace: asking before the lock" 2>/dev/null
    "$PROMPT" "$GRACE_S" &
    echo $! > "$PID_FILE"
    if wait $!; then
        rm -f "$PID_FILE"
        claude-sessions log idle INFO "answered: no lock" 2>/dev/null
        exit 0 # answered
    fi
    rm -f "$PID_FILE"
    claude-sessions log idle INFO "no answer in ${GRACE_S} s: locking" 2>/dev/null
    loginctl lock-session
    sleep "$SCREEN_OFF_AFTER_S"
    if pidof hyprlock >/dev/null; then
        hyprctl dispatch 'hl.dsp.dpms({action = "off"})'
    fi
}

cancel() {
    [[ -f $PID_FILE ]] || return 0
    kill "$(cat "$PID_FILE")" 2>/dev/null || true
    rm -f "$PID_FILE"
}

case ${1:-} in
    prompt) prompt ;;
    cancel) cancel ;;
    *) echo "usage: claude-idle.sh prompt|cancel" >&2; exit 2 ;;
esac
