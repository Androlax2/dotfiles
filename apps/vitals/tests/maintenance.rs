use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use serde_json::json;
use vitals::maintenance::{self, RunState};

static ENVIRONMENT: Mutex<()> = Mutex::new(());

struct Fixture {
    root: PathBuf,
    previous: Vec<(&'static str, Option<OsString>)>,
    _lock: MutexGuard<'static, ()>,
}

impl Fixture {
    fn new() -> Self {
        let lock = ENVIRONMENT.lock().unwrap_or_else(|error| error.into_inner());
        let root = std::env::temp_dir().join(format!("vitals-maintenance-{}", vitals::host::token_hex(6)));
        for name in ["bin", "state/disk-maintenance", "runtime", "home"] { fs::create_dir_all(root.join(name)).unwrap(); }
        let values = [("HOME", root.join("home").into_os_string()),
                      ("XDG_STATE_HOME", root.join("state").into_os_string()),
                      ("XDG_RUNTIME_DIR", root.join("runtime").into_os_string()),
                      ("PATH", OsString::from(format!("{}:/usr/bin:/bin", root.join("bin").display()))),
                      ("VITALS_TEST_FIXTURE", root.as_os_str().to_owned())];
        let mut previous = Vec::new();
        for (key, value) in values {
            previous.push((key, std::env::var_os(key)));
            // SAFETY: every test in this integration-test process holds ENVIRONMENT.
            unsafe { std::env::set_var(key, value); }
        }
        let fixture = Self { root, previous, _lock: lock };
        fixture.executable("systemctl", r#"#!/bin/sh
printf '%s\n' "$*" >> "$VITALS_TEST_FIXTURE/calls"
if [ -f "$VITALS_TEST_FIXTURE/fail" ]; then echo 'bus unavailable' >&2; exit 1; fi
case "$*" in *--user*) cat "$VITALS_TEST_FIXTURE/user";; *) cat "$VITALS_TEST_FIXTURE/system";; esac
"#);
        fixture.executable("systemd-analyze", "#!/bin/sh\ncat \"$VITALS_TEST_FIXTURE/journal\"\n");
        for name in ["system", "user", "journal"] { fs::write(fixture.root.join(name), "").unwrap(); }
        fixture
    }

    fn executable(&self, name: &str, content: &str) {
        use std::os::unix::fs::PermissionsExt;
        let path = self.root.join("bin").join(name);
        fs::write(&path, content).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn units(&self, timer: &str, service: &str) {
        fs::write(self.root.join("system"), format!("{timer}\n\n{service}\n")).unwrap();
    }

    fn report(&self) -> PathBuf { self.root.join("state/disk-maintenance/latest.json") }

    fn review(&self) -> serde_json::Value {
        json!({"schema_version":1,"advisory_only":true,"completed_at":1000,"complete":true,
            "games":[{"path":"/games/large","allocated_bytes":1073741824u64,"complete":true}],
            "checkouts":[],"measured_games":4,"measured_checkouts":2,"omitted_games":0,
            "omitted_checkouts":0,"unavailable_roots":[],"errors":[],"omitted_errors":0})
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for (key, value) in &self.previous {
            // SAFETY: the environment lock remains held until after restoration.
            unsafe { if let Some(value) = value { std::env::set_var(key, value); } else { std::env::remove_var(key); } }
        }
        fs::remove_dir_all(&self.root).unwrap();
    }
}

const TIMER: &str = "Id=paccache.timer\nLoadState=loaded\nUnitFileState=enabled\nActiveState=active\nLastTriggerUSec=@1000\nNextElapseUSecRealtime=@2000";

#[test]
fn enabled_is_not_proof_of_a_successful_run() {
    let fixture = Fixture::new();
    fixture.units(TIMER, "Id=paccache.service\nLoadState=loaded\nActiveState=inactive\nResult=success\nExecMainStatus=0\nExecMainExitTimestamp=");
    let snapshot = maintenance::gather();
    let task = snapshot.tasks.iter().find(|task| task.timer == "paccache.timer").unwrap();
    assert_eq!(task.enabled, Some(true));
    assert_eq!(task.state, RunState::Unknown);
    assert_eq!((task.last_run, task.next_run), (Some(1000), Some(2000)));
}

#[test]
fn running_completed_and_failed_results_are_distinct() {
    let fixture = Fixture::new();
    for (state, result, exit, finished, expected) in [
        ("activating", "success", "0", "", RunState::Running),
        ("inactive", "success", "0", "@1050", RunState::Succeeded),
        ("failed", "exit-code", "1", "@1050", RunState::Failed),
        ("inactive", "success", "1", "@1050", RunState::Failed),
    ] {
        fixture.units(TIMER, &format!("Result={result}\nExecMainStatus={exit}\nId=paccache.service\nLoadState=loaded\nActiveState={state}\nExecMainStartTimestamp=@990\nExecMainExitTimestamp={finished}"));
        let snapshot = maintenance::gather();
        assert_eq!(snapshot.tasks[0].state, expected);
        assert_eq!(snapshot.tasks[0].last_run, Some(990));
    }
}

#[test]
fn disabled_timer_can_have_a_previous_successful_run() {
    let fixture = Fixture::new();
    fixture.units(&TIMER.replace("enabled", "disabled").replace("LastTriggerUSec=@1000", "LastTriggerUSec="),
        "Id=paccache.service\nLoadState=loaded\nActiveState=inactive\nResult=success\nExecMainStatus=0\nExecMainExitTimestamp=@1050");
    let snapshot = maintenance::gather();
    assert_eq!(snapshot.tasks[0].enabled, Some(false));
    assert_eq!(snapshot.tasks[0].state, RunState::Succeeded);
    assert_eq!(snapshot.tasks[0].last_run, None);
}

#[test]
fn timestamp_fractions_and_reversed_unit_order_do_not_lose_known_runs() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("system"), format!(
        "Id=paccache.service\nLoadState=loaded\nActiveState=inactive\nResult=success\nExecMainStatus=0\nExecMainStartTimestamp=@990.123456\nExecMainExitTimestamp=@1050.123456\n\n{}\n",
        TIMER.replace("@2000", "@2000.750000"))).unwrap();
    let snapshot = maintenance::gather();
    assert_eq!(snapshot.tasks[0].last_run, Some(990));
    assert_eq!(snapshot.tasks[0].next_run, Some(2000));
    assert_eq!(snapshot.tasks[0].state, RunState::Succeeded);
}

