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
    Task { id: "dust", title: "Dépoussiérer les filtres et les ventilateurs", every_days: 90,
           why: "La poussière sur les filtres et les ventilateurs fait monter toutes les températures un peu plus chaque mois.",
           how: "PC éteint et débranché : un pinceau doux sur les filtres, de courtes rafales d'air sec sur les ventilateurs, bloqués à la main." },
    Task { id: "radiator", title: "Nettoyer le radiateur", every_days: 180,
           why: "La poussière tassée entre les ailettes du radiateur ralentit tout le circuit d'eau.",
           how: "PC éteint : air sec à travers le radiateur, de l'intérieur vers l'extérieur, ses ventilateurs bloqués." },
    Task { id: "watercooling", title: "Vérifier le watercooling", every_days: 180,
           why: "Une pompe qui fatigue ou un raccord qui suinte se remarque avant de lâcher.",
           how: "Écoute si la pompe claque ou fait plus de bruit, cherche de l'humidité le long des tuyaux et des raccords ; la vitesse de la pompe est dans Aperçu." },
    Task { id: "paste", title: "Refaire la pâte thermique du CPU", every_days: 3 * 365,
           why: "La pâte sèche avec les années et le CPU chauffe plus pour le même travail.",
           how: "À faire plus tôt si la tendance au repos de l'onglet Refroidissement continue de monter après un dépoussiérage." },
    Task { id: "restore", title: "Tester la restauration d'une sauvegarde", every_days: 180,
           why: "Une sauvegarde qu'on ne peut pas restaurer n'en est pas une.",
           how: "restic restore latest --target /tmp/restore-test --include ~/dotfiles, puis compare quelques fichiers." },
    Task { id: "bios", title: "Chercher une mise à jour du BIOS", every_days: 180,
           why: "Les mises à jour corrigent des bugs de stabilité et le pilotage des ventilateurs.",
           how: "Page support Gigabyte de la B850 AORUS ELITE WIFI7 ; lis les notes de version avant de flasher." },
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

/// Days written for a person: "aujourd'hui", "12 jours", "3 mois".
pub fn days(count: i64) -> String {
    match count {
        0 => "aujourd'hui".into(),
        1 => "1 jour".into(),
        many if many < 60 => format!("{many} jours"),
        many if many < 730 => format!("{} mois", many / 30),
        many => format!("{} ans", many / 365),
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
        assert_eq!((days(0), days(12), days(90), days(1095)), ("aujourd'hui".into(), "12 jours".into(), "3 mois".into(), "3 ans".into()));
    }
}
