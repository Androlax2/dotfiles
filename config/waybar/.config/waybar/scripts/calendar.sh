#!/usr/bin/env bash
# Waybar custom module: the next khal event as "HH:MM title · N min" (the icon is image#cal-icon),
# with the 7-day agenda in the tooltip. The bar colours come from Pango markup
# in the text, so the module is configured with return-type json and escape false.
HOURS_AHEAD=2
SOON_MINUTES=15

MUTED="#a9b1d6"
GREEN="#9ece6a"
ORANGE="#ff9e64"

# khal prints dates as dd/mm/yyyy (see ~/.config/khal/config), which date(1) misreads.
to_epoch() {
    local day_month_year_time=$1
    date -d "${day_month_year_time:6:4}-${day_month_year_time:3:2}-${day_month_year_time:0:2} ${day_month_year_time:11:5}" +%s
}

escape_pango() {
    sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g'
}

escape_json() {
    sed 's/\\/\\\\/g; s/"/\\"/g'
}

next_event=$(khal list now "${HOURS_AHEAD}h" --format "{start}|{end}|{title}" 2>/dev/null \
    | grep -v -e "^error" -e "^Today" -e "^$" | head -1)

if [[ -z "$next_event" ]]; then
    echo '{"text": ""}'
    exit 0
fi

IFS='|' read -r start end title <<< "$next_event"
start_epoch=$(to_epoch "$start")
end_epoch=$(to_epoch "$end")
now_epoch=$(date +%s)
# Rounded up so "10 min" is shown until the tenth minute is over, as a countdown reads.
minutes_left=$(( (start_epoch - now_epoch + 59) / 60 ))
start_time=${start:11:5}

time_colour=$GREEN
class="upcoming"
countdown="${minutes_left} min"
if (( minutes_left <= 0 )); then
    class="now"
    countdown="now"
elif (( minutes_left <= SOON_MINUTES )); then
    time_colour=$ORANGE
    class="soon"
fi

safe_title=$(printf '%s' "$title" | escape_pango | escape_json)
text="<span color='${time_colour}' weight='600'>${start_time}</span> ${safe_title} <span color='${MUTED}'>· ${countdown}</span>"

tooltip="<b>Upcoming Events</b>\\n\\n"
while IFS= read -r line; do
    if [[ "$line" =~ ^(Today|Tomorrow|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday) ]]; then
        tooltip+="<span color='#7aa2f7'><b>$line</b></span>\\n"
    elif [[ -n "$line" ]]; then
        tooltip+="    $(printf '%s' "$line" | escape_pango | escape_json)\\n"
    fi
done < <(khal list now 7d 2>/dev/null | grep -v "^error" | head -20)

echo "{\"text\": \"${text}\", \"tooltip\": \"${tooltip}\", \"class\": \"${class}\"}"

# Refresh the module when the event ends so the next one shows up without waiting for the interval.
if (( end_epoch > now_epoch )); then
    (sleep $((end_epoch - now_epoch + 5)) && pkill -RTMIN+9 waybar) &>/dev/null &
    disown
fi
