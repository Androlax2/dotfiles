//! AUR builds run in a user scope at low CPU weight with a memory cap, wherever yay is started:
//! fish, `make update` / `make packages`, and the Waybar update button.

mod common;
use common::sandbox::*;
use std::fs;

const CAPPED_YAY: &str = "systemd-run --user --scope --quiet -p CPUWeight=20 -p MemoryHigh=12G yay";

fn fish_yay(sandbox: &Sandbox, command: &str) {
    let function = repo().join("config/fish/.config/fish/functions/yay.fish");
    let script = format!("source {}; {command}", function.display());
    let output = sandbox.run("fish", &["--no-config", "-c", &script], &[]);
    assert!(output.status.success(), "{}", stderr(&output));
}

#[test]
fn yay_in_fish_runs_in_a_capped_scope() {
    let sandbox = Sandbox::new();
    sandbox.shim("systemd-run", "");
    sandbox.shim("yay", "");
    fish_yay(&sandbox, "yay -Syu");
    assert_eq!(sandbox.calls("systemd-run"), vec![format!("{CAPPED_YAY} -Syu")]);
}

#[test]
fn command_yay_in_fish_bypasses_the_cap() {
    let sandbox = Sandbox::new();
    sandbox.shim("systemd-run", "");
    sandbox.shim("yay", "");
    fish_yay(&sandbox, "command yay -Syu");
    assert!(sandbox.calls("systemd-run").is_empty());
    assert_eq!(sandbox.calls("yay"), vec!["yay -Syu"]);
}

#[test]
fn make_update_and_packages_build_in_a_capped_scope() {
    let sandbox = Sandbox::new();
    let output = sandbox.command("make", &[]).current_dir(repo()).args(["--dry-run", "update", "packages"]).output().unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    let yay_lines: Vec<String> = stdout(&output).lines().filter(|line| line.contains("yay")).map(str::to_string).collect();
    assert_eq!(yay_lines.len(), 2, "{yay_lines:?}");
    for line in yay_lines {
        assert!(line.starts_with(CAPPED_YAY), "{line}");
    }
}

#[test]
fn the_waybar_update_button_builds_in_a_capped_scope() {
    let config = fs::read_to_string(repo().join("config/waybar/.config/waybar/config.jsonc")).unwrap();
    let clicks: Vec<&str> = config.lines().filter(|line| line.contains("on-click") && line.contains("yay")).collect();
    assert!(!clicks.is_empty());
    for click in clicks {
        assert!(click.contains(&format!("'{CAPPED_YAY} -Syu;")), "{click}");
    }
}
