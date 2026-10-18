//! What the machine says beyond its sensors: disk health, space, failed services, kernel errors,
//! memory, the backup.
//!
//! Disk health (SMART) and the kernel log need root: vitals-root.timer, a system unit that runs
//! only smartctl and journalctl, leaves them in /var/lib/vitals for this unprivileged collector.
//! VITALS_ROOT_DATA points elsewhere (tests).

use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Smart {
    pub device: String,
    pub critical_warning: u64,
    pub percentage_used: u64,
    pub available_spare: u64,
    pub spare_threshold: u64,
    pub media_errors: u64,
    /// Seen at the previous collection, to tell new errors from old ones.
    pub previous_media_errors: Option<u64>,
    pub read_at: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Facts {
    pub disks: Vec<Smart>,
    /// mount point → percent used
    pub disk_usage: Vec<(String, f64)>,
    pub failed_units: Vec<String>,
    /// category (gpu, disk, hardware, thermal) → the kernel's lines of the last 24 h
    pub kernel_errors: BTreeMap<String, Vec<String>>,
    pub memory_available_percent: Option<f64>,
    /// Share of the last 5 minutes some task waited on memory (PSI "some avg300").
    pub memory_pressure: Option<f64>,
    /// restic-backup.service's result, when it ran since boot (systemd forgets it at a reboot).
    pub backup_result: Option<String>,
    /// The last successful backup the collector saw (kept across reboots in its own state).
    pub last_backup: Option<i64>,
    pub uptime_s: Option<i64>,
}

pub fn root_data() -> PathBuf {
    std::env::var("VITALS_ROOT_DATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/var/lib/vitals"))
}

fn modified_at(path: &Path) -> i64 {
    fs::metadata(path).and_then(|metadata| metadata.modified()).ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok()).map(|age| age.as_secs() as i64).unwrap_or(0)
}

/// One NVMe drive from `smartctl -j -a` output.
pub fn parse_smart(text: &str, device: &str, read_at: i64) -> Option<Smart> {
    let report: Value = serde_json::from_str(text).ok()?;
    let log = report.get("nvme_smart_health_information_log")?;
    let number = |key: &str| log.get(key).and_then(Value::as_u64).unwrap_or(0);
    Some(Smart { device: device.into(), critical_warning: number("critical_warning"), percentage_used: number("percentage_used"),
                 available_spare: number("available_spare"), spare_threshold: number("available_spare_threshold"),
                 media_errors: number("media_errors"), previous_media_errors: None, read_at })
}

pub fn smart_reports(root: &Path) -> Vec<Smart> {
    let Ok(entries) = fs::read_dir(root.join("smart")) else { return Vec::new() };
    let mut disks: Vec<Smart> = entries.flatten().filter_map(|entry| {
        let path = entry.path();
        let device = path.file_stem()?.to_string_lossy().into_owned();
        parse_smart(&fs::read_to_string(&path).ok()?, &device, modified_at(&path))
    }).collect();
    disks.sort_by(|a, b| a.device.cmp(&b.device));
    disks
}

static KERNEL_CATEGORIES: LazyLock<[(&str, Regex); 4]> = LazyLock::new(|| [
    ("gpu", Regex::new(r"(?i)amdgpu.*(reset|hang|timeout|timed out)|ring \S+ timeout").unwrap()),
    ("disk", Regex::new(r"(?i)I/O error|nvme\S*:.*(timeout|reset|failed)|blk_update_request").unwrap()),
    ("hardware", Regex::new(r"(?i)machine check|\bmce\b|hardware error").unwrap()),
    ("thermal", Regex::new(r"(?i)thermal|throttl|temperature above threshold").unwrap()),
]);

/// The kernel's error lines sorted into what this monitor watches; the rest is ignored (drivers
/// complain about plenty that harms nothing).
pub fn kernel_errors(log: &str) -> BTreeMap<String, Vec<String>> {
    let mut found: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in log.lines() {
        if let Some((category, _)) = KERNEL_CATEGORIES.iter().find(|(_, pattern)| pattern.is_match(line)) {
            found.entry(category.to_string()).or_default().push(line.trim().to_string());
        }
    }
    found
}

/// MemAvailable as a share of MemTotal.
pub fn memory_available(meminfo: &str) -> Option<f64> {
    let field = |name: &str| meminfo.lines().find(|line| line.starts_with(name))
        .and_then(|line| line.split_whitespace().nth(1)).and_then(|kilobytes| kilobytes.parse::<f64>().ok());
    let (available, total) = (field("MemAvailable:")?, field("MemTotal:")?);
    (total > 0.0).then(|| available * 100.0 / total)
}

pub fn memory_pressure(pressure: &str) -> Option<f64> {
    let line = pressure.lines().find(|line| line.starts_with("some "))?;
    line.split_whitespace().find_map(|field| field.strip_prefix("avg300=")).and_then(|value| value.parse().ok())
}

