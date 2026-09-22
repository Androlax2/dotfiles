#!/usr/bin/env bash
# Waybar custom module: dGPU usage in the bar, temps/VRAM/power/clocks in the
# tooltip. Everything is read from sysfs, no amdgpu_top/rocm-smi needed.
# The icon lives in the module's format string; the script only returns the value
# and a warning/critical class for the bar to colour it.
# The Raphael iGPU is also an amdgpu card, so the dGPU is picked by PCI id
# rather than by card index (card0/card1 and hwmonN can swap across boots).
GPU_PCI_ID="1002:7550" # RX 9070 XT
GPU_NAME="RX 9070 XT"

for uevent in /sys/class/drm/card*/device/uevent; do
    grep -q "^PCI_ID=$GPU_PCI_ID" "$uevent" && device=$(dirname "$uevent") && break
done
if [ -z "$device" ]; then
    echo "{\"text\": \"?\", \"tooltip\": \"No amdgpu card with PCI id $GPU_PCI_ID\", \"class\": \"critical\"}"
    exit 0
fi
hwmon=$(echo "$device"/hwmon/hwmon*)

read_or_zero() {
    cat "$1" 2>/dev/null || echo 0
}

usage=$(read_or_zero "$device/gpu_busy_percent")
vram_used=$(read_or_zero "$device/mem_info_vram_used")
vram_total=$(read_or_zero "$device/mem_info_vram_total")
edge=$(( $(read_or_zero "$hwmon/temp1_input") / 1000 ))
junction=$(( $(read_or_zero "$hwmon/temp2_input") / 1000 ))
power=$(( $(read_or_zero "$hwmon/power1_average") / 1000000 ))
sclk=$(( $(read_or_zero "$hwmon/freq1_input") / 1000000 ))
mclk=$(( $(read_or_zero "$hwmon/freq2_input") / 1000000 ))
fan=$(read_or_zero "$hwmon/fan1_input")
vram=$(awk -v used="$vram_used" -v total="$vram_total" 'BEGIN { printf "%.1f / %.1f GiB", used / 1073741824, total / 1073741824 }')

class=""
if [ "$usage" -ge 90 ] || [ "$junction" -ge 95 ] || [ "$edge" -ge 85 ]; then
    class="critical"
elif [ "$usage" -ge 70 ]; then
    class="warning"
fi

tooltip="<b>GPU  ${GPU_NAME}</b>\nUsage ${usage}%\nTemp ${edge}°C edge · ${junction}°C junction\nVRAM ${vram}\nPower ${power} W\nClocks ${sclk} / ${mclk} MHz\nFan ${fan} rpm"
echo "{\"text\": \"${usage}%\", \"tooltip\": \"${tooltip}\", \"class\": \"${class}\"}"
