//! What happened to the machine, for the Journal tab: crashes, programs killed for memory, GPU
//! hangs and other kernel errors (read from what vitals-root leaves in /var/lib/vitals), and hard
//! stops (a freeze, a power cut, the power button held), which the collector notices itself.
//!
//! A hard stop is a boot whose previous one never said goodbye: vitals-shutdown.service (a user
//! unit) records the boot when it starts and again when it stops cleanly at shutdown.

use crate::facts::{self, root_data};
use crate::store::{Event, Store};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// `journalctl -o short-iso`'s first field: 2026-10-18T21:42:05+0200.
fn time_of(line: &str) -> Option<i64> {
    let stamp = line.split_whitespace().next()?;
    chrono::DateTime::parse_from_str(stamp, "%Y-%m-%dT%H:%M:%S%z").ok().map(|time| time.timestamp())
}

/// The message without its time, host and sender.
fn message_of(line: &str) -> String {
    let message = line.split_once("]: ").or_else(|| line.split_once(": ")).map(|(_, message)| message).unwrap_or(line);
    message.trim().chars().take(200).collect()
}

/// Kernel error lines, each under the category facts.rs sorts it into.
pub fn kernel_events(log: &str) -> Vec<Event> {
    log.lines().filter_map(|line| {
        let kind = facts::kernel_category(line)?;
        Some(Event { at: time_of(line)?, kind: kind.to_string(), text: message_of(line) })
    }).collect()
}

/// systemd-oomd's kills (`journalctl -u systemd-oomd --grep=Killed`).
pub fn oomd_events(log: &str) -> Vec<Event> {
    log.lines().filter_map(|line| Some(Event { at: time_of(line)?, kind: "memory".into(), text: message_of(line) })).collect()
}

fn signal_name(number: i64) -> String {
    match number {
        4 => "SIGILL".into(),
        5 => "SIGTRAP".into(),
        6 => "SIGABRT".into(),
        7 => "SIGBUS".into(),
        8 => "SIGFPE".into(),
        11 => "SIGSEGV".into(),
        other => format!("signal {other}"),
    }
}

/// `coredumpctl list --json=short`: one event per crashed program.
pub fn crash_events(json: &str) -> Vec<Event> {
    let Ok(Value::Array(dumps)) = serde_json::from_str::<Value>(json) else { return Vec::new() };
    dumps.iter().filter_map(|dump| {
        let at = dump.get("time")?.as_i64()? / 1_000_000;
        let program = dump.get("exe").and_then(Value::as_str).and_then(|exe| exe.rsplit('/').next()).unwrap_or("?");
        let signal = dump.get("sig").and_then(Value::as_i64).map(signal_name).unwrap_or_default();
        Some(Event { at, kind: "crash".into(), text: format!("{program} ({signal})") })
    }).collect()
}

pub fn boot_id(root: &Path) -> Option<String> {
    fs::read_to_string(root.join("proc/sys/kernel/random/boot_id")).ok().map(|id| id.trim().to_string()).filter(|id| !id.is_empty())
}

/// A new boot after one that was watched and never stopped cleanly: a hard stop, dated by the
/// last collection of that boot.
pub fn check_boot(store: &Store, current: &str, now: i64) -> rusqlite::Result<Option<Event>> {
    let previous = store.state("boot_id")?;
    let last_seen: Option<i64> = store.state("last_seen")?.and_then(|stamp| stamp.parse().ok());
    store.set_state("boot_id", current)?;
    store.set_state("last_seen", &now.to_string())?;
    let Some(previous) = previous.filter(|previous| previous != current) else { return Ok(None) };
    let watched = store.state("watched_boot")?.as_deref() == Some(previous.as_str());
    let clean = store.state("clean_shutdown")?.as_deref() == Some(previous.as_str());
    Ok((watched && !clean).then(|| Event {
        at: last_seen.unwrap_or(now),
        kind: "unclean".into(),
        text: "le PC s'est arrêté sans extinction propre (gel, coupure de courant ou bouton maintenu)".into(),
    }))
}

