//! What counts as wrong. Pure: the collector hands in the history and the facts it gathered, and
//! gets back the findings that hold now. A finding that stops holding clears its alert.

use crate::facts::{Facts, Smart};
use crate::sensors::label;
use std::collections::BTreeMap;

pub const DAY: i64 = 86400;
/// Three readings in a row over a limit: one spike is no finding.
const SUSTAINED_MINUTES: usize = 3;
const DEFAULT_HIGH: f64 = 85.0;
/// Below this load a minute counts as idle; at or above, as under load.
const IDLE_LOAD: f64 = 15.0;
const BUSY_LOAD: f64 = 70.0;
const TREND_RISE: f64 = 5.0;
const TREND_WINDOW_DAYS: i64 = 14;
const MIN_DAYS_PER_WINDOW: usize = 7;
const MIN_IDLE_MINUTES_PER_DAY: usize = 30;
const MIN_BUSY_MINUTES: usize = 20;
const FAN_MATTERS_FROM: f64 = 60.0;
const DISK_WARNING_PERCENT: f64 = 85.0;
const DISK_CRITICAL_PERCENT: f64 = 95.0;
const DISK_WORN_PERCENT: u64 = 80;
const MEMORY_LOW_PERCENT: f64 = 5.0;
const MEMORY_PRESSURE: f64 = 20.0;
const BACKUP_MAX_AGE_S: i64 = 2 * DAY;
const BACKUP_GRACE_AFTER_BOOT_S: i64 = 3600;
const SMART_MAX_AGE_S: i64 = 2 * 3600;

/// The temperatures whose trend matters, each with the load that tells idle from busy.
pub const TRENDED: [(&str, &str, &str); 2] = [("temp:k10temp/Tctl", "load:cpu", "cpu"), ("temp:amdgpu/edge", "load:gpu", "gpu")];

#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub key: String,
    pub level: &'static str,
    pub title: String,
    pub detail: String,
}

fn warn(key: impl Into<String>, title: impl Into<String>, detail: impl Into<String>) -> Finding {
    Finding { key: key.into(), level: "warn", title: title.into(), detail: detail.into() }
}

fn critical(key: impl Into<String>, title: impl Into<String>, detail: impl Into<String>) -> Finding {
    Finding { key: key.into(), level: "crit", title: title.into(), detail: detail.into() }
}

/// Alert keys become OSD notice ids: lowercase letters, digits and dashes.
fn slug(text: &str) -> String {
    let mut slug: String = text.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').chars().take(40).collect()
}

fn percentile(values: &mut [f64], fraction: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(values[(((values.len() - 1) as f64) * fraction).round() as usize])
}

pub type Minutes = BTreeMap<i64, BTreeMap<String, f64>>;
pub type Limits = BTreeMap<String, (Option<f64>, Option<f64>)>;

/// Where a sensor's trouble starts: the driver's limit, else ours.
pub fn high_of(sensor: &str, limits: &Limits) -> f64 {
    limits.get(sensor).and_then(|limit| limit.0).unwrap_or(match sensor {
        "temp:k10temp/Tctl" => 90.0,
        _ => DEFAULT_HIGH,
    })
}

/// Temperatures over their limit for the last few readings, or past the driver's critical one now.
pub fn over_limits(minutes: &Minutes, limits: &Limits) -> Vec<Finding> {
    let recent: Vec<&BTreeMap<String, f64>> = minutes.values().rev().take(SUSTAINED_MINUTES).collect();
    let Some(latest) = recent.first() else { return Vec::new() };
    let mut findings = Vec::new();
    for (sensor, value) in latest.iter().filter(|(sensor, _)| sensor.starts_with("temp:")) {
        let name = label(sensor);
        if let Some(critical_at) = limits.get(sensor).and_then(|limit| limit.1).filter(|critical_at| value >= critical_at) {
            findings.push(critical(format!("hot-{}", slug(&name)), format!("{name} critique : {value:.0} °C"),
                                   format!("{name} est à {value:.0} °C, au-delà de sa limite critique de {critical_at:.0} °C.")));
            continue;
        }
        let high = high_of(sensor, limits);
        let sustained = recent.len() == SUSTAINED_MINUTES && recent.iter().all(|minute| minute.get(sensor).is_some_and(|reading| *reading >= high));
        if sustained {
            findings.push(warn(format!("hot-{}", slug(&name)), format!("{name} trop chaud : {value:.0} °C"),
                               format!("{name} est resté à {high:.0} °C ou plus pendant {SUSTAINED_MINUTES} minutes (maintenant {value:.0} °C).")));
        }
    }
    findings
}

