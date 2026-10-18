//! What the hardware reports right now: every hwmon temperature and fan, CPU and GPU load.
//!
//! Read from /sys directly (no `sensors` to parse). VITALS_SYS_ROOT points at another tree (tests).
//! A sensor's key is `<kind>:<chip>/<label>`, stable across reboots (hwmonN numbers are not):
//! the chip is its driver name, with its bus address when two chips share a driver (the two RAM
//! sticks), and the label is the driver's own (Tctl, edge, Composite) or tempN / fanN.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn sys_root() -> PathBuf {
    std::env::var("VITALS_SYS_ROOT").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Temperature,
    Fan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub key: String,
    pub kind: Kind,
    pub value: f64,
    /// Where the driver says trouble starts (temp*_max, else temp*_crit minus 10), if it says.
    pub high: Option<f64>,
    pub critical: Option<f64>,
}

fn read_number(path: &Path) -> Option<f64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn read_text(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|text| text.trim().to_string()).filter(|text| !text.is_empty())
}

/// Driver limits outside this range are placeholders (nvme's +65261.8 °C).
fn sane_limit(celsius: f64) -> Option<f64> {
    (celsius > 20.0 && celsius < 150.0).then_some(celsius)
}

/// The hwmon folders, each with the chip name it will be known by.
fn chips(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(root.join("sys/class/hwmon")) else { return Vec::new() };
    let mut found: Vec<(String, String, PathBuf)> = entries.flatten().filter_map(|entry| {
        let folder = entry.path();
        let name = read_text(&folder.join("name"))?;
        let address = fs::read_link(folder.join("device")).ok()
            .and_then(|device| device.file_name().map(|name| name.to_string_lossy().into_owned())).unwrap_or_default();
        Some((name, address, folder))
    }).collect();
    found.sort();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for (name, _, _) in &found {
        *counts.entry(name.clone()).or_default() += 1;
    }
    found.into_iter().map(|(name, address, folder)| {
        let chip = if counts[&name] > 1 && !address.is_empty() { format!("{name} {address}") } else { name };
        (chip, folder)
    }).collect()
}

pub fn read_hwmon(root: &Path) -> Vec<Reading> {
    let mut readings = Vec::new();
    for (chip, folder) in chips(root) {
        let Ok(entries) = fs::read_dir(&folder) else { continue };
        let mut inputs: Vec<String> = entries.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with("_input") && (name.starts_with("temp") || name.starts_with("fan"))).collect();
        inputs.sort();
        for input in inputs {
            let channel = input.trim_end_matches("_input");
            let Some(raw) = read_number(&folder.join(&input)) else { continue };
            let label = read_text(&folder.join(format!("{channel}_label"))).unwrap_or_else(|| channel.to_string());
            if channel.starts_with("temp") {
                let limit = |suffix: &str| read_number(&folder.join(format!("{channel}_{suffix}"))).and_then(|millis| sane_limit(millis / 1000.0));
                let critical = limit("crit");
                let high = limit("max").or_else(|| critical.map(|critical| critical - 10.0));
                readings.push(Reading { key: format!("temp:{chip}/{label}"), kind: Kind::Temperature, value: raw / 1000.0, high, critical });
            } else {
                readings.push(Reading { key: format!("fan:{chip}/{label}"), kind: Kind::Fan, value: raw, high: None, critical: None });
            }
        }
    }
    readings
}

/// CPU time counters (busy, total) from /proc/stat's first line.
pub fn cpu_counters(root: &Path) -> Option<(u64, u64)> {
    let text = fs::read_to_string(root.join("proc/stat")).ok()?;
    let fields: Vec<u64> = text.lines().next()?.split_whitespace().skip(1).filter_map(|field| field.parse().ok()).collect();
    if fields.len() < 4 {
        return None;
    }
    let total: u64 = fields.iter().sum();
    let idle = fields[3] + fields.get(4).copied().unwrap_or(0); // idle + iowait
    Some((total - idle, total))
}

/// Busy share in percent between two counter readings.
pub fn cpu_load(before: (u64, u64), after: (u64, u64)) -> Option<f64> {
    let total = after.1.checked_sub(before.1)?;
    let busy = after.0.checked_sub(before.0)?;
    (total > 0).then(|| busy as f64 * 100.0 / total as f64)
}

/// The busiest GPU's load in percent, if a driver reports one (amdgpu does).
pub fn gpu_load(root: &Path) -> Option<f64> {
    let entries = fs::read_dir(root.join("sys/class/drm")).ok()?;
    entries.flatten().filter_map(|entry| read_number(&entry.path().join("device/gpu_busy_percent"))).reduce(f64::max)
}

