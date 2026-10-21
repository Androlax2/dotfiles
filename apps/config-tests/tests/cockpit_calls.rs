//! The dotfiles' configuration that calls the cockpit (agent-cockpit): the lock screen, the tmux
//! bindings, opencode's configuration.

mod common;
use common::sandbox::*;

use serde_json::{Value, json};
use std::fs;

#[test]
fn the_lock_screen_tells_the_phone_and_no_longer_hands_sessions_off() {
    // The phone page took over from Remote Control: the lock screen calls away, never handoff.
    let lock = fs::read_to_string(repo().join("config/hyprland/.config/hypr/scripts/lock.sh")).unwrap();
    assert!(lock.contains("cockpit away on") && lock.contains("cockpit away off") && !lock.contains("handoff"), "{lock}");
}

#[test]
fn tmux_bindings_name_cockpit_by_its_full_path() {
    // The tmux server runs popups with PATH=/usr/local/bin:/usr/bin: a bare cockpit is not
    // found there, and the Alt+Q popup closed the moment it opened.
    let conf = fs::read_to_string(repo().join("config/tmux/.config/tmux/tmux.conf")).unwrap();
    for line in conf.lines().filter(|line| !line.trim_start().starts_with('#')) {
        for (at, _) in line.match_indices("cockpit ") {
            assert!(line[..at].ends_with("/.local/bin/"), "bare cockpit in: {line}");
        }
    }
}

/// opencode.jsonc as JSON: its whole-line // comments left out.
fn opencode_config() -> Value {
    let text = fs::read_to_string(repo().join("config/opencode/.config/opencode/opencode.jsonc")).unwrap();
    let json: String = text.lines().filter(|line| !line.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
    serde_json::from_str(&json).expect("opencode.jsonc is JSON once its comment lines are left out")
}

#[test]
fn opencode_registers_figcraft_app_on_the_port_the_launcher_forwards() {
    let server = &opencode_config()["mcp"]["figcraft-app"];
    let sandbox = Sandbox::new();
    sandbox.shim("figcraft-cli", "exit 0");
    for port in ["17979", "17980"] {
        let arguments = server["command"].as_array().unwrap();
        let mut command = sandbox.command(arguments[0].as_str().unwrap(), &[("FIGCRAFT_CONTROL_PORT", port)]);
        for argument in &arguments[1..] { command.arg(argument.as_str().unwrap()); }
        assert!(command.output().unwrap().status.success());
        assert_eq!(sandbox.calls("figcraft-cli").last().unwrap(), &format!("figcraft-cli mcp --attach {port}"));
    }
    let arguments = server["command"].as_array().unwrap();
    let mut command = sandbox.command(arguments[0].as_str().unwrap(), &[("FIGCRAFT_CONTROL_PORT", "")]);
    for argument in &arguments[1..] { command.arg(argument.as_str().unwrap()); }
    assert!(!command.output().unwrap().status.success(), "connected to an implicit default port");
}

#[test]
fn opencode_leaves_updates_to_opencode_update() {
    assert_eq!(opencode_config()["autoupdate"], json!(false));
}

#[test]
fn opencode_reads_the_same_rules_as_claude() {
    // opencode 2 reads ~/.config/opencode/AGENTS.md, never ~/.claude/CLAUDE.md: the stowed AGENTS.md
    // is the same file (agent-cockpit's opencode profile lets sessions read it where it lives).
    let agents = repo().join("config/opencode/.config/opencode/AGENTS.md");
    assert_eq!(fs::read_link(&agents).unwrap(), std::path::Path::new("../../../claude/.claude/CLAUDE.md"));
    assert_eq!(fs::read_to_string(&agents).unwrap(), fs::read_to_string(repo().join("config/claude/.claude/CLAUDE.md")).unwrap());
}