fn day_of(at: i64) -> i64 {
    at.div_euclid(DAY)
}

/// Each day's idle temperature: the 10th percentile of its idle minutes (all of them when the load
/// is unknown, as in the imported log).
pub fn idle_baselines(minutes: &Minutes, sensor: &str, load: &str) -> BTreeMap<i64, f64> {
    let mut by_day: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
    for (at, readings) in minutes {
        let Some(value) = readings.get(sensor) else { continue };
        if readings.get(load).is_none_or(|busy| *busy < IDLE_LOAD) {
            by_day.entry(day_of(*at)).or_default().push(*value);
        }
    }
    by_day.into_iter().filter(|(_, values)| values.len() >= MIN_IDLE_MINUTES_PER_DAY)
        .filter_map(|(day, mut values)| percentile(&mut values, 0.10).map(|baseline| (day, baseline))).collect()
}

fn median(values: &mut [f64]) -> Option<f64> {
    percentile(values, 0.5)
}

/// The idle temperature of the last two weeks against the two before: dust, a tiring pump or
/// dried paste show here first, weeks before any limit.
pub fn idle_trend(minutes: &Minutes, now: i64, sensor: &str, load: &str, name: &str) -> Option<Finding> {
    let baselines = idle_baselines(minutes, sensor, load);
    let today = day_of(now);
    let window = |from: i64, to: i64| -> Vec<f64> { baselines.range(today - to..today - from).map(|(_, value)| *value).collect() };
    let (mut last, mut before) = (window(0, TREND_WINDOW_DAYS), window(TREND_WINDOW_DAYS, 2 * TREND_WINDOW_DAYS));
    if last.len() < MIN_DAYS_PER_WINDOW || before.len() < MIN_DAYS_PER_WINDOW {
        return None;
    }
    let (now_idle, earlier_idle) = (median(&mut last)?, median(&mut before)?);
    let rise = now_idle - earlier_idle;
    (rise >= TREND_RISE).then(|| warn(format!("idle-{name}"), format!("{} plus chaud au repos : +{rise:.1} °C", name.to_uppercase()),
        format!("Au repos, le {} est passé de {earlier_idle:.1} °C à {now_idle:.1} °C en deux semaines : poussière dans les filtres ou le radiateur, \
                 la pompe, ou la pâte thermique.", name.to_uppercase())))
}

/// The temperature under load this week against the same load three weeks earlier.
pub fn load_trend(minutes: &Minutes, now: i64, sensor: &str, load: &str, name: &str) -> Option<Finding> {
    let busy_between = |from: i64, to: i64| -> Vec<f64> {
        minutes.range(now - to..now - from)
            .filter(|(_, readings)| readings.get(load).is_some_and(|busy| *busy >= BUSY_LOAD))
            .filter_map(|(_, readings)| readings.get(sensor).copied()).collect()
    };
    let (mut this_week, mut earlier) = (busy_between(0, 7 * DAY), busy_between(21 * DAY, 28 * DAY));
    if this_week.len() < MIN_BUSY_MINUTES || earlier.len() < MIN_BUSY_MINUTES {
        return None;
    }
    let (now_busy, earlier_busy) = (median(&mut this_week)?, median(&mut earlier)?);
    let rise = now_busy - earlier_busy;
    (rise >= TREND_RISE).then(|| warn(format!("load-{name}"), format!("{} plus chaud en charge : +{rise:.1} °C", name.to_uppercase()),
        format!("En charge, le {} est passé de {earlier_busy:.1} °C il y a trois semaines à {now_busy:.1} °C cette semaine : le refroidissement suit moins bien.",
                name.to_uppercase())))
}

