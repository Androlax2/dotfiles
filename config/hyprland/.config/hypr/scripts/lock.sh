#!/usr/bin/env bash
# The one way the screen locks: hypridle's lock_cmd (so loginctl lock-session, before sleep and
# claude-idle.sh), lock-if-not-gaming.sh and the hot corner all run this. While the screen is
# locked, running Claude sessions are on Remote Control (claude-sessions handoff lock), so they
# can be followed and answered from the phone; unlocking takes back the ones handed off here.
pidof hyprlock >/dev/null && exit 0
locked_at=$(date +%s)
claude-sessions log lock INFO "locked; handing the Claude sessions to Remote Control" 2>/dev/null
claude-sessions handoff lock >/dev/null 2>&1 || true
hyprlock
status=$?
claude-sessions handoff unlock >/dev/null 2>&1 || true
claude-sessions log lock "$( ((status)) && echo ERROR || echo INFO)" "unlocked after $(( $(date +%s) - locked_at )) s (hyprlock exit $status)" 2>/dev/null
