#!/usr/bin/env python3
import re, sys
from datetime import datetime, timedelta, timezone
from collections import defaultdict

LOG = "/home/theo/.local/share/templogs/temps.log"

LINE_RE = re.compile(r"^(?P<ts>[^ ]+)\s+cpu=(?P<cpu>[-0-9.]+)(?:\s+gpu=(?P<gpu>[-0-9.]+))?$")

def parse_iso(ts: str) -> datetime:
    # date -Is yields ISO8601 like 2026-02-12T19:03:10+01:00
    return datetime.fromisoformat(ts)

def percentile(values, p: float):
    if not values:
        return None
    vs = sorted(values)
    idx = int(round((len(vs)-1) * p))
    return vs[max(0, min(len(vs)-1, idx))]

def median(values):
    if not values:
        return None
    vs = sorted(values)
    n = len(vs)
    mid = n // 2
    return (vs[mid] if n % 2 else (vs[mid-1] + vs[mid]) / 2)

def main():
    try:
        lines = open(LOG, "r", encoding="utf-8").read().splitlines()
    except FileNotFoundError:
        print("No log file yet.")
        return 0

    by_day_cpu = defaultdict(list)
    by_day_gpu = defaultdict(list)

    for line in lines:
        m = LINE_RE.match(line.strip())
        if not m:
            continue
        ts = parse_iso(m.group("ts"))
        day = ts.date().isoformat()
        cpu = float(m.group("cpu"))
        by_day_cpu[day].append(cpu)
        gpu_s = m.group("gpu")
        if gpu_s is not None and gpu_s != "":
            by_day_gpu[day].append(float(gpu_s))

    # compute "baseline" as 10th percentile of each day (proxy for idle-ish)
    daily_cpu = {}
    daily_gpu = {}
    for day, vals in by_day_cpu.items():
        daily_cpu[day] = percentile(vals, 0.10)
    for day, vals in by_day_gpu.items():
        if vals:
            daily_gpu[day] = percentile(vals, 0.10)

    all_days = sorted(daily_cpu.keys())
    if len(all_days) < 28:
        print("Need at least ~28 days of data for trend detection.")
        return 0

    # last 14 days vs previous 14 days
    last14 = all_days[-14:]
    prev14 = all_days[-28:-14]

    last_cpu = [daily_cpu[d] for d in last14 if daily_cpu[d] is not None]
    prev_cpu = [daily_cpu[d] for d in prev14 if daily_cpu[d] is not None]

    last_gpu = [daily_gpu.get(d) for d in last14 if daily_gpu.get(d) is not None]
    prev_gpu = [daily_gpu.get(d) for d in prev14 if daily_gpu.get(d) is not None]

    if not last_cpu or not prev_cpu:
        print("Not enough CPU data in the last 28 days.")
        return 0

    last_cpu_med = median(last_cpu)
    prev_cpu_med = median(prev_cpu)
    cpu_delta = last_cpu_med - prev_cpu_med

    gpu_delta = None
    if last_gpu and prev_gpu:
        gpu_delta = median(last_gpu) - median(prev_gpu)

    # thresholds
    CPU_DELTA_ALERT = 5.0
    GPU_DELTA_ALERT = 5.0

    problems = []
    if cpu_delta >= CPU_DELTA_ALERT:
        problems.append(f"CPU baseline up by {cpu_delta:.1f}°C (median 10th pct: {prev_cpu_med:.1f}→{last_cpu_med:.1f})")

    if gpu_delta is not None and gpu_delta >= GPU_DELTA_ALERT:
        problems.append(f"GPU baseline up by {gpu_delta:.1f}°C (last 14d vs previous 14d)")

    if problems:
        msg = "Cooling trend warning:\n- " + "\n- ".join(problems)
        print(msg)
        return 2

    print(f"OK: CPU baseline change {cpu_delta:+.1f}°C" + (f", GPU {gpu_delta:+.1f}°C" if gpu_delta is not None else ""))
    return 0

if __name__ == "__main__":
    sys.exit(main())

