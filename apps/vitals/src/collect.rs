//! One collection (vitals-collect.timer, every minute): read everything, keep it, check it, and
//! tell Theo what changed: an OSD banner per new problem, the bar's dot for as long as any lasts.

use crate::care;
use crate::checks::{self, DAY, Finding, TRENDED};
use crate::facts;
use crate::games;
use crate::journal;
use crate::summary;
use crate::sensors::{self, Kind};
use crate::store::{Alert, KEEP_SAMPLES_S, Store};
use serde_json::json;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

const WAYBAR_SIGNAL: &str = "-RTMIN+17";
const NOTICE_TITLE: &str = "PC";

pub fn status_path() -> PathBuf {
    cockpit::paths::runtime_dir().join("vitals.json")
}

/// Text for a banner: the OSD refuses control characters, tmux formats and long text.
fn banner_text(text: &str) -> String {
    let clean: String = text.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
    clean.replace("#(", "# (").replace("#{", "# {").chars().take(280).collect()
}

fn notice_id(key: &str) -> String {
    format!("vitals-{key}")
}

fn notify(alert: &Alert) -> bool {
    let notice = json!({"id": notice_id(&alert.key), "title": banner_text(&format!("{NOTICE_TITLE} · {}", alert.title)), "message": banner_text(&alert.detail)});
    cockpit::approval::send_host(&format!("notice {notice}"))
}

/// Brings the alerts in line with the findings. New problems (or ones that turned critical) get a
/// banner unless Theo is gaming; then they wait for the next collection. Solved ones lose theirs.
pub fn apply(store: &Store, findings: &[Finding], now: i64, is_gaming: bool, send: &mut dyn FnMut(&str, Option<&Alert>) -> bool) -> rusqlite::Result<bool> {
    let previous: BTreeMap<String, Alert> = store.alerts()?.into_iter().map(|alert| (alert.key.clone(), alert)).collect();
    let mut changed = false;
    for finding in findings {
        let mut alert = match previous.get(&finding.key) {
            Some(known) => Alert { level: finding.level.into(), title: finding.title.clone(), detail: finding.detail.clone(),
                                   notified: known.notified && !(known.level == "warn" && finding.level == "crit"), ..known.clone() },
            None => {
                changed = true;
                Alert { key: finding.key.clone(), level: finding.level.into(), title: finding.title.clone(), detail: finding.detail.clone(),
                        since: now, notified: false }
            }
        };
        if !alert.notified && !is_gaming {
            alert.notified = send(&alert.key, Some(&alert));
        }
        store.save_alert(&alert)?;
    }
    for (key, alert) in &previous {
        if !findings.iter().any(|finding| &finding.key == key) {
            store.drop_alert(key)?;
            store.save_ended_alert(alert, now)?;
            if alert.notified {
                send(key, None);
            }
            changed = true;
        }
    }
    Ok(changed)
}

/// What the bar shows: the worst level and every alert's title.
pub fn write_status(store: &Store, now: i64) -> std::io::Result<()> {
    let alerts = store.alerts().unwrap_or_default();
    let level = if alerts.iter().any(|alert| alert.level == "crit") { "crit" } else if alerts.is_empty() { "ok" } else { "warn" };
    let titles: Vec<&str> = alerts.iter().map(|alert| alert.title.as_str()).collect();
    cockpit::util::write_json(&status_path(), &json!({"level": level, "alerts": titles, "at": now}))
}

fn record_sensors(store: &Store, now: i64) -> rusqlite::Result<()> {
    let root = sensors::sys_root();
    for reading in sensors::read_hwmon(&root) {
        store.record(now, &reading.key, reading.value)?;
        if reading.kind == Kind::Temperature {
            store.set_limits(&reading.key, reading.high, reading.critical)?;
        }
    }
    if let Some(counters) = sensors::cpu_counters(&root) {
        let previous = store.state("cpu_counters")?.and_then(|text| {
            let (busy, total) = text.split_once(' ')?;
            Some((busy.parse().ok()?, total.parse().ok()?))
        });
        if let Some(load) = previous.and_then(|before| sensors::cpu_load(before, counters)) {
            store.record(now, "load:cpu", load)?;
        }
        store.set_state("cpu_counters", &format!("{} {}", counters.0, counters.1))?;
    }
    if let Some(load) = sensors::gpu_load(&root) {
        store.record(now, "load:gpu", load)?;
    }
    for (key, value) in sensors::gpu_details(&root) {
        store.record(now, &key, value)?;
    }
    store.forget_before(now - KEEP_SAMPLES_S)
}