#[test]
fn a_failed_batch_preserves_valid_units_and_marks_the_missing_units_unknown() {
    let fixture = Fixture::new();
    fixture.units(TIMER, "Id=paccache.service\nLoadState=loaded\nActiveState=inactive\nResult=success\nExecMainStatus=0\nExecMainExitTimestamp=@1050");
    fixture.executable("systemctl", "#!/bin/sh\ncat \"$VITALS_TEST_FIXTURE/system\"\necho 'one unit unavailable' >&2\nexit 1\n");
    let snapshot = maintenance::gather();
    assert_eq!(snapshot.tasks[0].state, RunState::Succeeded);
    assert!(snapshot.tasks[0].detail.is_some());
    assert_eq!(snapshot.tasks[1].state, RunState::Unknown);
}

#[test]
fn missing_units_and_failed_queries_stay_unknown() {
    let fixture = Fixture::new();
    fixture.units("Id=paccache.timer\nLoadState=not-found", "Id=paccache.service\nLoadState=not-found\nResult=success\nExecMainExitTimestamp=@1050");
    let snapshot = maintenance::gather();
    assert_eq!(snapshot.tasks[0].enabled, None);
    assert_eq!(snapshot.tasks[0].state, RunState::Unknown);
    fs::write(fixture.root.join("fail"), "").unwrap();
    let snapshot = maintenance::gather();
    assert!(snapshot.tasks.iter().all(|task| task.state == RunState::Unknown && task.detail.is_some()));
}

#[test]
fn queries_use_both_scopes_and_only_read_only_operations() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("user"), "Id=disk-space-review.timer\nLoadState=loaded\nUnitFileState=enabled\nNextElapseUSecRealtime=@3000\n\nId=disk-space-review.service\nLoadState=loaded\nActiveState=activating").unwrap();
    let snapshot = maintenance::gather();
    let review = snapshot.tasks.iter().find(|task| task.timer == "disk-space-review.timer").unwrap();
    assert_eq!(review.state, RunState::Running);
    assert_eq!(review.next_run, Some(3000));
    let calls = fs::read_to_string(fixture.root.join("calls")).unwrap();
    assert_eq!(calls.lines().count(), 2);
    assert!(calls.lines().any(|line| line.contains("--user")));
    for line in calls.lines() {
        assert!(line.split_whitespace().any(|word| word == "show"));
        for forbidden in ["start", "enable", "reload", "stop", "restart"] {
            assert!(!line.split_whitespace().any(|word| word == forbidden), "{line}");
        }
    }
}

#[test]
fn journal_retention_uses_the_last_active_value_not_comments() {
    let fixture = Fixture::new();
    for (content, expected) in [
        ("#MaxRetentionSec=0\n[Journal]\nMaxRetentionSec=60day\n# /etc/drop-in\nMaxRetentionSec = 30day\n", Some("30day")),
        ("#MaxRetentionSec=0\n", None),
        ("[Journal]\nMaxRetentionSec=30day\nMaxRetentionSec=\n", None),
        ("[Journal]\nMaxRetentionSec=30day\n[Ignored]\nMaxRetentionSec=1day\n", Some("30day")),
    ] {
        fs::write(fixture.root.join("journal"), content).unwrap();
        assert_eq!(maintenance::gather().journal_retention.unwrap().as_deref(), expected);
    }
}

