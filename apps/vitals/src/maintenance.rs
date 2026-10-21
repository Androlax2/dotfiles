//! Read-only native maintenance schedules/results and the existing advisory disk report.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::process::Command;
use serde_json::Value;

const TASKS: [(&str, &str, bool, &str); 5] = [
    ("paccache", "Paquets", false, "2 versions + archives récentes de 30 jours"),
    ("package-cache-report", "Rapport paquets", false, "simulation uniquement · aucune suppression"),
    ("cache-maintenance-report", "Rapport Turbo", true, "fichiers de plus de 30 jours · aucune suppression"),
    ("disk-space-review", "Revue mensuelle", true, "jeux/projets · suppression seulement après accord"),
    ("fstrim", "TRIM SSD", false, "entretien natif hebdomadaire du SSD"),
];
const PROPERTIES: &str = "Id,LoadState,UnitFileState,ActiveState,LastTriggerUSec,NextElapseUSecRealtime,Result,ExecMainStartTimestamp,ExecMainExitTimestamp,ExecMainStatus";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunState { Running, Succeeded, Failed, Unknown }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Check { SpaceReview, TurboReport }

#[derive(Clone, Debug)]
pub struct Task {
    pub timer: String,
    pub label: &'static str,
    /// Agreed policy, not an audit of arbitrary overridden service commands.
    pub policy: &'static str,
    pub enabled: Option<bool>,
    /// Service start or timer trigger; this is not proof of completed maintenance.
    pub last_run: Option<i64>,
    pub next_run: Option<i64>,
    pub state: RunState,
    pub detail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ReviewEntry {
    pub path: String,
    pub allocated_bytes: u64,
    pub complete: bool,
}

#[derive(Clone, Debug)]
pub struct ReviewIssue {
    pub path: String,
    pub error: String,
}

#[derive(Clone, Debug)]
pub struct Review {
    pub completed_at: i64,
    pub complete: bool,
    pub measured_games: u64,
    pub measured_checkouts: u64,
    pub omitted_games: u64,
    pub omitted_checkouts: u64,
    pub errors: u64,
    pub issues: Vec<ReviewIssue>,
    /// None identifies older reports which did not distinguish expected link skips.
    pub skipped_symlinks: Option<u64>,
    pub unavailable: usize,
    pub games: Vec<ReviewEntry>,
    pub checkouts: Vec<ReviewEntry>,
}

#[derive(Clone, Debug)]
pub struct Maintenance {
    pub tasks: Vec<Task>,
    pub journal_retention: Result<Option<String>, String>,
    pub review: Result<Option<Review>, String>,
}

fn clean_text(text: &str) -> String {
    text.chars().filter(|character| !character.is_control()).take(4096).collect()
}

/// Request one of the two unprivileged, non-deleting report services; not a completed result.
pub fn request_check(check: Check) -> Result<(), String> {
    let unit = match check {
        Check::SpaceReview => "disk-space-review.service",
        Check::TurboReport => "cache-maintenance-report.service",
    };
    let output = Command::new("systemctl").args(["--user", "start", "--no-block", unit])
        .output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!("demande refusée ({}) : {}", output.status, clean_text(&String::from_utf8_lossy(&output.stderr))));
    }
    Ok(())
}

fn timestamp(value: Option<&String>) -> Option<i64> {
    let stamp = value?.strip_prefix('@')?.split('.').next()?.parse::<i64>().ok()?;
    (stamp > 0).then_some(stamp)
}

type Properties = BTreeMap<String, String>;

fn read_units(user: bool) -> (BTreeMap<String, Properties>, Option<String>) {
    let mut command = Command::new("systemctl");
    if user { command.arg("--user"); }
    command.args(["show", "--no-pager", "--timestamp=unix", &format!("--property={PROPERTIES}")]);
    for (name, _, scope, _) in TASKS {
        if scope == user { command.args([format!("{name}.timer"), format!("{name}.service")]); }
    }
    let output = match command.output() {
        Ok(output) => output,
        Err(error) => return (BTreeMap::new(), Some(format!("systemctl : {error}"))),
    };
    let error = (!output.status.success()).then(|| format!("systemctl {} : {}", output.status, clean_text(&String::from_utf8_lossy(&output.stderr))));
    let text = String::from_utf8_lossy(&output.stdout);
    let units = text.split("\n\n").filter_map(|block| {
        let properties: Properties = block.lines().filter_map(|line| line.split_once('='))
            .map(|(key, value)| (key.to_string(), value.to_string())).collect();
        let id = properties.get("Id")?.clone();
        Some((id, properties))
    }).collect();
    (units, error)
}

