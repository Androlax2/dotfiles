//! Check our global command registration without sending a prompt or activating a provider.
mod common;
use common::sandbox::*;

use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

const COMMANDS: [(&str, bool); 2] = [("code-review", true), ("simplify", false)];

fn command_source(name: &str) -> String {
    fs::read_to_string(repo().join(format!("config/opencode/.config/opencode/commands/{name}.md")))
        .unwrap_or_else(|error| panic!("missing global /{name} definition: {error}"))
}

#[test]
fn global_templates_keep_tool_approval_and_model_selection_at_invocation() {
    for (name, is_background) in COMMANDS {
        let source = command_source(name);
        let (metadata, template) = source.strip_prefix("---\n").unwrap().split_once("\n---\n").unwrap();
        let fields: Vec<_> = metadata.lines().collect();
        assert_eq!(fields.len(), 2, "/{name} must not override agents, models or permissions");
        assert!(fields[0].starts_with("description: ") && fields[0].len() > "description: ".len());
        assert_eq!(fields[1], format!("subagent: {is_background}"));
        assert_eq!(template.matches("$ARGUMENTS").count(), 1, "/{name} must receive its target");
        assert!(!template.contains("!`") && !template.contains("```!"), "/{name} must not execute shell interpolation outside tool approvals");
    }
}

struct OpenCode(Child);

impl Drop for OpenCode {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn commands(sandbox: &Sandbox, binary: &Path, url: &str) -> Vec<Value> {
    let output = sandbox.command(binary, &[])
        .env_clear().envs(&sandbox.env).current_dir(&sandbox.root)
        .args(["api", "--server", url, "get", "/api/command"]).output().unwrap();
    assert!(output.status.success(), "OpenCode command discovery: {}", stderr(&output));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| panic!("OpenCode command discovery: {error}: {}", stdout(&output)));
    response["data"].as_array().unwrap().clone()
}

fn wait_for_commands(sandbox: &Sandbox, binary: &Path, url: &str, names: &[&str]) -> Vec<Value> {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let entries = commands(sandbox, binary, url);
        if names.iter().all(|name| entries.iter().any(|entry| entry["name"] == *name)) {
            return entries;
        }
        assert!(Instant::now() < deadline, "OpenCode did not register {names:?}: {entries:?}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn native_discovery_loads_both_global_commands_without_collisions() {
    let binary = std::env::var_os("OPENCODE_BINARY").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap()).join(".opencode/bin/opencode"));
    assert!(binary.is_file(), "this discovery test requires OpenCode V2 at {} (or OPENCODE_BINARY)", binary.display());
    let mut sandbox = Sandbox::new();
    for (name, folder) in [("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data"), ("XDG_CACHE_HOME", "cache")] {
        sandbox.env.insert(name.into(), sandbox.root.join(folder).display().to_string());
    }
    sandbox.env.insert("OPENCODE_DB".into(), sandbox.root.join("opencode.db").display().to_string());
    sandbox.env.insert("OPENCODE_CONFIG_CONTENT".into(), "{}".into());
    sandbox.env.insert("OPENCODE_DISABLE_AUTOUPDATE".into(), "1".into());
    sandbox.env.insert("OPENCODE_SERVER_PASSWORD".into(), "scratch-command-discovery-only".into());
    let mut server = OpenCode(sandbox.command(&binary, &[])
        .env_clear().envs(&sandbox.env).current_dir(&sandbox.root)
        .args(["serve", "--stdio", "--hostname", "127.0.0.1", "--port", "0"])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn().unwrap());
    let output = server.0.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut line = String::new();
        BufReader::new(output).read_line(&mut line).unwrap();
        let _ = sender.send(line);
    });
    let ready = receiver.recv_timeout(Duration::from_secs(15)).expect("OpenCode did not announce its scratch server");
    reader.join().unwrap();
    let ready: Value = serde_json::from_str(&ready).unwrap();
    let url = ready["url"].as_str().unwrap();
    // A one-shot standalone request can return before the built-in command plugins finish loading.
    let baseline = wait_for_commands(&sandbox, &binary, url, &["init", "review", "goal"]);
    for (name, _) in COMMANDS {
        assert!(!baseline.iter().any(|entry| entry["name"] == name), "/{name} already exists before installing our global definition");
    }

    let global = sandbox.root.join("config/opencode");
    fs::create_dir_all(global.join("commands")).unwrap();
    // Check the Stow package's other registrations without loading its real plugins or MCP servers.
    let source = repo().join("config/opencode/.config/opencode");
    for name in ["opencode.json", "opencode.jsonc"] {
        let text = fs::read_to_string(source.join(name)).unwrap();
        let json = text.lines().filter(|line| !line.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
        let config: Value = serde_json::from_str(&json).unwrap();
        for (command, _) in COMMANDS {
            assert!(config["commands"][command].is_null() && config["command"][command].is_null(), "{name} shadows /{command}");
        }
    }
    for (name, _) in COMMANDS {
        assert!(!source.join(format!("command/{name}.md")).exists(), "legacy command shadows /{name}");
        fs::write(global.join(format!("commands/{name}.md")), command_source(name)).unwrap();
    }
    let discovered = wait_for_commands(&sandbox, &binary, url, &["code-review", "simplify"]);
    for (name, _) in COMMANDS {
        let entries: Vec<_> = discovered.iter().filter(|entry| entry["name"] == name).collect();
        assert_eq!(entries.len(), 1, "/{name} is not uniquely discoverable");
        let source = command_source(name);
        let description = source.lines().nth(1).unwrap().strip_prefix("description: ").unwrap();
        assert_eq!(entries[0]["description"], description, "/{name} did not load our global Markdown definition");
    }
}
