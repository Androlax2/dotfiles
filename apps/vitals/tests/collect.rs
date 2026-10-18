//! `vitals collect` and `vitals status` end to end, on a made-up /sys and root data.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Machine {
    root: PathBuf,
}

impl Machine {
    fn new() -> Machine {
        let root = std::env::temp_dir().join(format!("vitals-e2e-{}", cockpit::util::token_hex(6)));
        let cpu = root.join("sys/sys/class/hwmon/hwmon2");
        fs::create_dir_all(&cpu).unwrap();
        fs::write(cpu.join("name"), "k10temp\n").unwrap();
        fs::write(cpu.join("temp1_label"), "Tctl\n").unwrap();
        fs::write(cpu.join("temp1_input"), "52500\n").unwrap();
        fs::create_dir_all(root.join("sys/proc")).unwrap();
        fs::write(root.join("sys/proc/stat"), "cpu  100 0 100 800 0 0 0 0 0 0\n").unwrap();
        fs::write(root.join("sys/proc/uptime"), "60.0 100.0\n").unwrap();
        for folder in ["state", "run", "root-data/smart", "bin"] {
            fs::create_dir_all(root.join(folder)).unwrap();
        }
        // The host's services and disks stay out of it: systemctl and df answer nothing wrong.
        for (name, body) in [("systemctl", ""), ("df", "echo 'Use% Mounted on'; echo ' 40% /'"), ("hyprctl", "")] {
            let shim = root.join("bin").join(name);
            let writer = Command::new("sh").args(["-c", "cat > \"$1\" && chmod 755 \"$1\"", "sh"]).arg(&shim)
                .stdin(std::process::Stdio::piped()).spawn().unwrap();
            use std::io::Write;
            writer.stdin.as_ref().unwrap().write_all(format!("#!/bin/sh\n{body}\n").as_bytes()).unwrap();
            assert!(writer.wait_with_output().unwrap().status.success());
        }
        Machine { root }
    }

    fn vitals(&self, arguments: &[&str]) -> std::process::Output {
        let path = |name: &str| self.root.join(name);
        Command::new(env!("CARGO_BIN_EXE_vitals")).args(arguments)
            .env("VITALS_SYS_ROOT", path("sys")).env("VITALS_DIR", path("state")).env("XDG_RUNTIME_DIR", path("run"))
            .env("VITALS_ROOT_DATA", path("root-data")).env("HOME", path("state"))
            .env("PATH", format!("{}:/usr/bin:/bin", path("bin").display()))
            .output().unwrap()
    }

    fn status(&self) -> Value {
        serde_json::from_str(&String::from_utf8_lossy(&self.vitals(&["status"]).stdout)).unwrap()
    }

    fn smart(&self, critical_warning: u64) {
        fs::write(self.root.join("root-data/smart/nvme0n1.json"), format!(
            r#"{{"nvme_smart_health_information_log": {{"critical_warning": {critical_warning}, "available_spare": 100,
                "available_spare_threshold": 5, "percentage_used": 2, "media_errors": 0}}}}"#)).unwrap();
    }
}

impl Drop for Machine {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn exists(path: &Path) -> bool {
    path.exists()
}

#[test]
fn a_healthy_machine_keeps_its_readings_and_shows_no_chip() {
    let machine = Machine::new();
    machine.smart(0);
    let output = machine.vitals(&["collect"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(exists(&machine.root.join("state/vitals.sqlite")));
    let status = machine.status();
    assert_eq!((status["class"].as_str(), status["text"].as_str()), (Some("ok"), Some("")));
}

#[test]
fn a_failing_disk_shows_a_red_chip_at_once() {
    let machine = Machine::new();
    machine.smart(4);
    machine.vitals(&["collect"]);
    let status = machine.status();
    assert_eq!(status["class"], "crit");
    assert!(status["text"].as_str().unwrap().contains("nvme0n1"), "{status}");
}

#[test]
fn without_a_collection_the_chip_says_so() {
    let machine = Machine::new();
    assert_eq!(machine.status()["class"], "stale");
}

fn store(machine: &Machine) -> vitals::store::Store {
    vitals::store::Store::at(machine.root.join("state/vitals.sqlite")).unwrap()
}

#[test]
fn a_collection_during_game_mode_opens_a_game_session() {
    let machine = Machine::new();
    fs::write(machine.root.join("run/game-performance.paused"), "").unwrap();
    machine.vitals(&["collect"]);
    let running = store(&machine).running_game().unwrap().expect("no session");
    assert_eq!(running.name, vitals::games::UNNAMED);
}

#[test]
fn a_crash_left_by_the_root_unit_reaches_the_journal() {
    let machine = Machine::new();
    fs::write(machine.root.join("root-data/crashes.json"),
              r#"[{"time": 1792352525000000, "pid": 42, "uid": 1000, "sig": 6, "exe": "/usr/bin/waybar"}]"#).unwrap();
    machine.vitals(&["collect"]);
    machine.vitals(&["collect"]);
    let events = store(&machine).events(0).unwrap();
    assert_eq!(events.iter().map(|event| event.text.as_str()).collect::<Vec<_>>(), ["waybar (SIGABRT)"]);
}

#[test]
fn a_boot_that_was_watched_and_never_shut_down_cleanly_is_a_hard_stop() {
    let machine = Machine::new();
    let boot_id = machine.root.join("sys/proc/sys/kernel/random/boot_id");
    fs::create_dir_all(boot_id.parent().unwrap()).unwrap();
    fs::write(&boot_id, "first\n").unwrap();
    assert!(machine.vitals(&["shutdown", "watch"]).status.success());
    machine.vitals(&["collect"]);
    fs::write(&boot_id, "second\n").unwrap();
    machine.vitals(&["collect"]);
    let kinds: Vec<String> = store(&machine).events(0).unwrap().into_iter().map(|event| event.kind).collect();
    assert_eq!(kinds, ["unclean"]);
}

#[test]
fn a_clean_shutdown_leaves_the_journal_empty() {
    let machine = Machine::new();
    let boot_id = machine.root.join("sys/proc/sys/kernel/random/boot_id");
    fs::create_dir_all(boot_id.parent().unwrap()).unwrap();
    fs::write(&boot_id, "first\n").unwrap();
    machine.vitals(&["shutdown", "watch"]);
    machine.vitals(&["collect"]);
    machine.vitals(&["shutdown", "clean"]);
    fs::write(&boot_id, "second\n").unwrap();
    machine.vitals(&["collect"]);
    assert!(store(&machine).events(0).unwrap().is_empty());
}