/// Facts plus what only the history knows: media errors at the last look, the last good backup.
fn facts_with_history(store: &Store) -> rusqlite::Result<facts::Facts> {
    let mut facts = facts::gather(&sensors::sys_root());
    for disk in &mut facts.disks {
        let name = format!("media_errors {}", disk.device);
        disk.previous_media_errors = store.state(&name)?.and_then(|count| count.parse().ok());
        store.set_state(&name, &disk.media_errors.to_string())?;
    }
    let remembered: Option<i64> = store.state("last_backup")?.and_then(|stamp| stamp.parse().ok());
    facts.last_backup = facts.last_backup.max(remembered);
    if let Some(finished) = facts.last_backup {
        store.set_state("last_backup", &finished.to_string())?;
    }
    Ok(facts)
}

pub fn findings(store: &Store, facts: &facts::Facts, now: i64) -> rusqlite::Result<Vec<Finding>> {
    let mut trended: Vec<&str> = TRENDED.iter().flat_map(|(sensor, load, _)| [*sensor, *load]).collect();
    trended.dedup();
    let mut minutes = store.minutes(now - 29 * DAY, &trended)?;
    for (at, readings) in store.minutes(now - 7 * DAY, &["fan:%"])? {
        minutes.entry(at).or_default().extend(readings);
    }
    for (at, readings) in store.minutes(now - 5 * 60, &["temp:%"])? {
        minutes.entry(at).or_default().extend(readings);
    }
    let mut found = checks::all(&minutes, &store.limits()?, facts, now);
    let pump_minutes = store.minutes(now - 29 * DAY, &[summary::PUMP, "load:cpu"])?;
    found.extend(checks::pump_trend(&pump_minutes, now, summary::PUMP, "load:cpu"));
    for (mount, _) in &facts.disk_usage {
        let history: Vec<(i64, f64)> = store.hours(&disk_key(mount), now - 14 * DAY)?.into_iter().map(|(hour, _, mean, _)| (hour, mean)).collect();
        found.extend(checks::filling(mount, &history));
    }
    found.extend(care_findings(store, now));
    Ok(found)
}

/// The history key of a mount's usage.
pub fn disk_key(mount: &str) -> String {
    format!("disk:{mount}")
}

/// Upkeep past its interval, like any other problem. Never-recorded tasks raise nothing.
pub fn care_findings(store: &Store, now: i64) -> Vec<Finding> {
    care::TASKS.iter().filter(|task| matches!(care::due(task, care::last_done(store, task), now), care::Due::Overdue(_)))
        .map(|task| Finding { key: format!("care-{}", task.id), level: "warn", title: format!("Il est temps de {}", task.title.to_lowercase()),
                              detail: format!("{} {}", task.why, task.how) }).collect()
}

