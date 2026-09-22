#!/usr/bin/env bash
# Waybar custom module: CPU usage in the bar, temps/load/freq in the tooltip.
# Usage is a delta against the previous /proc/stat sample kept in
# $XDG_RUNTIME_DIR, so each 2s poll reports the average over that window.
prev_sample="${XDG_RUNTIME_DIR:-/tmp}/waybar-cpu-stats.prev"

read_cpu_line() {
    awk '/^cpu / { idle = $5 + $6; total = 0; for (i = 2; i <= NF; i++) total += $i; print idle, total }' /proc/stat
}

current=$(read_cpu_line)
if [ -f "$prev_sample" ]; then
    previous=$(cat "$prev_sample")
else
    previous=$current
    sleep 1
    current=$(read_cpu_line)
fi
echo "$current" > "$prev_sample"

usage=$(awk -v prev="$previous" -v cur="$current" 'BEGIN {
    split(prev, p, " "); split(cur, c, " ")
    total = c[2] - p[2]
    if (total <= 0) { print 0; exit }
    printf "%d", 100 * (1 - (c[1] - p[1]) / total)
}')

for hwmon in /sys/class/hwmon/hwmon*; do
    [ "$(cat "$hwmon/name")" = "k10temp" ] && cpu_hwmon=$hwmon && break
done
if [ -z "$cpu_hwmon" ]; then
    echo '{"text": "CPU ?", "tooltip": "k10temp hwmon not found", "class": "hot"}'
    exit 0
fi

tctl=$(( $(cat "$cpu_hwmon/temp1_input") / 1000 ))
tccd1=$(( $(cat "$cpu_hwmon/temp3_input" 2>/dev/null || echo 0) / 1000 ))
load=$(cut -d' ' -f1-3 /proc/loadavg)
avg_freq=$(awk '{ sum += $1; n++ } END { printf "%.1f", sum / n / 1000000 }' /sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq)

class=""
[ "$tctl" -ge 85 ] && class="hot"

tooltip="<b>CPU</b>\nUsage ${usage}%\nTctl ${tctl}°C · Tccd1 ${tccd1}°C\nLoad ${load}\nFreq ${avg_freq} GHz"
echo "{\"text\": \"<span font_family='JetBrainsMono NF' size='11pt'>󰻠</span> ${usage}%\", \"tooltip\": \"${tooltip}\", \"class\": \"${class}\"}"