/// Pump readings at idle below this many minutes per window say nothing.
const MIN_IDLE_PUMP_MINUTES: usize = 120;
/// A pump that lost this share of its idle speed in three weeks is tiring.
const PUMP_SLOWDOWN: f64 = 0.15;
/// A disk filling up warns this many days before it is full.
const FILL_WARNING_DAYS: f64 = 30.0;
const MIN_FILL_HISTORY_S: i64 = 3 * DAY;

/// The pump's idle speed this week against three weeks earlier: a pump wears out slowly, and
/// slows down before it stops.
pub fn pump_trend(minutes: &Minutes, now: i64, pump: &str, load: &str) -> Option<Finding> {
    let idle_between = |from: i64, to: i64| -> Vec<f64> {
        minutes.range(now - to..now - from)
            .filter(|(_, readings)| readings.get(load).is_some_and(|busy| *busy < IDLE_LOAD))
            .filter_map(|(_, readings)| readings.get(pump).copied()).filter(|rpm| *rpm > 0.0).collect()
    };
    let (mut this_week, mut earlier) = (idle_between(0, 7 * DAY), idle_between(21 * DAY, 28 * DAY));
    if this_week.len() < MIN_IDLE_PUMP_MINUTES || earlier.len() < MIN_IDLE_PUMP_MINUTES {
        return None;
    }
    let (now_rpm, earlier_rpm) = (median(&mut this_week)?, median(&mut earlier)?);
    let slowdown = 1.0 - now_rpm / earlier_rpm;
    (slowdown >= PUMP_SLOWDOWN).then(|| warn("pump-slower", format!("La pompe ralentit : −{:.0} %", slowdown * 100.0),
        format!("Au repos, la pompe est passée de {earlier_rpm:.0} à {now_rpm:.0} tr/min en trois semaines : vérifie le watercooling \
                 (bruit, bulles, raccords) avant qu'elle ne s'arrête.")))
}

/// Days until a disk is full at the pace of its history (time, percent used), oldest first; None
/// when it is not filling up or the history is too short to tell.
pub fn days_until_full(history: &[(i64, f64)]) -> Option<f64> {
    let (first, last) = (history.first()?, history.last()?);
    if last.0 - first.0 < MIN_FILL_HISTORY_S {
        return None;
    }
    let count = history.len() as f64;
    let mean_time = history.iter().map(|point| point.0 as f64).sum::<f64>() / count;
    let mean_used = history.iter().map(|point| point.1).sum::<f64>() / count;
    let covariance: f64 = history.iter().map(|point| (point.0 as f64 - mean_time) * (point.1 - mean_used)).sum();
    let variance: f64 = history.iter().map(|point| (point.0 as f64 - mean_time).powi(2)).sum();
    let per_second = covariance / variance;
    (per_second > 0.0).then(|| (100.0 - last.1).max(0.0) / per_second / DAY as f64)
}

/// A disk that will be full within a month at its pace; the space warning takes over at 85 %.
pub fn filling(mount: &str, history: &[(i64, f64)]) -> Option<Finding> {
    let used = history.last()?.1;
    let days = days_until_full(history).filter(|days| *days <= FILL_WARNING_DAYS && used < DISK_WARNING_PERCENT)?;
    let key = match slug(mount).as_str() { "" => "root".to_string(), named => named.to_string() };
    Some(warn(format!("filling-{key}"), format!("{mount} sera plein dans ~{days:.0} jours"),
              format!("{mount} est plein à {used:.0} % et se remplit au rythme des deux dernières semaines : fais de la place.")))
}