fn task(name: &str, label: &'static str, policy: &'static str, units: &BTreeMap<String, Properties>, error: &Option<String>) -> Task {
    let mut task = Task { timer: format!("{name}.timer"), label, policy, enabled: None,
                          last_run: None, next_run: None, state: RunState::Unknown, detail: error.clone() };
    let timer = units.get(&task.timer).filter(|properties| properties.get("LoadState").is_some_and(|value| value == "loaded"));
    if let Some(timer) = timer {
        task.enabled = match timer.get("UnitFileState").map(String::as_str) {
            Some("enabled" | "enabled-runtime") => Some(true),
            Some("disabled" | "masked" | "masked-runtime") => Some(false),
            _ => None,
        };
        task.last_run = timestamp(timer.get("LastTriggerUSec"));
        task.next_run = timestamp(timer.get("NextElapseUSecRealtime"));
    }
    let Some(service) = units.get(&format!("{name}.service"))
        .filter(|properties| properties.get("LoadState").is_some_and(|value| value == "loaded")) else {
            task.detail.get_or_insert_with(|| "unité absente ou données indisponibles".into());
            return task;
        };
    task.last_run = timestamp(service.get("ExecMainStartTimestamp")).or(task.last_run);
    let active = service.get("ActiveState").map(String::as_str);
    let result = service.get("Result").map(String::as_str);
    let exit_status = service.get("ExecMainStatus").and_then(|value| value.parse::<i32>().ok());
    task.state = if matches!(active, Some("activating" | "active" | "deactivating")) {
        RunState::Running
    } else if active == Some("failed") || result.is_some_and(|result| !result.is_empty() && result != "success") || exit_status.is_some_and(|code| code != 0) {
        task.detail = Some(format!("résultat : {} · code : {}", clean_text(result.unwrap_or("inconnu")), exit_status.map(|code| code.to_string()).unwrap_or_else(|| "inconnu".into())));
        RunState::Failed
    } else if result == Some("success") && exit_status == Some(0) && timestamp(service.get("ExecMainExitTimestamp")).is_some() {
        RunState::Succeeded
    } else { RunState::Unknown };
    task
}

fn journal_retention() -> Result<Option<String>, String> {
    let output = Command::new("systemd-analyze").args(["cat-config", "systemd/journald.conf"])
        .output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!("journald : {}", clean_text(&String::from_utf8_lossy(&output.stderr))));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut in_journal = false;
    let mut retention = None;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_journal = line == "[Journal]";
        } else if in_journal && let Some((key, value)) = line.split_once('=') && key.trim() == "MaxRetentionSec" {
            let value = value.trim();
            retention = (!value.is_empty()).then(|| clean_text(value));
        }
    }
    Ok(retention)
}

fn count(report: &Value, key: &str) -> Result<u64, String> {
    report[key].as_u64().ok_or_else(|| format!("compteur invalide : {key}"))
}

fn entries(report: &Value, key: &str) -> Result<Vec<ReviewEntry>, String> {
    let entries = report[key].as_array().ok_or_else(|| format!("liste invalide : {key}"))?;
    if entries.len() > 20 { return Err(format!("liste trop longue : {key}")); }
    let mut result = Vec::new();
    for entry in entries {
        let path = clean_text(entry["path"].as_str().ok_or("chemin absent")?);
        let allocated_bytes = entry["allocated_bytes"].as_u64().ok_or("taille invalide")?;
        let complete = entry["complete"].as_bool().ok_or("état partiel absent")?;
        if result.len() < 3 { result.push(ReviewEntry { path, allocated_bytes, complete }); }
    }
    Ok(result)
}

fn read_review() -> Result<Option<Review>, String> {
    let path = crate::host::state_home().join("disk-maintenance/latest.json");
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
        Ok(metadata) if !metadata.is_file() => return Err("la revue doit être un fichier normal, pas un lien".into()),
        Ok(_) => {}
    }
    let mut content = Vec::new();
    fs::File::open(&path).map_err(|error| error.to_string())?.take(1_048_577)
        .read_to_end(&mut content).map_err(|error| error.to_string())?;
    if content.len() > 1_048_576 { return Err("revue supérieure à 1 Mio".into()); }
    let report: Value = serde_json::from_slice(&content).map_err(|error| error.to_string())?;
    if report["schema_version"].as_u64() != Some(1) || report["advisory_only"].as_bool() != Some(true) {
        return Err("format de revue inconnu".into());
    }
    let completed_at = report["completed_at"].as_i64().filter(|stamp| *stamp > 0 && *stamp <= crate::host::now_secs()).ok_or("date de revue invalide")?;
    let error_entries = report["errors"].as_array().ok_or("liste d'erreurs absente")?;
    let mut issues = Vec::new();
    for entry in error_entries {
        let path = clean_text(entry["path"].as_str().ok_or("chemin d'erreur absent")?);
        let error = clean_text(entry["error"].as_str().ok_or("message d'erreur absent")?);
        if issues.len() < 3 { issues.push(ReviewIssue { path, error }); }
    }
    let errors = error_entries.len() as u64;
    let errors = errors.checked_add(count(&report, "omitted_errors")?).ok_or("compteur d'erreurs trop grand")?;
    let skipped_symlinks = report.get("skipped_symlinks")
        .map(|value| value.as_u64().ok_or("compteur d'exclusions invalide")).transpose()?;
    Ok(Some(Review {
        completed_at,
        complete: report["complete"].as_bool().ok_or("état de revue absent")?,
        measured_games: count(&report, "measured_games")?, measured_checkouts: count(&report, "measured_checkouts")?,
        omitted_games: count(&report, "omitted_games")?, omitted_checkouts: count(&report, "omitted_checkouts")?,
        errors, issues, skipped_symlinks,
        unavailable: report["unavailable_roots"].as_array().ok_or("chemins indisponibles absents")?.len(),
        games: entries(&report, "games")?, checkouts: entries(&report, "checkouts")?,
    }))
}

/// Read native schedules/results and the published review without starting maintenance.
pub fn gather() -> Maintenance {
    let (system, system_error) = read_units(false);
    let (user, user_error) = read_units(true);
    let tasks = TASKS.into_iter().map(|(name, label, is_user, policy)| {
        if is_user { task(name, label, policy, &user, &user_error) }
        else { task(name, label, policy, &system, &system_error) }
    }).collect();
    Maintenance { tasks, journal_retention: journal_retention(), review: read_review() }
}
