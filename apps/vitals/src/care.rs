//! Upkeep only a person can do, on a schedule: dusting, the radiator, the water cooling, the
//! thermal paste, a restore test, the BIOS. The TUI's Care tab records when each was done (d);
//! a task past its interval becomes an alert like any other problem. A task never recorded raises
//! nothing: the TUI asks for its date instead.

use crate::store::Store;

pub struct Task {
    pub id: &'static str,
    pub title: &'static str,
    pub every_days: i64,
    pub why: &'static str,
    pub how: &'static str,
}

pub const TASKS: [Task; 6] = [
    Task { id: "dust", title: "Dust the filters and fans", every_days: 90,
           why: "Dust on the filters and fans raises every temperature a little more each month.",
           how: "PC off and unplugged: a soft brush on the filters, short bursts of compressed air on the fans, held still." },
    Task { id: "radiator", title: "Clean the radiator", every_days: 180,
           why: "Dust packed between the radiator's fins slows the whole water loop.",
           how: "PC off: compressed air through the radiator from the inside out, its fans held still." },
    Task { id: "watercooling", title: "Check the water cooling", every_days: 180,
           why: "A tiring pump or a weeping joint shows before it fails.",
           how: "Listen for a rattling or louder pump, look along the tubes and joints for moisture; the pump's speed is in Overview." },
    Task { id: "paste", title: "Renew the CPU thermal paste", every_days: 3 * 365,
           why: "Paste dries out over the years and the CPU runs warmer for the same work.",
           how: "Worth doing earlier when the Cooling tab's idle trend keeps climbing after a dusting." },
    Task { id: "restore", title: "Test restoring a backup", every_days: 180,
           why: "A backup that cannot be restored is not one.",
           how: "restic restore latest --target /tmp/restore-test --include ~/dotfiles, then compare a few files." },
    Task { id: "bios", title: "Check for a BIOS update", every_days: 180,
           why: "Updates fix stability bugs and fan control.",
           how: "Gigabyte's support page for the B850 AORUS ELITE WIFI7; read the release notes before flashing." },
];

pub const DAY: i64 = 86400;

#[derive(Clone, Debug, PartialEq)]
pub enum Due {
    /// Never recorded: the TUI asks when it was last done.
    Unknown,
    /// Days left before it is due.
    In(i64),
    /// Days past due.
    Overdue(i64),
}

pub fn due(task: &Task, last_done: Option<i64>, now: i64) -> Due {
    let Some(done) = last_done else { return Due::Unknown };
    let left = (done + task.every_days * DAY - now).div_euclid(DAY);
    if left >= 0 { Due::In(left) } else { Due::Overdue(-left) }
}

fn state_name(task: &Task) -> String {
    format!("care_done {}", task.id)
}

pub fn last_done(store: &Store, task: &Task) -> Option<i64> {
    store.state(&state_name(task)).ok().flatten().and_then(|stamp| stamp.parse().ok())
}

pub fn mark_done(store: &Store, task: &Task, at: i64) -> rusqlite::Result<()> {
    store.set_state(&state_name(task), &at.to_string())
}

/// Days written for a person: "today", "in 3 days", "12 days ago".
pub fn days(count: i64) -> String {
    match count {
        0 => "today".into(),
        1 => "1 day".into(),
        many if many < 60 => format!("{many} days"),
        many if many < 730 => format!("{} months", many / 30),
        many => format!("{} years", many / 365),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1000 * DAY;

    #[test]
    fn a_task_never_recorded_has_no_due_date() {
        assert_eq!(due(&TASKS[0], None, NOW), Due::Unknown);
    }

    #[test]
    fn a_task_done_recently_is_due_later() {
        assert_eq!(due(&TASKS[0], Some(NOW - 30 * DAY), NOW), Due::In(60));
    }

    #[test]
    fn a_task_past_its_interval_is_overdue() {
        assert_eq!(due(&TASKS[0], Some(NOW - 100 * DAY), NOW), Due::Overdue(10));
    }

    #[test]
    fn marking_a_task_done_restarts_its_interval() {
        let store = Store::in_memory().unwrap();
        mark_done(&store, &TASKS[0], NOW).unwrap();
        assert_eq!(due(&TASKS[0], last_done(&store, &TASKS[0]), NOW), Due::In(90));
    }

    #[test]
    fn days_read_like_a_person_says_them() {
        assert_eq!((days(0), days(12), days(90), days(1095)), ("today".into(), "12 days".into(), "3 months".into(), "3 years".into()));
    }
}