/// vitals-shutdown.service: `watch` when the session starts, `clean` when it stops at shutdown.
pub fn mark(store: &Store, root: &Path, moment: &str) -> Result<(), String> {
    let boot = boot_id(root).ok_or("no boot id in /proc")?;
    let name = match moment {
        "watch" => "watched_boot",
        "clean" => "clean_shutdown",
        other => return Err(format!("unknown moment {other}: watch or clean")),
    };
    store.set_state(name, &boot).map_err(|error| error.to_string())
}

/// Every source, once a minute; the store keeps each event once.
pub fn record(store: &Store, root: &Path, now: i64) -> rusqlite::Result<()> {
    let data = root_data();
    let read = |name: &str| fs::read_to_string(data.join(name)).unwrap_or_default();
    let mut events = kernel_events(&read("kernel-errors.log"));
    events.extend(oomd_events(&read("oomd.log")));
    events.extend(crash_events(&read("crashes.json")));
    if let Some(boot) = boot_id(root) {
        events.extend(check_boot(store, &boot, now)?);
    }
    for event in &events {
        store.add_event(event)?;
    }
    Ok(())
}

/// The kind as the Journal and the summary name it.
pub fn kind_name(kind: &str) -> &'static str {
    match kind {
        "crash" => "Plantage",
        "unclean" => "Arrêt brutal",
        "memory" => "Mémoire épuisée",
        "gpu" => "Blocage du GPU",
        "disk" => "Erreur disque",
        "hardware" => "Erreur matérielle",
        "thermal" => "Bridage thermique",
        _ => "Événement",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gpu_reset_in_the_kernel_log_is_an_event_with_its_time() {
        let events = kernel_events("2026-10-18T21:42:05+0200 desktop kernel: amdgpu 0000:03:00.0: amdgpu: GPU reset begin!\n");
        assert_eq!(events.len(), 1);
        assert_eq!((events[0].at, events[0].kind.as_str()), (1_792_352_525, "gpu"));
        assert_eq!(events[0].text, "amdgpu 0000:03:00.0: amdgpu: GPU reset begin!");
    }

    #[test]
    fn a_kernel_memory_kill_is_a_memory_event() {
        let events = kernel_events("2026-10-18T21:42:05+0200 desktop kernel: Out of memory: Killed process 4242 (ArcRaiders.exe)\n");
        assert_eq!(events[0].kind, "memory");
    }

    #[test]
    fn a_crash_names_the_program_and_its_signal() {
        let events = crash_events(r#"[{"time": 1792352525000000, "pid": 42, "uid": 1000, "sig": 11, "exe": "/usr/bin/hyprland", "corefile": "present"}]"#);
        assert_eq!(events[0], Event { at: 1_792_352_525, kind: "crash".into(), text: "hyprland (SIGSEGV)".into() });
    }

    #[test]
    fn no_crash_file_means_no_crash() {
        assert!(crash_events("").is_empty());
    }

    #[test]
    fn a_boot_after_a_clean_shutdown_is_quiet() {
        let store = Store::in_memory().unwrap();
        check_boot(&store, "first", 100).unwrap();
        store.set_state("watched_boot", "first").unwrap();
        store.set_state("clean_shutdown", "first").unwrap();
        assert_eq!(check_boot(&store, "second", 5000).unwrap(), None);
    }

    #[test]
    fn a_boot_after_a_hard_stop_is_dated_by_the_last_collection() {
        let store = Store::in_memory().unwrap();
        check_boot(&store, "first", 100).unwrap();
        store.set_state("watched_boot", "first").unwrap();
        check_boot(&store, "first", 160).unwrap();
        let event = check_boot(&store, "second", 5000).unwrap().expect("no hard stop");
        assert_eq!((event.at, event.kind.as_str()), (160, "unclean"));
    }

    #[test]
    fn a_boot_nobody_watched_is_not_judged() {
        let store = Store::in_memory().unwrap();
        check_boot(&store, "first", 100).unwrap();
        assert_eq!(check_boot(&store, "second", 5000).unwrap(), None);
    }
}
