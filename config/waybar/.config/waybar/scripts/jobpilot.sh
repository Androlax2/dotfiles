#!/usr/bin/env bash
# Waybar custom module: jobpilot status. Text is "applications sent automatically today" plus
# "(N)" while N matches wait for manual intervention; the tooltip lists both and reports the last hourly run.
# state.json is read-only here: jobpilot rewrites it whole, so any write of ours would race and lose data.
# A read that lands mid-rewrite fails to parse; it is retried once, then shown as a neutral "–".
# JOBPILOT_STATE overrides the path for testing against a fixture.
STATE="${JOBPILOT_STATE:-/home/theo/IdeaProjects/jobpilot/state.json}"
UNIT="jobpilot-apply"

print_unavailable() {
    jq -cn --arg reason "$1" '{text: "–", tooltip: "<b>jobpilot</b>\n\($reason | gsub("&"; "&amp;") | gsub("<"; "&lt;") | gsub(">"; "&gt;"))", class: "unknown", alt: "unknown"}'
}

if [[ ! -e "$STATE" ]]; then
    print_unavailable "state.json not found at ${STATE}"
    exit 0
fi
if [[ ! -r "$STATE" ]]; then
    print_unavailable "state.json is not readable"
    exit 0
fi

run_state=$(systemctl --user is-active "${UNIT}.service" 2>/dev/null)
last_exit=$(systemctl --user show "${UNIT}.service" -p ExecMainStatus --value 2>/dev/null)
next_run=$(systemctl --user show "${UNIT}.timer" -p NextElapseUSecRealtime --value 2>/dev/null)

# Local-day bounds as epochs, so jq never needs localtime; "tomorrow 00:00" keeps DST days right.
day_start=$(date -d 'today 00:00' +%s)
day_end=$(date -d 'tomorrow 00:00' +%s)

read -r -d '' PROGRAM <<'JQ'
# Timestamps come both as UTC ("…:06.018Z") and with an offset ("…:37+02:00"); fromdateiso8601 takes
# neither the fraction nor the offset, so the offset is applied by hand. Other shapes raise, not hide rows.
def epoch:
  if . == null then null
  else
    (capture("^(?<base>[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2})(\\.[0-9]+)?(?<zone>Z|(?<sign>[+-])(?<hours>[0-9]{2}):(?<minutes>[0-9]{2}))$")
      // error("unrecognised timestamp \(.)")) as $parts
    | ("\($parts.base)Z" | fromdateiso8601)
      - (if $parts.zone == "Z" then 0
         else ($parts.hours | tonumber) * 3600 + ($parts.minutes | tonumber) * 60
              | if $parts.sign == "-" then -. else . end
         end)
  end;
def is_today: epoch as $at | $at != null and $at >= $day_start and $at < $day_end;
def pango: gsub("&"; "&amp;") | gsub("<"; "&lt;") | gsub(">"; "&gt;");
def platform: split(":")[0];
def header: "<span color='#7aa2f7'><b>\(.)</b></span>";
def job_line: "\(.title // "?" | pango) <span color='#a9b1d6'>— \(.company // "?" | pango)</span>";
# Long lists would push the run status off the bottom of the tooltip.
def capped($limit): .[:$limit][], (if length > $limit then "    <span color='#565f89'>+\(length - $limit) more</span>" else empty end);

(.applications // {}) as $applications
| (.manual // {}) as $manual
| [$applications | to_entries[] | select(.value.sentAt | is_today) | .value + {platform: (.key | platform)}] as $auto
| [$manual[] | select(.status == "sent" and (.resolvedAt | is_today))] as $by_hand
| [$manual | to_entries[] | .key as $key | select(.value.status == "pending" and ($applications | has($key) | not)) | .value] as $pending
| [$pending[] | select(.firstSeenAt | is_today)] as $new_pending
| ($run_state == "active" or $run_state == "activating") as $is_running
| ([
    if $last_exit == "2" then "critical" else empty end,
    if $last_exit == "1" then "error" else empty end,
    if $is_running then "running" else empty end,
    if ($pending | length) > 0 then "attention" else empty end
  ]) as $classes
| {
    text: ("\($auto | length)" + (if ($pending | length) > 0 then " (\($pending | length))" else "" end)),
    tooltip: ([
        ("Sent automatically today  \($auto | length)" | header),
        (if ($auto | length) == 0 then "    none"
         else ($auto | sort_by(.sentAt | epoch) | reverse
             | map("    <span color='#565f89'>\(.sentAt | epoch | strflocaltime("%H:%M"))</span> \(.platform) \(job_line)")
             | capped(15)) end),
        "",
        ("By hand today  \($by_hand | length)" | header),
        "",
        ("Needs manual intervention  \($pending | length)" + (if ($new_pending | length) > 0 then " (+\($new_pending | length) new today)" else "" end) | header),
        ($pending | sort_by(.firstSeenAt | epoch) | reverse | map("    \(.offer | job_line)") | capped(10)),
        "",
        ("Last run" | header),
        (if $is_running then "    in progress"
         elif $last_exit == "0" then "    exit 0, clean"
         elif $last_exit == "1" then "    <span color='#ff9e64'>exit 1, a search or application failed</span>"
         elif $last_exit == "2" then "    <span color='#f7768e'>exit 2, credentials need refreshing</span>"
         elif $last_exit == "" then "    unknown (no user systemd bus)"
         else "    exit \($last_exit)" end),
        "    next  \(if $next_run == "" then "unknown" else $next_run end)",
        "",
        "<span color='#565f89'>Click: triage pending · Right-click: last run log</span>"
      ] | join("\n")),
    class: $classes,
    alt: ($classes[0] // "idle")
  }
JQ

run_jq() {
    jq -c \
        --argjson day_start "$day_start" --argjson day_end "$day_end" \
        --arg run_state "$run_state" --arg last_exit "$last_exit" --arg next_run "$next_run" \
        "$PROGRAM" "$STATE" 2>"$jq_errors"
}

# jq's message goes into the tooltip so a schema change shows up as itself, not as "being rewritten".
jq_errors=$(mktemp)
trap 'rm -f "$jq_errors"' EXIT
if ! output=$(run_jq); then
    sleep 0.3
    if ! output=$(run_jq); then
        print_unavailable "state.json could not be read (being rewritten, or invalid):
$(head -c 300 "$jq_errors")"
        exit 0
    fi
fi
printf '%s\n' "$output"
