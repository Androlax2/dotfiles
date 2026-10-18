//! Long-term cooling trend: whether the idle temperature of the CPU or GPU crept up.
//!
//! The log (log-temps.sh) has one line per sample, `<ISO time> cpu=<°C> [gpu=<°C>]`. Each day's
//! baseline is its 10th percentile, a stand-in for idle; the median baseline of the last 14 days is
//! compared with the 14 days before.

use chrono::DateTime;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

const DAYS_COMPARED: usize = 14;
const ALERT_DELTA: f64 = 5.0;
const BASELINE_PERCENTILE: f64 = 0.10;

static SAMPLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?P<time>\S+)\s+cpu=(?P<cpu>[-0-9.]+)(?:\s+gpu=(?P<gpu>[-0-9.]+))?$").unwrap());

#[derive(Debug, PartialEq)]
pub enum Verdict {
    /// Not enough days logged yet; the sentence says what is missing.
    TooEarly(String),
    Steady(String),
    Warming(String),
}

#[derive(Default)]
struct Day {
    cpu: Vec<f64>,
    gpu: Vec<f64>,
}

/// The value at fraction `p` of the sorted values, rounded to the nearest index like the Python
/// version did (half to even).
fn percentile(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let index = (((sorted.len() - 1) as f64) * p).round_ties_even() as usize;
    Some(sorted[index.min(sorted.len() - 1)])
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    Some(if sorted.len() % 2 == 1 { sorted[middle] } else { (sorted[middle - 1] + sorted[middle]) / 2.0 })
}

/// Samples by the day of their own timestamp. A line that is not a sample is skipped; a sample
/// whose time does not parse is an error.
fn days_of(log: &str) -> Result<BTreeMap<String, Day>, String> {
    let mut days: BTreeMap<String, Day> = BTreeMap::new();
    for line in log.lines() {
        let Some(sample) = SAMPLE.captures(line.trim()) else { continue };
        let time = DateTime::parse_from_rfc3339(&sample["time"]).map_err(|error| format!("bad time {:?}: {error}", &sample["time"]))?;
        let day = days.entry(time.date_naive().to_string()).or_default();
        day.cpu.push(sample["cpu"].parse().map_err(|_| format!("bad cpu value in {line:?}"))?);
        if let Some(gpu) = sample.name("gpu") {
            day.gpu.push(gpu.as_str().parse().map_err(|_| format!("bad gpu value in {line:?}"))?);
        }
    }
    Ok(days)
}

pub fn check(log: &str) -> Result<Verdict, String> {
    let days = days_of(log)?;
    let baselines: Vec<(Option<f64>, Option<f64>)> = days.values()
        .map(|day| (percentile(&day.cpu, BASELINE_PERCENTILE), percentile(&day.gpu, BASELINE_PERCENTILE))).collect();
    if baselines.len() < 2 * DAYS_COMPARED {
        return Ok(Verdict::TooEarly("Need at least ~28 days of data for trend detection.".into()));
    }
    let last = &baselines[baselines.len() - DAYS_COMPARED..];
    let previous = &baselines[baselines.len() - 2 * DAYS_COMPARED..baselines.len() - DAYS_COMPARED];
    let cpu = |span: &[(Option<f64>, Option<f64>)]| span.iter().filter_map(|day| day.0).collect::<Vec<_>>();
    let gpu = |span: &[(Option<f64>, Option<f64>)]| span.iter().filter_map(|day| day.1).collect::<Vec<_>>();
    let (Some(last_cpu), Some(previous_cpu)) = (median(&cpu(last)), median(&cpu(previous))) else {
        return Ok(Verdict::TooEarly("Not enough CPU data in the last 28 days.".into()));
    };
    let cpu_delta = last_cpu - previous_cpu;
    let gpu_delta = median(&gpu(last)).zip(median(&gpu(previous))).map(|(last, previous)| last - previous);

    let mut problems = Vec::new();
    if cpu_delta >= ALERT_DELTA {
        problems.push(format!("CPU baseline up by {cpu_delta:.1}°C (median 10th pct: {previous_cpu:.1}→{last_cpu:.1})"));
    }
    if let Some(delta) = gpu_delta.filter(|delta| *delta >= ALERT_DELTA) {
        problems.push(format!("GPU baseline up by {delta:.1}°C (last 14d vs previous 14d)"));
    }
    if !problems.is_empty() {
        return Ok(Verdict::Warming(format!("Cooling trend warning:\n- {}", problems.join("\n- "))));
    }
    let gpu_part = gpu_delta.map(|delta| format!(", GPU {delta:+.1}°C")).unwrap_or_default();
    Ok(Verdict::Steady(format!("OK: CPU baseline change {cpu_delta:+.1}°C{gpu_part}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One sample a day at noon for `days` days from 2026-01-01, the CPU at `cpu(day)` °C.
    fn log(days: usize, cpu: impl Fn(usize) -> f64, gpu: Option<f64>) -> String {
        (0..days).map(|day| {
            let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Days::new(day as u64);
            let gpu = gpu.map(|value| format!(" gpu={value:.1}")).unwrap_or_default();
            format!("{date}T12:00:00+01:00 cpu={:.1}{gpu}\n", cpu(day))
        }).collect()
    }

    #[test]
    fn fewer_than_28_days_is_too_early() {
        assert!(matches!(check(&log(27, |_| 50.0, None)).unwrap(), Verdict::TooEarly(_)));
    }

    #[test]
    fn a_steady_baseline_is_ok() {
        assert_eq!(check(&log(28, |_| 50.0, Some(40.0))).unwrap(), Verdict::Steady("OK: CPU baseline change +0.0°C, GPU +0.0°C".into()));
    }

    #[test]
    fn a_cpu_baseline_up_5_degrees_warns() {
        let verdict = check(&log(28, |day| if day < 14 { 50.0 } else { 56.0 }, None)).unwrap();
        assert_eq!(verdict, Verdict::Warming("Cooling trend warning:\n- CPU baseline up by 6.0°C (median 10th pct: 50.0→56.0)".into()));
    }

    #[test]
    fn without_gpu_samples_only_the_cpu_is_reported() {
        assert_eq!(check(&log(28, |_| 50.0, None)).unwrap(), Verdict::Steady("OK: CPU baseline change +0.0°C".into()));
    }

    #[test]
    fn the_baseline_is_the_days_10th_percentile() {
        assert_eq!(percentile(&[70.0, 40.0, 60.0, 50.0, 80.0, 90.0, 55.0, 65.0, 75.0, 85.0, 95.0], 0.10), Some(50.0));
    }

    #[test]
    fn lines_that_are_not_samples_are_skipped() {
        let text = format!("garbage\n{}", log(28, |_| 50.0, None));
        assert!(matches!(check(&text).unwrap(), Verdict::Steady(_)));
    }

    #[test]
    fn a_sample_with_a_bad_time_is_an_error() {
        assert!(check("yesterday cpu=50.0\n").is_err());
    }
}