/// Announce an advisory report once, acknowledging only successful delivery.
pub fn announce_disk_review(store: &Store, path: &Path, now: i64, is_gaming: bool,
                            send: &mut dyn FnMut(&str, &str) -> bool) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if !metadata.is_file() {
        return Err("report must be a regular file, not a symlink".into());
    }
    let mut content = Vec::new();
    std::fs::File::open(path).map_err(|error| error.to_string())?.take(1_048_577)
        .read_to_end(&mut content).map_err(|error| error.to_string())?;
    if content.len() > 1_048_576 {
        return Err("report exceeds 1 MiB".into());
    }
    let report: serde_json::Value = serde_json::from_slice(&content).map_err(|error| error.to_string())?;
    if report["schema_version"].as_u64() != Some(1) || report["advisory_only"].as_bool() != Some(true)
        || !["games", "checkouts", "errors"].iter().all(|key| report[*key].as_array().is_some_and(|items| items.len() <= 20)) {
        return Err("invalid advisory report schema".into());
    }
    let completed = report["completed_at"].as_i64().filter(|stamp| *stamp > 0 && *stamp <= now)
        .ok_or("invalid report completion time")?;
    let complete = report["complete"].as_bool().ok_or("missing report completeness")?;
    let previous = store.state("disk_review_notified_at").map_err(|error| error.to_string())?
        .map(|stamp| stamp.parse::<i64>()).transpose().map_err(|error| error.to_string())?;
    if is_gaming || previous.is_some_and(|stamp| stamp >= completed) {
        return Ok(());
    }
    let detail = if complete {
        format!("Revue mensuelle disponible : {}. Toute suppression demande ton accord.", path.display())
    } else {
        format!("Revue mensuelle partielle : {}. Certains chemins n'ont pas pu être mesurés. Aucune suppression automatique.", path.display())
    };
    if send("Espace disque : revue mensuelle", &detail) {
        store.set_state("disk_review_notified_at", &completed.to_string()).map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn run() -> Result<(), String> {
    let store = Store::open().map_err(|error| format!("cannot open the history: {error}"))?;
    let now = cockpit::util::now_secs();
    record_sensors(&store, now).map_err(|error| format!("cannot record: {error}"))?;
    let facts = facts_with_history(&store).map_err(|error| format!("cannot read the history: {error}"))?;
    for (mount, used) in &facts.disk_usage {
        store.record(now, &disk_key(mount), *used).map_err(|error| format!("cannot record: {error}"))?;
    }
    journal::record(&store, &sensors::sys_root(), now).map_err(|error| format!("cannot keep the journal: {error}"))?;
    games::follow(&store, now).map_err(|error| format!("cannot follow the game: {error}"))?;
    let found = findings(&store, &facts, now).map_err(|error| format!("cannot check: {error}"))?;
    let mut send = |key: &str, alert: Option<&Alert>| match alert {
        Some(alert) => notify(alert),
        None => cockpit::approval::send_host(&format!("notice-clear {}", notice_id(key))),
    };
    let changed = apply(&store, &found, now, cockpit::util::gaming(), &mut send).map_err(|error| format!("cannot keep the alerts: {error}"))?;
    write_status(&store, now).map_err(|error| format!("cannot write {}: {error}", status_path().display()))?;
    if changed {
        let _ = std::process::Command::new("pkill").args(["-x", WAYBAR_SIGNAL, "waybar"]).status();
    }
    let review = cockpit::paths::state_home().join("disk-maintenance/latest.json");
    announce_disk_review(&store, &review, now, cockpit::util::gaming(), &mut |title, detail| {
        notify(&Alert { key: "disk-review".into(), level: "warn".into(), title: title.into(), detail: detail.into(),
                        since: now, notified: false })
    }).map_err(|error| format!("disk review: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(key: &str, level: &'static str) -> Finding {
        Finding { key: key.into(), level, title: format!("{key} title"), detail: "detail".into() }
    }

    #[test]
    fn a_new_problem_gets_one_banner() {
        let store = Store::in_memory().unwrap();
        let mut sent = Vec::new();
        let mut send = |key: &str, alert: Option<&Alert>| { sent.push((key.to_string(), alert.is_some())); true };
        apply(&store, &[finding("hot-cpu", "warn")], 100, false, &mut send).unwrap();
        apply(&store, &[finding("hot-cpu", "warn")], 160, false, &mut send).unwrap();
        assert_eq!(sent, [("hot-cpu".to_string(), true)]);
        assert_eq!(store.alerts().unwrap()[0].since, 100);
    }

    #[test]
    fn while_gaming_the_banner_waits_for_the_next_collection() {
        let store = Store::in_memory().unwrap();
        let sent = std::cell::RefCell::new(Vec::new());
        let mut send = |key: &str, _: Option<&Alert>| { sent.borrow_mut().push(key.to_string()); true };
        apply(&store, &[finding("hot-cpu", "warn")], 100, true, &mut send).unwrap();
        assert!(sent.borrow().is_empty());
        apply(&store, &[finding("hot-cpu", "warn")], 160, false, &mut send).unwrap();
        assert_eq!(*sent.borrow(), ["hot-cpu"]);
    }

    #[test]
    fn a_solved_problem_clears_its_banner() {
        let store = Store::in_memory().unwrap();
        let mut sent = Vec::new();
        let mut send = |key: &str, alert: Option<&Alert>| { sent.push((key.to_string(), alert.is_some())); true };
        apply(&store, &[finding("hot-cpu", "warn")], 100, false, &mut send).unwrap();
        let changed = apply(&store, &[], 160, false, &mut send).unwrap();
        assert!(changed);
        assert_eq!(store.ended_alerts(0).unwrap()[0].until, 160);
        assert_eq!(sent.last().unwrap(), &("hot-cpu".to_string(), false));
        assert!(store.alerts().unwrap().is_empty());
    }

    #[test]
    fn a_warning_that_turns_critical_is_announced_again() {
        let store = Store::in_memory().unwrap();
        let mut count = 0;
        let mut send = |_: &str, _: Option<&Alert>| { count += 1; true };
        apply(&store, &[finding("hot-cpu", "warn")], 100, false, &mut send).unwrap();
        apply(&store, &[finding("hot-cpu", "crit")], 160, false, &mut send).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn a_banner_the_osd_did_not_take_is_tried_again() {
        let store = Store::in_memory().unwrap();
        let mut tries = 0;
        let mut send = |_: &str, _: Option<&Alert>| { tries += 1; tries > 1 };
        apply(&store, &[finding("hot-cpu", "warn")], 100, false, &mut send).unwrap();
        apply(&store, &[finding("hot-cpu", "warn")], 160, false, &mut send).unwrap();
        apply(&store, &[finding("hot-cpu", "warn")], 220, false, &mut send).unwrap();
        assert_eq!(tries, 2);
    }

    #[test]
    fn overdue_upkeep_becomes_an_alert_and_unrecorded_upkeep_does_not() {
        let store = Store::in_memory().unwrap();
        let now = 1000 * care::DAY;
        care::mark_done(&store, &care::TASKS[0], now - 100 * care::DAY).unwrap();
        let keys: Vec<String> = care_findings(&store, now).into_iter().map(|finding| finding.key).collect();
        assert_eq!(keys, ["care-dust"]);
    }

    #[test]
    fn banner_text_loses_what_the_osd_refuses() {
        assert_eq!(banner_text("a\nb #(id) #{x}"), "a b # (id) # {x}");
    }
}