/// `df --output=pcent,target`, real file systems only.
pub fn disk_usage(df: &str) -> Vec<(String, f64)> {
    df.lines().skip(1).filter_map(|line| {
        let (percent, mount) = line.trim().split_once(' ')?;
        Some((mount.trim().to_string(), percent.trim_end_matches('%').parse().ok()?))
    }).collect()
}

/// Unit names from `systemctl --failed --plain --no-legend`.
pub fn failed_units(listing: &str) -> Vec<String> {
    listing.lines().filter_map(|line| line.split_whitespace().next()).map(str::to_string).collect()
}

/// `systemctl show --property=Result,ExecMainExitTimestamp --timestamp=unix`: the result and the
/// time it finished, or None when it has not run since boot.
pub fn backup_state(show: &str) -> Option<(String, i64)> {
    let value = |name: &str| show.lines().find_map(|line| line.strip_prefix(&format!("{name}="))).map(str::to_string);
    let finished = value("ExecMainExitTimestamp").and_then(|stamp| stamp.trim_start_matches('@').parse().ok())?;
    Some((value("Result").unwrap_or_default(), finished))
}

pub fn uptime(text: &str) -> Option<i64> {
    text.split_whitespace().next()?.parse::<f64>().ok().map(|seconds| seconds as i64)
}

fn run(program: &str, arguments: &[&str]) -> String {
    cockpit::util::output_of(program, arguments).unwrap_or_default()
}

pub fn gather(root: &Path) -> Facts {
    let mut failed = failed_units(&run("systemctl", &["--failed", "--plain", "--no-legend", "--no-pager"]));
    failed.extend(failed_units(&run("systemctl", &["--user", "--failed", "--plain", "--no-legend", "--no-pager"])));
    let backup_show = run("systemctl", &["--user", "show", "restic-backup.service", "--property=Result,ExecMainExitTimestamp", "--timestamp=unix"]);
    Facts {
        disks: smart_reports(&root_data()),
        disk_usage: disk_usage(&run("df", &["--output=pcent,target", "-x", "tmpfs", "-x", "devtmpfs", "-x", "efivarfs", "-x", "overlay", "-x", "squashfs"])),
        failed_units: failed,
        kernel_errors: kernel_errors(&fs::read_to_string(root_data().join("kernel-errors.log")).unwrap_or_default()),
        memory_available_percent: fs::read_to_string(root.join("proc/meminfo")).ok().and_then(|text| memory_available(&text)),
        memory_pressure: fs::read_to_string(root.join("proc/pressure/memory")).ok().and_then(|text| memory_pressure(&text)),
        backup_result: backup_state(&backup_show).map(|(result, _)| result),
        last_backup: backup_state(&backup_show).filter(|(result, _)| result == "success").map(|(_, finished)| finished),
        uptime_s: fs::read_to_string(root.join("proc/uptime")).ok().and_then(|text| uptime(&text)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_nvme_smart_report_is_read() {
        let report = r#"{"nvme_smart_health_information_log": {"critical_warning": 0, "available_spare": 100, "available_spare_threshold": 5,
                         "percentage_used": 2, "media_errors": 0}}"#;
        let disk = parse_smart(report, "nvme0", 42).unwrap();
        assert_eq!((disk.percentage_used, disk.spare_threshold, disk.read_at), (2, 5, 42));
    }

    #[test]
    fn kernel_lines_are_sorted_by_what_they_mean() {
        let log = "2026-10-18T10:00:00+0200 kernel: amdgpu 0000:03:00.0: amdgpu: GPU reset begin!\n\
                   2026-10-18T10:01:00+0200 kernel: usb 1-2: device descriptor read/64, error -71\n\
                   2026-10-18T10:02:00+0200 kernel: blk_update_request: I/O error, dev nvme0n1, sector 42\n";
        let found = kernel_errors(log);
        assert_eq!(found.keys().collect::<Vec<_>>(), ["disk", "gpu"]);
    }

    #[test]
    fn available_memory_is_a_share_of_the_total() {
        assert_eq!(memory_available("MemTotal:       1000 kB\nMemFree: 1 kB\nMemAvailable:    250 kB\n"), Some(25.0));
    }

    #[test]
    fn memory_pressure_is_the_five_minute_average() {
        assert_eq!(memory_pressure("some avg10=1.00 avg60=2.00 avg300=3.50 total=1\nfull avg10=0 avg60=0 avg300=0 total=0\n"), Some(3.5));
    }

    #[test]
    fn disk_usage_lists_each_mount() {
        assert_eq!(disk_usage("Use% Mounted on\n 82% /\n  7% /boot\n"), [("/".to_string(), 82.0), ("/boot".to_string(), 7.0)]);
    }

    #[test]
    fn the_backup_state_reads_systemctl_show() {
        assert_eq!(backup_state("Result=success\nExecMainExitTimestamp=@1792300000\n"), Some(("success".into(), 1_792_300_000)));
    }

    #[test]
    fn a_backup_that_has_not_run_since_boot_has_no_state() {
        assert_eq!(backup_state("Result=success\nExecMainExitTimestamp=\n"), None);
    }
}
