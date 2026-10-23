#!/usr/bin/env bash
# The one way the screen locks: hypridle's lock_cmd (so loginctl lock-session, before sleep and
# claude-idle.sh), lock-if-not-gaming.sh and the hot corner all run this. While the screen is
# locked, the phone page (cockpit-remote) notifies: a session needs you, an ask-to-run asks, a turn
# ends (cockpit away on); unlocked, it stays quiet. The setting phone-on-lock (the
# cockpit's S, or cockpit setting) turns that off.
pidof hyprlock >/dev/null && exit 0
locked_at=$(date +%s)
cockpit log lock INFO "locked" 2>/dev/null
cockpit away on >/dev/null 2>&1 || true
hyprlock
status=$?
cockpit away off >/dev/null 2>&1 || true
cockpit log lock "$( ((status)) && echo ERROR || echo INFO)" "unlocked after $(( $(date +%s) - locked_at )) s (hyprlock exit $status)" 2>/dev/null