#[test]
fn complete_partial_and_missing_reports_remain_distinct() {
    let fixture = Fixture::new();
    assert!(maintenance::gather().review.unwrap().is_none());
    let mut report = fixture.review();
    fs::write(fixture.report(), report.to_string()).unwrap();
    let review = maintenance::gather().review.unwrap().unwrap();
    assert_eq!(review.measured_games, 4);
    assert_eq!(review.games[0].allocated_bytes, 1073741824);
    assert!(review.complete);
    report["complete"] = json!(false);
    report["games"][0]["complete"] = json!(false);
    report["errors"] = json!([{"path":"/skipped","error":"permission denied"}]);
    fs::write(fixture.report(), report.to_string()).unwrap();
    let review = maintenance::gather().review.unwrap().unwrap();
    assert!(!review.complete);
    assert!(!review.games[0].complete);
    assert_eq!(review.errors, 1);
}

#[test]
fn invalid_oversized_and_future_reports_are_errors_not_empty_success() {
    let fixture = Fixture::new();
    let mut future = fixture.review();
    future["completed_at"] = json!(vitals::host::now_secs() + 10000);
    let mut negative = fixture.review();
    negative["measured_games"] = json!(-1);
    for content in ["invalid".to_string(), "{}".to_string(), "x".repeat(1048577), future.to_string(), negative.to_string()] {
        fs::write(fixture.report(), content).unwrap();
        assert!(maintenance::gather().review.is_err());
    }
}

#[test]
fn report_names_are_sanitized_and_summaries_bounded() {
    let fixture = Fixture::new();
    let mut report = fixture.review();
    report["games"] = json!((0..8).map(|index| json!({"path":format!("/game-{index}\n\u{1b}[31m"),"allocated_bytes":100,"complete":true})).collect::<Vec<_>>());
    fs::write(fixture.report(), report.to_string()).unwrap();
    let review = maintenance::gather().review.unwrap().unwrap();
    assert_eq!(review.games.len(), 3);
    assert!(!review.games[0].path.chars().any(char::is_control));
}

#[test]
fn read_only_check_requests_target_only_the_two_known_user_reports() {
    use maintenance::Check;
    let fixture = Fixture::new();
    maintenance::request_check(Check::SpaceReview).unwrap();
    maintenance::request_check(Check::TurboReport).unwrap();
    let calls = fs::read_to_string(fixture.root.join("calls")).unwrap();
    assert_eq!(calls.lines().collect::<Vec<_>>(), [
        "--user start --no-block disk-space-review.service",
        "--user start --no-block cache-maintenance-report.service",
    ]);
}

#[test]
fn failed_check_requests_return_the_native_error_without_claiming_completion() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("fail"), "").unwrap();
    let result = maintenance::request_check(maintenance::Check::SpaceReview);
    assert!(result.unwrap_err().contains("bus unavailable"));
}

#[test]
fn expected_symlink_skips_are_distinct_from_real_errors_and_legacy_unknowns() {
    let fixture = Fixture::new();
    let mut report = fixture.review();
    fs::write(fixture.report(), report.to_string()).unwrap();
    assert_eq!(maintenance::gather().review.unwrap().unwrap().skipped_symlinks, None);
    for skipped in [0, 92733] {
        report["skipped_symlinks"] = json!(skipped);
        fs::write(fixture.report(), report.to_string()).unwrap();
        let review = maintenance::gather().review.unwrap().unwrap();
        assert_eq!(review.skipped_symlinks, Some(skipped));
        assert_eq!(review.errors, 0);
    }
    report["errors"] = json!([{"path":"/protected","error":"permission denied"}]);
    report["complete"] = json!(false);
    fs::write(fixture.report(), report.to_string()).unwrap();
    let review = maintenance::gather().review.unwrap().unwrap();
    assert_eq!(review.errors, 1);
    assert!(!review.complete);
    report["skipped_symlinks"] = json!(-1);
    fs::write(fixture.report(), report.to_string()).unwrap();
    assert!(maintenance::gather().review.is_err());
}

#[test]
fn review_keeps_a_bounded_sanitized_sample_of_actual_read_issues() {
    let fixture = Fixture::new();
    let mut report = fixture.review();
    report["complete"] = json!(false);
    report["errors"] = json!((0..8).map(|index| json!({
        "path":format!("/project/protected-{index}\n\u{1b}[31m"),
        "error":"[Errno 13] Permission denied"
    })).collect::<Vec<_>>());
    fs::write(fixture.report(), report.to_string()).unwrap();
    let review = maintenance::gather().review.unwrap().unwrap();
    assert_eq!(review.errors, 8);
    assert_eq!(review.issues.len(), 3);
    assert!(!review.issues[0].path.chars().any(char::is_control));
    assert!(review.issues[0].error.contains("Permission denied"));
}
