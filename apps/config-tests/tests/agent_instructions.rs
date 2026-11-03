//! The global agent instructions: one file, shared by Claude Code and opencode.

mod common;
use common::sandbox::*;

use std::fs;

#[test]
fn opencode_reads_the_same_rules_as_claude() {
    // opencode 2 reads ~/.config/opencode/AGENTS.md, never ~/.claude/CLAUDE.md: the stowed AGENTS.md
    // is the same file.
    let agents = repo().join("config/opencode/.config/opencode/AGENTS.md");
    assert_eq!(fs::read_link(&agents).unwrap(), std::path::Path::new("../../../claude/.claude/CLAUDE.md"));
    assert_eq!(fs::read_to_string(&agents).unwrap(), fs::read_to_string(repo().join("config/claude/.claude/CLAUDE.md")).unwrap());
}

#[test]
fn stowing_both_harnesses_keeps_one_global_instruction_owner() {
    let sandbox = Sandbox::new();
    let output = sandbox.command("stow", &[])
        .arg("--dir").arg(repo().join("config"))
        .arg("--target").arg(&sandbox.home)
        .args(["--no-folding", "claude", "opencode"]).output().unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    let owner = repo().join("config/claude/.claude/CLAUDE.md").canonicalize().unwrap();
    for relative in [".claude/CLAUDE.md", ".config/opencode/AGENTS.md"] {
        assert_eq!(sandbox.home.join(relative).canonicalize().unwrap(), owner,
            "{relative} no longer loads the shared global instructions");
    }
    for name in ["code-review", "simplify"] {
        assert_eq!(sandbox.home.join(format!(".config/opencode/commands/{name}.md")).canonicalize().unwrap(),
            repo().join(format!("config/opencode/.config/opencode/commands/{name}.md")).canonicalize().unwrap());
    }
}
