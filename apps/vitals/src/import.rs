//! The temperature log kept since February by log-temps.sh (`<ISO time> cpu=<°C> [gpu=<°C>]`), into
//! the history: the last 35 days as samples (the trends need them), all of it as hourly summaries.

use crate::store::{KEEP_SAMPLES_S, Store};
use chrono::DateTime;

pub const CPU: &str = "temp:k10temp/Tctl";
pub const GPU: &str = "temp:amdgpu/edge";

/// (time, cpu, gpu) per line; lines that are not samples are skipped.
pub fn parse(log: &str) -> Vec<(i64, f64, Option<f64>)> {
    log.lines().filter_map(|line| {
        let mut fields = line.split_whitespace();
        let at = DateTime::parse_from_rfc3339(fields.next()?).ok()?.timestamp();
        let mut cpu = None;
        let mut gpu = None;
        for field in fields {
            if let Some(value) = field.strip_prefix("cpu=") { cpu = value.parse().ok(); }
            if let Some(value) = field.strip_prefix("gpu=") { gpu = value.parse().ok(); }
        }
        Some((at, cpu?, gpu))
    }).collect()
}

pub fn import(store: &Store, log: &str, now: i64) -> rusqlite::Result<usize> {
    let samples = parse(log);
    store.connection.execute_batch("BEGIN")?;
    for (at, cpu, gpu) in &samples {
        for (sensor, value) in [(CPU, Some(*cpu)), (GPU, *gpu)] {
            let Some(value) = value else { continue };
            if now - at < KEEP_SAMPLES_S { store.record(*at, sensor, value)? } else { store.record_hour(*at, sensor, value)? }
        }
    }
    store.connection.execute_batch("COMMIT")?;
    Ok(samples.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_lines_become_samples() {
        let parsed = parse("2026-02-12T19:57:41+01:00 cpu=62.1 gpu=52.0\nnoise\n2026-02-12T19:59:39+01:00 cpu=61.5 gpu=\n");
        assert_eq!(parsed, [(1_770_922_661, 62.1, Some(52.0)), (1_770_922_779, 61.5, None)]);
    }

    #[test]
    fn old_samples_only_feed_the_hours() {
        let store = Store::in_memory().unwrap();
        let now = 1_770_922_661 + 100 * 86400;
        import(&store, "2026-02-12T19:57:41+01:00 cpu=62.1 gpu=52.0\n", now).unwrap();
        assert!(store.series(CPU, 0).unwrap().is_empty());
        assert_eq!(store.hours(CPU, 0).unwrap().len(), 1);
    }
}