/// A fan or pump that stopped while its chip is warm, though it spun this week. Fans that stop by
/// design when cool (the GPU's zero-RPM mode) stay quiet below FAN_MATTERS_FROM.
pub fn stopped_fans(minutes: &Minutes, now: i64) -> Vec<Finding> {
    let recent: Vec<&BTreeMap<String, f64>> = minutes.values().rev().take(SUSTAINED_MINUTES).collect();
    let Some(latest) = recent.first() else { return Vec::new() };
    let mut findings = Vec::new();
    for sensor in latest.keys().filter(|sensor| sensor.starts_with("fan:")) {
        let stopped = recent.len() == SUSTAINED_MINUTES && recent.iter().all(|minute| minute.get(sensor).is_some_and(|rpm| *rpm == 0.0));
        let spun_this_week = minutes.range(now - 7 * DAY..).any(|(_, readings)| readings.get(sensor).is_some_and(|rpm| *rpm > 0.0));
        let chip = sensor.trim_start_matches("fan:").split('/').next().unwrap_or("");
        let chip_temperature = latest.iter().filter(|(key, _)| key.trim_start_matches("temp:").split('/').next() == Some(chip) && key.starts_with("temp:"))
            .map(|(_, value)| *value).reduce(f64::max)
            .or_else(|| latest.get("temp:k10temp/Tctl").copied()).unwrap_or(0.0);
        if stopped && spun_this_week && chip_temperature >= FAN_MATTERS_FROM {
            let name = label(sensor);
            findings.push(critical(format!("fan-{}", slug(&name)), format!("{name} à l'arrêt"),
                                   format!("{name} est à 0 tr/min alors que sa puce est à {chip_temperature:.0} °C ; il tournait cette semaine.")));
        }
    }
    findings
}

pub fn smart(disks: &[Smart], now: i64) -> Vec<Finding> {
    let mut findings = Vec::new();
    for disk in disks {
        let key = slug(&disk.device);
        if now - disk.read_at > SMART_MAX_AGE_S {
            findings.push(warn(format!("smart-stale-{key}"), format!("Pas de données de santé récentes pour {}", disk.device),
                               "vitals-root.timer n'a pas rafraîchi les données SMART depuis deux heures : tourne-t-il ?"));
            continue;
        }
        if disk.critical_warning != 0 {
            findings.push(critical(format!("smart-{key}"), format!("{} signale un avertissement critique", disk.device),
                                   format!("Avertissement critique SMART {:#04x} : sauvegarde maintenant et regarde `sudo smartctl -a /dev/{}`.", disk.critical_warning, disk.device)));
        }
        if disk.available_spare < disk.spare_threshold {
            findings.push(critical(format!("spare-{key}"), format!("{} n'a plus de blocs de réserve", disk.device),
                                   format!("Réserve à {} %, sous son seuil de {} % : le disque lâche.", disk.available_spare, disk.spare_threshold)));
        }
        if disk.percentage_used >= DISK_WORN_PERCENT {
            findings.push(warn(format!("worn-{key}"), format!("{} est usé à {} %", disk.device, disk.percentage_used),
                               "Son endurance en écriture est presque épuisée : prévois un remplacement."));
        }
        if disk.media_errors > disk.previous_media_errors.unwrap_or(disk.media_errors) {
            findings.push(warn(format!("media-{key}"), format!("{} a de nouvelles erreurs de support", disk.device),
                               format!("{} erreurs de support, {} avant.", disk.media_errors, disk.previous_media_errors.unwrap_or(0))));
        }
    }
    findings
}