/// Where a sensor sorts on screen: the CPU and GPU first, the rest after.
pub fn rank(key: &str) -> (usize, String) {
    let name = label(key);
    let order = ["CPU", "GPU", "RAM", "NVMe", "Board"];
    (order.iter().position(|prefix| name.starts_with(prefix)).unwrap_or(order.len()), name)
}

/// A readable name for a sensor key, for the screen and the banners.
pub fn label(key: &str) -> String {
    let (kind, rest) = key.split_once(':').unwrap_or(("", key));
    let (chip, channel) = rest.split_once('/').unwrap_or((rest, ""));
    let driver = chip.split(' ').next().unwrap_or(chip);
    let named = match (driver, channel) {
        ("k10temp", "Tctl") | ("coretemp", "Package id 0") => "CPU".to_string(),
        ("k10temp", ccd) if ccd.starts_with("Tccd") => format!("CPU die {}", ccd.trim_start_matches("Tccd")),
        ("amdgpu", "edge") => "GPU".to_string(),
        ("amdgpu", "junction") => "GPU hotspot".to_string(),
        ("amdgpu", "mem") => "GPU memory".to_string(),
        ("amdgpu", fan) if kind == "fan" => format!("GPU {fan}"),
        ("nvme", "Composite") => "NVMe".to_string(),
        ("nvme", sensor) => format!("NVMe {}", sensor.to_lowercase()),
        ("spd5118", _) => format!("RAM {}", chip.rsplit('-').next().unwrap_or(chip)),
        ("gigabyte_wmi", temp) => format!("Board {}", temp.trim_start_matches("temp")),
        ("r8169_0_a00:00", _) => "Network chip".to_string(),
        ("prom21_xhci", _) => "USB controller".to_string(),
        _ => format!("{chip} {channel}"),
    };
    match kind {
        "load" => format!("{} load", rest.to_uppercase()),
        _ => named,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chip(root: &Path, number: usize, name: &str, files: &[(&str, &str)], device: Option<&str>) {
        let folder = root.join(format!("sys/class/hwmon/hwmon{number}"));
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("name"), format!("{name}\n")).unwrap();
        for (file, content) in files {
            fs::write(folder.join(file), format!("{content}\n")).unwrap();
        }
        if let Some(device) = device {
            std::os::unix::fs::symlink(format!("../../devices/{device}"), folder.join("device")).unwrap();
        }
    }

    fn scratch() -> PathBuf {
        let root = std::env::temp_dir().join(format!("vitals-sensors-{}", cockpit::util::token_hex(6)));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn temperatures_carry_their_driver_label_and_limits() {
        let root = scratch();
        chip(&root, 2, "k10temp", &[("temp1_input", "52500"), ("temp1_label", "Tctl")], None);
        chip(&root, 7, "amdgpu", &[("temp1_input", "54000"), ("temp1_label", "edge"), ("temp1_crit", "110000"), ("fan1_input", "0")], None);
        let readings = read_hwmon(&root);
        let cpu = readings.iter().find(|reading| reading.key == "temp:k10temp/Tctl").unwrap();
        assert_eq!((cpu.value, cpu.high), (52.5, None));
        let gpu = readings.iter().find(|reading| reading.key == "temp:amdgpu/edge").unwrap();
        assert_eq!((gpu.high, gpu.critical), (Some(100.0), Some(110.0)));
        assert!(readings.iter().any(|reading| reading.key == "fan:amdgpu/fan1" && reading.kind == Kind::Fan));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn two_chips_of_one_driver_are_told_apart_by_their_address() {
        let root = scratch();
        chip(&root, 4, "spd5118", &[("temp1_input", "46000")], Some("2-0051"));
        chip(&root, 5, "spd5118", &[("temp1_input", "45000")], Some("2-0053"));
        let keys: Vec<String> = read_hwmon(&root).into_iter().map(|reading| reading.key).collect();
        assert_eq!(keys, ["temp:spd5118 2-0051/temp1", "temp:spd5118 2-0053/temp1"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn placeholder_limits_are_ignored() {
        let root = scratch();
        chip(&root, 0, "nvme", &[("temp2_input", "34900"), ("temp2_label", "Sensor 1"), ("temp2_max", "65261850")], None);
        assert_eq!(read_hwmon(&root)[0].high, None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cpu_load_is_the_busy_share_between_two_readings() {
        assert_eq!(cpu_load((100, 1000), (400, 2000)), Some(30.0));
        assert_eq!(cpu_load((100, 1000), (100, 1000)), None);
    }

    #[test]
    fn sensors_get_readable_names() {
        assert_eq!(label("temp:k10temp/Tctl"), "CPU");
        assert_eq!(label("temp:amdgpu/junction"), "GPU hotspot");
        assert_eq!(label("temp:spd5118 2-0053/temp1"), "RAM 0053");
        assert_eq!(label("load:cpu"), "CPU load");
        assert_eq!(label("temp:nvme/Sensor 1"), "NVMe sensor 1");
    }
}
