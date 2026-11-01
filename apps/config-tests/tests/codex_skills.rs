//! The Codex skills are copies of the OpenCode review commands: both port Claude Code's
//! /code-review and /simplify, so the two harnesses must not drift apart.
mod common;
use common::sandbox::*;

use std::fs;

const PORTS: [(&str, &str); 2] = [
    ("code-review", "Arguments: "),
    ("simplify", "Review target (optional): "),
];

fn split_frontmatter<'a>(source: &'a str, file: &str) -> (&'a str, &'a str) {
    source.strip_prefix("---\n").and_then(|rest| rest.split_once("\n---\n"))
        .unwrap_or_else(|| panic!("{file} has no frontmatter"))
}

fn read(relative: &str) -> String {
    fs::read_to_string(repo().join(relative)).unwrap_or_else(|error| panic!("missing {relative}: {error}"))
}

/// Codex invokes skills as `$name` where OpenCode types `/name`; the argument line names
/// where the arguments come from. Everything else is the same recipe.
fn recipe_lines(body: &str, argument_prefix: &str) -> Vec<String> {
    body.lines().filter(|line| !line.starts_with(argument_prefix))
        .map(|line| line.replace("$code-review", "/code-review").replace("$simplify", "/simplify")).collect()
}

#[test]
fn codex_skills_follow_the_same_recipe_as_the_opencode_commands() {
    for (name, argument_prefix) in PORTS {
        let command_file = format!("config/opencode/.config/opencode/commands/{name}.md");
        let skill_file = format!("config/codex/.codex/skills/{name}/SKILL.md");
        let (command, skill) = (read(&command_file), read(&skill_file));
        let (_, command_body) = split_frontmatter(&command, &command_file);
        let (_, skill_body) = split_frontmatter(&skill, &skill_file);
        assert_eq!(recipe_lines(skill_body, argument_prefix), recipe_lines(command_body, argument_prefix),
            "the Codex {name} skill drifted from the OpenCode /{name} command");
    }
}

#[test]
fn codex_skills_say_where_their_arguments_come_from() {
    for (name, argument_prefix) in PORTS {
        let skill_file = format!("config/codex/.codex/skills/{name}/SKILL.md");
        let skill = read(&skill_file);
        let (_, body) = split_frontmatter(&skill, &skill_file);
        assert_eq!(body.lines().filter(|line| line.starts_with(argument_prefix)).count(), 1,
            "the Codex {name} skill must have exactly one argument line");
    }
}

#[test]
fn codex_skills_keep_the_command_name_and_description() {
    for (name, _) in PORTS {
        let command_file = format!("config/opencode/.config/opencode/commands/{name}.md");
        let skill_file = format!("config/codex/.codex/skills/{name}/SKILL.md");
        let (command, skill) = (read(&command_file), read(&skill_file));
        let (command_metadata, _) = split_frontmatter(&command, &command_file);
        let (skill_metadata, _) = split_frontmatter(&skill, &skill_file);
        let description = command_metadata.lines().find(|line| line.starts_with("description: ")).unwrap()
            .replace("/code-review", "$code-review");
        assert_eq!(skill_metadata.lines().collect::<Vec<_>>(), [format!("name: {name}").as_str(), description.as_str()]);
    }
}

#[test]
fn codex_runs_the_review_skills_only_when_asked() {
    // Both skills can edit files (--fix, /simplify), so Codex must not pick them on its own.
    for (name, _) in PORTS {
        assert_eq!(read(&format!("config/codex/.codex/skills/{name}/agents/openai.yaml")),
            "policy:\n  allow_implicit_invocation: false\n", "Codex may start the {name} skill unasked");
    }
}