pub fn system(facts: &Facts, now: i64) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (mount, used) in &facts.disk_usage {
        if *used >= DISK_CRITICAL_PERCENT {
            findings.push(critical(format!("full-{}", slug(mount)), format!("{mount} est plein à {used:.0} %"), "Disque presque plein : libère de la place maintenant."));
        } else if *used >= DISK_WARNING_PERCENT {
            findings.push(warn(format!("full-{}", slug(mount)), format!("{mount} est plein à {used:.0} %"), "Libère de la place avant qu'il soit plein."));
        }
    }
    if !facts.failed_units.is_empty() {
        findings.push(warn("failed-units", format!("{} service(s) en échec", facts.failed_units.len()), facts.failed_units.join(", ")));
    }
    for (category, lines) in &facts.kernel_errors {
        let (level, title) = match category.as_str() {
            "gpu" => ("crit", "Réinitialisations ou blocages du GPU"),
            "disk" => ("crit", "Erreurs d'entrée/sortie disque"),
            "hardware" => ("crit", "Erreurs matérielles (machine check)"),
            "memory" => ("warn", "Programme tué faute de mémoire"),
            _ => ("warn", "Bridage thermique"),
        };
        let detail = format!("{} message(s) du noyau en 24 h, le dernier : {}", lines.len(), lines.last().map(String::as_str).unwrap_or(""));
        findings.push(Finding { key: format!("kernel-{category}"), level, title: title.into(), detail: detail.chars().take(280).collect() });
    }
    if let Some(available) = facts.memory_available_percent.filter(|available| *available < MEMORY_LOW_PERCENT) {
        findings.push(warn("memory-low", format!("Mémoire presque pleine : {available:.0} % libre"), "Ferme quelque chose avant que le système se mette à swapper fort."));
    }
    if let Some(pressure) = facts.memory_pressure.filter(|pressure| *pressure >= MEMORY_PRESSURE) {
        findings.push(warn("memory-pressure", "Pression mémoire", format!("Les tâches ont attendu la mémoire {pressure:.0} % des 5 dernières minutes.")));
    }
    if let Some(result) = facts.backup_result.as_ref().filter(|result| *result != "success") {
        findings.push(warn("backup-failed", "Sauvegarde en échec", format!("restic-backup.service s'est terminé avec {result}.")));
    }
    // Right after boot the backup has not had its turn yet (15 minutes in).
    let settled = facts.uptime_s.is_some_and(|uptime| uptime >= BACKUP_GRACE_AFTER_BOOT_S);
    if settled && facts.last_backup.is_none_or(|finished| now - finished > BACKUP_MAX_AGE_S) {
        findings.push(warn("backup-old", "Aucune sauvegarde depuis deux jours", "restic-backup.service n'a pas réussi depuis deux jours."));
    }
    findings
}

