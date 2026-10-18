//! The machine in plain words: one headline, then a sentence per area (cooling, disk, system,
//! backup, upkeep), each saying what it means and what to do when something drifts.

use crate::care::{self, Due};
use crate::store::Alert;

/// The pump's header on this board (B850 AORUS ELITE WIFI7, it87 driver): the one that spins.
pub const PUMP: &str = "fan:it8696/fan5";
/// The idle temperature may wander this much in two weeks; past it, dusting is near.
const SLIGHT_RISE: f64 = 2.0;
const CLEAR_RISE: f64 = 5.0;

#[derive(Default)]
pub struct Machine {
    pub alerts: Vec<Alert>,
    /// Median idle CPU temperature of the last two weeks, and of the two before.
    pub cpu_idle: (Option<f64>, Option<f64>),
    pub cpu_now: Option<f64>,
    pub pump_rpm: Option<f64>,
    /// device, % of rated writes used, healthy (no critical warning, spare above threshold)
    pub disk: Option<(String, u64, bool)>,
    /// The fullest file system: mount, % used.
    pub fullest: Option<(String, f64)>,
    pub failed_units: Vec<String>,
    pub kernel_problems: Vec<String>,
    pub backup_age_days: Option<i64>,
    /// Each upkeep task's title and due state.
    pub care: Vec<(&'static str, Due)>,
}

#[derive(Debug, PartialEq)]
pub struct Sentence {
    pub topic: &'static str,
    pub text: String,
    /// "ok", "warn" or "crit"
    pub level: &'static str,
}

fn sentence(topic: &'static str, level: &'static str, text: impl Into<String>) -> Sentence {
    Sentence { topic, text: text.into(), level }
}

pub fn headline(machine: &Machine) -> (String, &'static str) {
    if let Some(alert) = machine.alerts.iter().find(|alert| alert.level == "crit") {
        return (format!("Something needs you now: {}.", alert.title), "crit");
    }
    match machine.alerts.as_slice() {
        [] => ("Your PC is healthy: nothing needs you.".into(), "ok"),
        [only] => (format!("Your PC is fine, one thing to look at: {}.", only.title), "warn"),
        many => (format!("Your PC is fine, {} things to look at: {}.", many.len(),
                         many.iter().map(|alert| alert.title.as_str()).collect::<Vec<_>>().join("; ")), "warn"),
    }
}

fn cooling(machine: &Machine) -> Sentence {
    if machine.pump_rpm == Some(0.0) {
        return sentence("Cooling", "crit", "The pump reads 0 RPM: check the water cooling now, before running anything heavy.");
    }
    let pump = machine.pump_rpm.map(|rpm| format!(", the pump turns at {rpm:.0} RPM")).unwrap_or_default();
    match machine.cpu_idle {
        (Some(recent), Some(before)) => {
            let rise = recent - before;
            if rise >= CLEAR_RISE {
                sentence("Cooling", "warn", format!("Getting warmer: the CPU idles at {recent:.0}°C, {rise:.0}°C more than two weeks ago{pump}. \
                    Dust the filters and the radiator, then check the pump; if it stays warm, the thermal paste is next."))
            } else if rise >= SLIGHT_RISE {
                sentence("Cooling", "ok", format!("Fine, slightly warmer: the CPU idles at {recent:.0}°C (+{rise:.0}°C in two weeks){pump}. \
                    Dust soon if it keeps climbing."))
            } else {
                sentence("Cooling", "ok", format!("Normal: the CPU idles at {recent:.0}°C ({:+.0}°C in two weeks){pump}.", rise))
            }
        }
        (Some(recent), None) => sentence("Cooling", "ok", format!("The CPU idles at {recent:.0}°C{pump}; two more weeks of history and the trend will show.")),
        _ => sentence("Cooling", "ok", match machine.cpu_now {
            Some(now) => format!("The CPU is at {now:.0}°C{pump}; the idle trend needs more history."),
            None => "No CPU temperature yet.".into(),
        }),
    }
}

fn disk(machine: &Machine) -> Sentence {
    let space = machine.fullest.as_ref().map(|(mount, used)| format!(", the fullest drive ({mount}) is {used:.0}% full")).unwrap_or_default();
    match &machine.disk {
        Some((device, _, false)) => sentence("Disk", "crit", format!("{device} reports a hardware problem: back up now and see the Disks tab.")),
        Some((_, worn, true)) => sentence("Disk", if machine.fullest.as_ref().is_some_and(|(_, used)| *used >= 90.0) { "warn" } else { "ok" },
                                          format!("Healthy, {worn}% of its rated writes used{space}.")),
        None => sentence("Disk", "warn", format!("No health data yet (vitals-root.timer){space}.")),
    }
}

fn system(machine: &Machine) -> Sentence {
    if !machine.kernel_problems.is_empty() {
        return sentence("System", "crit", format!("The kernel logged {}: see the System tab.", machine.kernel_problems.join(", ")));
    }
    match machine.failed_units.as_slice() {
        [] => sentence("System", "ok", "Every service runs and the kernel logged no hardware error."),
        units => sentence("System", "warn", format!("{} service{} failed ({}): see the System tab.", units.len(),
                                                     if units.len() == 1 { "" } else { "s" },
                                                     units.iter().map(|unit| unit.trim_end_matches(".service")).collect::<Vec<_>>().join(", "))),
    }
}

fn backup(machine: &Machine) -> Sentence {
    match machine.backup_age_days {
        Some(age) if age <= 2 => sentence("Backup", "ok", format!("The last good backup is {} old.", if age == 0 { "less than a day".into() } else { care::days(age) })),
        Some(age) => sentence("Backup", "warn", format!("The last good backup is {} old: check restic-backup.", care::days(age))),
        None => sentence("Backup", "warn", "No successful backup seen yet."),
    }
}

fn upkeep(machine: &Machine) -> Sentence {
    let overdue: Vec<String> = machine.care.iter().filter_map(|(title, due)| match due {
        Due::Overdue(days) => Some(format!("{} ({} late)", title.to_lowercase(), care::days(*days))),
        _ => None,
    }).collect();
    if !overdue.is_empty() {
        return sentence("Upkeep", "warn", format!("Due: {}. Press d in the Care tab once done.", overdue.join(", ")));
    }
    let unknown = machine.care.iter().filter(|(_, due)| *due == Due::Unknown).count();
    let next = machine.care.iter().filter_map(|(title, due)| match due { Due::In(days) => Some((days, title)), _ => None }).min();
    let mut text = match next {
        Some((days, title)) => format!("Nothing due; next: {} in {}.", title.to_lowercase(), care::days(*days)),
        None => String::new(),
    };
    if unknown > 0 {
        text = format!("{text}{}{unknown} task{} without a date: record when you last did them in the Care tab.",
                       if text.is_empty() { "" } else { " " }, if unknown == 1 { "" } else { "s" });
    }
    sentence("Upkeep", "ok", text)
}

pub fn sentences(machine: &Machine) -> Vec<Sentence> {
    vec![cooling(machine), disk(machine), system(machine), backup(machine), upkeep(machine)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy() -> Machine {
        Machine { cpu_idle: (Some(48.0), Some(50.0)), pump_rpm: Some(2860.0), disk: Some(("nvme0n1".into(), 2, true)),
                  fullest: Some(("/".into(), 82.0)), backup_age_days: Some(0), care: vec![("Dust the filters and fans", Due::In(60))],
                  ..Machine::default() }
    }

    #[test]
    fn a_healthy_machine_says_so_in_one_line() {
        assert_eq!(headline(&healthy()), ("Your PC is healthy: nothing needs you.".into(), "ok"));
        assert!(sentences(&healthy()).iter().all(|line| line.level == "ok"));
    }

    #[test]
    fn a_warmer_idle_says_to_dust_and_check_the_pump() {
        let machine = Machine { cpu_idle: (Some(56.0), Some(50.0)), ..healthy() };
        let line = &sentences(&machine)[0];
        assert_eq!(line.level, "warn");
        assert!(line.text.contains("Dust the filters and the radiator"), "{}", line.text);
    }

    #[test]
    fn a_stopped_pump_is_critical() {
        assert_eq!(sentences(&Machine { pump_rpm: Some(0.0), ..healthy() })[0].level, "crit");
    }

    #[test]
    fn failed_services_are_named_without_their_suffix() {
        let machine = Machine { failed_units: vec!["security-check.service".into(), "vdirsyncer.service".into()], ..healthy() };
        assert_eq!(sentences(&machine)[2].text, "2 services failed (security-check, vdirsyncer): see the System tab.");
    }

    #[test]
    fn overdue_upkeep_is_named_with_how_late_it_is() {
        let machine = Machine { care: vec![("Dust the filters and fans", Due::Overdue(12))], ..healthy() };
        assert_eq!(sentences(&machine)[4].text, "Due: dust the filters and fans (12 days late). Press d in the Care tab once done.");
    }

    #[test]
    fn upkeep_without_dates_asks_for_them() {
        let machine = Machine { care: vec![("Dust the filters and fans", Due::Unknown)], ..healthy() };
        assert!(sentences(&machine)[4].text.contains("1 task without a date"));
    }

    #[test]
    fn several_problems_are_listed_in_the_headline() {
        let alert = |title: &str| Alert { key: title.into(), level: "warn".into(), title: title.into(), detail: String::new(), since: 0, notified: true };
        let machine = Machine { alerts: vec![alert("2 failed service(s)"), alert("Time to dust the filters and fans")], ..healthy() };
        assert_eq!(headline(&machine).0, "Your PC is fine, 2 things to look at: 2 failed service(s); Time to dust the filters and fans.");
    }
}