pub fn all(minutes: &Minutes, limits: &Limits, facts: &Facts, now: i64) -> Vec<Finding> {
    let mut findings = over_limits(minutes, limits);
    for (sensor, load, name) in TRENDED {
        findings.extend(idle_trend(minutes, now, sensor, load, name));
        findings.extend(load_trend(minutes, now, sensor, load, name));
    }
    findings.extend(stopped_fans(minutes, now));
    findings.extend(smart(&facts.disks, now));
    findings.extend(system(facts, now));
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    const CPU: &str = "temp:k10temp/Tctl";
    const NOW: i64 = 100 * DAY;

    /// `days` days of minutes, `per_day` of them each day, the CPU at `temperature(day)` and the load at `load`.
    fn history(days: i64, per_day: i64, temperature: impl Fn(i64) -> f64, load: f64) -> Minutes {
        let mut minutes = Minutes::new();
        for day in 0..days {
            for minute in 0..per_day {
                let at = NOW - (days - day) * DAY + minute * 60;
                minutes.insert(at, BTreeMap::from([(CPU.to_string(), temperature(day)), ("load:cpu".to_string(), load)]));
            }
        }
        minutes
    }

    fn latest(readings: &[(&str, &[f64])]) -> Minutes {
        let count = readings[0].1.len();
        (0..count).map(|index| (NOW - (count - index) as i64 * 60,
                                readings.iter().map(|(key, values)| (key.to_string(), values[index])).collect())).collect()
    }

    #[test]
    fn three_minutes_over_the_limit_warn() {
        let findings = over_limits(&latest(&[(CPU, &[91.0, 92.0, 93.0])]), &Limits::new());
        assert_eq!(findings.len(), 1);
        assert_eq!((findings[0].key.as_str(), findings[0].level), ("hot-cpu", "warn"));
    }

    #[test]
    fn one_spike_is_no_finding() {
        assert!(over_limits(&latest(&[(CPU, &[60.0, 61.0, 95.0])]), &Limits::new()).is_empty());
    }

    #[test]
    fn the_drivers_critical_limit_is_critical_at_once() {
        let limits = Limits::from([("temp:amdgpu/edge".to_string(), (Some(100.0), Some(110.0)))]);
        let findings = over_limits(&latest(&[("temp:amdgpu/edge", &[60.0, 60.0, 111.0])]), &limits);
        assert_eq!(findings[0].level, "crit");
    }

    #[test]
    fn an_idle_temperature_that_climbed_five_degrees_in_two_weeks_warns() {
        let minutes = history(28, 60, |day| if day < 14 { 40.0 } else { 46.0 }, 5.0);
        let finding = idle_trend(&minutes, NOW, CPU, "load:cpu", "cpu").expect("no finding");
        assert_eq!(finding.key, "idle-cpu");
        assert!(finding.title.contains("+6.0"), "{}", finding.title);
    }

    #[test]
    fn a_steady_idle_temperature_is_fine() {
        assert_eq!(idle_trend(&history(28, 60, |_| 40.0, 5.0), NOW, CPU, "load:cpu", "cpu"), None);
    }

    #[test]
    fn busy_minutes_do_not_count_as_idle() {
        // Two weeks of games at 80 °C must not look like a hotter idle.
        let mut minutes = history(28, 60, |_| 40.0, 5.0);
        minutes.extend(history(14, 600, |_| 80.0, 90.0).into_iter().map(|(at, readings)| (at + 3600, readings)));
        assert_eq!(idle_trend(&minutes, NOW, CPU, "load:cpu", "cpu"), None);
    }

    #[test]
    fn too_little_history_says_nothing() {
        assert_eq!(idle_trend(&history(10, 60, |day| 40.0 + day as f64, 5.0), NOW, CPU, "load:cpu", "cpu"), None);
    }

    #[test]
    fn a_warmer_cpu_under_the_same_load_warns() {
        let mut minutes = history(28, 30, |day| if day < 14 { 70.0 } else { 77.0 }, 90.0);
        minutes.retain(|at, _| *at >= NOW - 7 * DAY || *at < NOW - 21 * DAY);
        assert_eq!(load_trend(&minutes, NOW, CPU, "load:cpu", "cpu").map(|finding| finding.key), Some("load-cpu".into()));
    }

    #[test]
    fn a_pump_that_stopped_while_warm_is_critical() {
        let minutes = latest(&[("fan:it8696/fan2", &[1800.0, 0.0, 0.0, 0.0]), (CPU, &[65.0, 66.0, 67.0, 68.0])]);
        let findings = stopped_fans(&minutes, NOW);
        assert_eq!(findings[0].level, "crit");
    }

    #[test]
    fn a_gpu_fan_resting_while_cool_is_fine() {
        let minutes = latest(&[("fan:amdgpu/fan1", &[900.0, 0.0, 0.0, 0.0]), ("temp:amdgpu/edge", &[50.0, 45.0, 44.0, 43.0])]);
        assert!(stopped_fans(&minutes, NOW).is_empty());
    }

    fn disk() -> Smart {
        Smart { device: "nvme0n1".into(), critical_warning: 0, percentage_used: 3, available_spare: 100, spare_threshold: 10,
                media_errors: 0, previous_media_errors: Some(0), read_at: NOW }
    }

    #[test]
    fn a_healthy_disk_is_fine() {
        assert!(smart(&[disk()], NOW).is_empty());
    }

    #[test]
    fn a_smart_critical_warning_is_critical() {
        assert_eq!(smart(&[Smart { critical_warning: 4, ..disk() }], NOW)[0].level, "crit");
    }

    #[test]
    fn new_media_errors_warn() {
        assert_eq!(smart(&[Smart { media_errors: 2, previous_media_errors: Some(1), ..disk() }], NOW)[0].key, "media-nvme0n1");
    }

    #[test]
    fn stale_smart_data_says_the_timer_stopped() {
        assert_eq!(smart(&[Smart { read_at: NOW - 3 * 3600, ..disk() }], NOW)[0].key, "smart-stale-nvme0n1");
    }

    #[test]
    fn failed_units_and_gpu_resets_are_reported() {
        let facts = Facts { failed_units: vec!["restic-backup.service".into()],
                            kernel_errors: BTreeMap::from([("gpu".to_string(), vec!["amdgpu: GPU reset begin!".to_string()])]), ..Facts::default() };
        let keys: Vec<String> = system(&facts, NOW).into_iter().map(|finding| finding.key).collect();
        assert_eq!(keys, ["failed-units", "kernel-gpu"]);
    }

    #[test]
    fn an_old_backup_warns() {
        let facts = Facts { last_backup: Some(NOW - 3 * DAY), uptime_s: Some(7200), ..Facts::default() };
        assert_eq!(system(&facts, NOW)[0].key, "backup-old");
    }

    #[test]
    fn right_after_boot_the_backup_is_not_judged() {
        let facts = Facts { last_backup: Some(NOW - 3 * DAY), uptime_s: Some(600), ..Facts::default() };
        assert!(system(&facts, NOW).is_empty());
    }

    #[test]
    fn a_failed_backup_warns() {
        let facts = Facts { backup_result: Some("exit-code".into()), last_backup: Some(NOW), uptime_s: Some(7200), ..Facts::default() };
        assert_eq!(system(&facts, NOW)[0].key, "backup-failed");
    }

    #[test]
    fn a_pump_that_lost_a_fifth_of_its_idle_speed_warns() {
        let mut minutes = Minutes::new();
        for minute in 0..200 {
            minutes.insert(NOW - 25 * DAY + minute * 60, BTreeMap::from([("fan:it8696/fan5".to_string(), 2800.0), ("load:cpu".to_string(), 5.0)]));
            minutes.insert(NOW - 2 * DAY + minute * 60, BTreeMap::from([("fan:it8696/fan5".to_string(), 2200.0), ("load:cpu".to_string(), 5.0)]));
        }
        assert_eq!(pump_trend(&minutes, NOW, "fan:it8696/fan5", "load:cpu").map(|finding| finding.key), Some("pump-slower".into()));
    }

    #[test]
    fn a_steady_pump_is_fine() {
        let mut minutes = Minutes::new();
        for minute in 0..200 {
            minutes.insert(NOW - 25 * DAY + minute * 60, BTreeMap::from([("fan:it8696/fan5".to_string(), 2800.0), ("load:cpu".to_string(), 5.0)]));
            minutes.insert(NOW - 2 * DAY + minute * 60, BTreeMap::from([("fan:it8696/fan5".to_string(), 2750.0), ("load:cpu".to_string(), 5.0)]));
        }
        assert_eq!(pump_trend(&minutes, NOW, "fan:it8696/fan5", "load:cpu"), None);
    }

    #[test]
    fn disk_space_warns_at_85_and_escalates_at_95_with_the_same_key() {
        for (used, expected) in [(84.0, None), (85.0, Some("warn")), (94.0, Some("warn")), (95.0, Some("crit")), (100.0, Some("crit"))] {
            let facts = Facts { disk_usage: vec![("/".into(), used)], ..Default::default() };
            let found = system(&facts, NOW);
            let space = found.iter().find(|finding| finding.key == "full-");
            assert_eq!(space.map(|finding| finding.level), expected, "used={used}");
        }
    }

    #[test]
    fn space_threshold_takes_over_from_the_growth_forecast_at_85_percent() {
        for (used, expected) in [(84.0, true), (85.0, false), (95.0, false)] {
            let history = [(NOW - 5 * DAY, used - 10.0), (NOW, used)];
            assert_eq!(filling("/", &history).is_some(), expected, "used={used}");
        }
    }

    #[test]
    fn a_disk_gaining_two_points_a_day_from_70_is_full_in_about_15_days() {
        let history: Vec<(i64, f64)> = (0..=5).map(|day| (NOW - (5 - day) * DAY, 60.0 + 2.0 * day as f64)).collect();
        let days = days_until_full(&history).unwrap();
        assert!((days - 15.0).abs() < 0.01, "{days}");
        assert_eq!(filling("/", &history).map(|finding| finding.key), Some("filling-root".into()));
    }

    #[test]
    fn a_disk_that_does_not_grow_has_no_forecast() {
        let history: Vec<(i64, f64)> = (0..=5).map(|day| (NOW - (5 - day) * DAY, 60.0)).collect();
        assert_eq!(days_until_full(&history), None);
    }

    #[test]
    fn a_day_of_history_is_too_short_to_forecast() {
        assert_eq!(days_until_full(&[(NOW - 3600, 50.0), (NOW, 60.0)]), None);
    }

    #[test]
    fn alert_keys_are_notice_ids() {
        assert_eq!(slug("RAM 0053 / Board 1"), "ram-0053-board-1");
    }
}
