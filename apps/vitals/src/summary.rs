//! The machine in plain words: one headline, then a sentence per area (cooling, disk, system,
//! backup, upkeep), each saying what it means and what to do when something drifts.

use crate::care::{self, Due};
use crate::store::Alert;

/// The pump's header on this board (B850 AORUS ELITE WIFI7, it87 driver): the one that spins.
pub const PUMP: &str = "fan:it8696/fan5";
/// The idle temperature may wander this much in two weeks; past it, dusting is near.
const SLIGHT_RISE: f64 = 2.0;
const CLEAR_RISE: f64 = 5.0;

#[derive(Default)]
pub struct Machine {
    pub alerts: Vec<Alert>,
    /// Median idle CPU temperature of the last two weeks, and of the two before.
    pub cpu_idle: (Option<f64>, Option<f64>),
    pub cpu_now: Option<f64>,
    pub pump_rpm: Option<f64>,
    /// device, % of rated writes used, healthy (no critical warning, spare above threshold)
    pub disk: Option<(String, u64, bool)>,
    /// The fullest file system: mount, % used.
    pub fullest: Option<(String, f64)>,
    pub failed_units: Vec<String>,
    pub kernel_problems: Vec<String>,
    pub backup_age_days: Option<i64>,
    /// Each upkeep task's title and due state.
    pub care: Vec<(&'static str, Due)>,
}

#[derive(Debug, PartialEq)]
pub struct Sentence {
    pub topic: &'static str,
    pub text: String,
    /// "ok", "warn" or "crit"
    pub level: &'static str,
}

fn sentence(topic: &'static str, level: &'static str, text: impl Into<String>) -> Sentence {
    Sentence { topic, text: text.into(), level }
}

pub fn headline(machine: &Machine) -> (String, &'static str) {
    if let Some(alert) = machine.alerts.iter().find(|alert| alert.level == "crit") {
        return (format!("À traiter maintenant : {}.", alert.title), "crit");
    }
    match machine.alerts.as_slice() {
        [] => ("Ton PC va bien : rien à faire.".into(), "ok"),
        [only] => (format!("Ton PC va bien, un point à regarder : {}.", only.title), "warn"),
        many => (format!("Ton PC va bien, {} points à regarder : {}.", many.len(),
                         many.iter().map(|alert| alert.title.as_str()).collect::<Vec<_>>().join(" ; ")), "warn"),
    }
}

fn cooling(machine: &Machine) -> Sentence {
    if machine.pump_rpm == Some(0.0) {
        return sentence("Refroidissement", "crit", "La pompe est à 0 tr/min : vérifie le watercooling tout de suite, avant de lancer quoi que ce soit de lourd.");
    }
    let pump = machine.pump_rpm.map(|rpm| format!(", la pompe tourne à {rpm:.0} tr/min")).unwrap_or_default();
    match machine.cpu_idle {
        (Some(recent), Some(before)) => {
            let rise = recent - before;
            if rise >= CLEAR_RISE {
                sentence("Refroidissement", "warn", format!("Ça chauffe davantage : le CPU est à {recent:.0} °C au repos, {rise:.0} °C de plus qu'il y a deux semaines{pump}. \
                    Dépoussière les filtres et le radiateur, puis vérifie la pompe ; si ça reste chaud, la pâte thermique est la suivante."))
            } else if rise >= SLIGHT_RISE {
                sentence("Refroidissement", "ok", format!("Correct, un peu plus chaud : le CPU est à {recent:.0} °C au repos (+{rise:.0} °C en deux semaines){pump}. \
                    Dépoussière bientôt si ça continue de monter."))
            } else {
                sentence("Refroidissement", "ok", format!("Normal : le CPU est à {recent:.0} °C au repos ({:+.0} °C en deux semaines){pump}.", rise))
            }
        }
        (Some(recent), None) => sentence("Refroidissement", "ok", format!("Le CPU est à {recent:.0} °C au repos{pump} ; encore deux semaines d'historique et la tendance apparaîtra.")),
        _ => sentence("Refroidissement", "ok", match machine.cpu_now {
            Some(now) => format!("Le CPU est à {now:.0} °C{pump} ; la tendance au repos a besoin de plus d'historique."),
            None => "Pas encore de température CPU.".into(),
        }),
    }
}

fn disk(machine: &Machine) -> Sentence {
    let space = machine.fullest.as_ref().map(|(mount, used)| format!(", le disque le plus rempli ({mount}) est plein à {used:.0} %")).unwrap_or_default();
    match &machine.disk {
        Some((device, _, false)) => sentence("Disque", "crit", format!("{device} signale un problème matériel : sauvegarde maintenant et regarde l'onglet Disques.")),
        Some((_, worn, true)) => sentence("Disque", if machine.fullest.as_ref().is_some_and(|(_, used)| *used >= 90.0) { "warn" } else { "ok" },
                                          format!("En bonne santé, {worn} % de son endurance en écriture utilisée{space}.")),
        None => sentence("Disque", "warn", format!("Pas encore de données de santé (vitals-root.timer){space}.")),
    }
}

fn system(machine: &Machine) -> Sentence {
    if !machine.kernel_problems.is_empty() {
        return sentence("Système", "crit", format!("Le noyau a signalé : {}. Regarde l'onglet Système.", machine.kernel_problems.join(", ")));
    }
    match machine.failed_units.as_slice() {
        [] => sentence("Système", "ok", "Tous les services tournent et le noyau n'a signalé aucune erreur matérielle."),
        units => sentence("Système", "warn", format!("{} service{} en échec ({}) : regarde l'onglet Système.", units.len(),
                                                     if units.len() == 1 { "" } else { "s" },
                                                     units.iter().map(|unit| unit.trim_end_matches(".service")).collect::<Vec<_>>().join(", "))),
    }
}

fn backup(machine: &Machine) -> Sentence {
    match machine.backup_age_days {
        Some(age) if age <= 2 => sentence("Sauvegarde", "ok", format!("La dernière sauvegarde réussie date de {}.", if age == 0 { "moins d'un jour".into() } else { care::days(age) })),
        Some(age) => sentence("Sauvegarde", "warn", format!("La dernière sauvegarde réussie date de {} : vérifie restic-backup.", care::days(age))),
        None => sentence("Sauvegarde", "warn", "Aucune sauvegarde réussie vue pour l'instant."),
    }
}

fn upkeep(machine: &Machine) -> Sentence {
    let overdue: Vec<String> = machine.care.iter().filter_map(|(title, due)| match due {
        Due::Overdue(days) => Some(format!("{} ({} de retard)", lowercase_first(title), care::days(*days))),
        _ => None,
    }).collect();
    if !overdue.is_empty() {
        return sentence("Entretien", "warn", format!("À faire : {}. Appuie sur d dans l'onglet Entretien une fois fait.", overdue.join(", ")));
    }
    let unknown = machine.care.iter().filter(|(_, due)| *due == Due::Unknown).count();
    let next = machine.care.iter().filter_map(|(title, due)| match due { Due::In(days) => Some((days, title)), _ => None }).min();
    let mut text = match next {
        Some((days, title)) => format!("Rien à faire ; prochain : {} dans {}.", lowercase_first(title), care::days(*days)),
        None => String::new(),
    };
    if unknown > 0 {
        text = format!("{text}{}{unknown} tâche{} sans date : indique quand tu l'as faite pour la dernière fois dans l'onglet Entretien.",
                       if text.is_empty() { "" } else { " " }, if unknown == 1 { "" } else { "s" });
    }
    sentence("Entretien", "ok", text)
}

/// "Dépoussiérer les filtres" in the middle of a sentence: only its first letter changes.
fn lowercase_first(text: &str) -> String {
    let mut characters = text.chars();
    characters.next().map(|first| first.to_lowercase().chain(characters).collect()).unwrap_or_default()
}

pub fn sentences(machine: &Machine) -> Vec<Sentence> {
    vec![cooling(machine), disk(machine), system(machine), backup(machine), upkeep(machine)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy() -> Machine {
        Machine { cpu_idle: (Some(48.0), Some(50.0)), pump_rpm: Some(2860.0), disk: Some(("nvme0n1".into(), 2, true)),
                  fullest: Some(("/".into(), 82.0)), backup_age_days: Some(0), care: vec![("Dépoussiérer les filtres et les ventilateurs", Due::In(60))],
                  ..Machine::default() }
    }

    #[test]
    fn a_healthy_machine_says_so_in_one_line() {
        assert_eq!(headline(&healthy()), ("Ton PC va bien : rien à faire.".into(), "ok"));
        assert!(sentences(&healthy()).iter().all(|line| line.level == "ok"));
    }

    #[test]
    fn a_warmer_idle_says_to_dust_and_check_the_pump() {
        let machine = Machine { cpu_idle: (Some(56.0), Some(50.0)), ..healthy() };
        let line = &sentences(&machine)[0];
        assert_eq!(line.level, "warn");
        assert!(line.text.contains("Dépoussière les filtres et le radiateur"), "{}", line.text);
    }

    #[test]
    fn a_stopped_pump_is_critical() {
        assert_eq!(sentences(&Machine { pump_rpm: Some(0.0), ..healthy() })[0].level, "crit");
    }

    #[test]
    fn failed_services_are_named_without_their_suffix() {
        let machine = Machine { failed_units: vec!["security-check.service".into(), "vdirsyncer.service".into()], ..healthy() };
        assert_eq!(sentences(&machine)[2].text, "2 services en échec (security-check, vdirsyncer) : regarde l'onglet Système.");
    }

    #[test]
    fn overdue_upkeep_is_named_with_how_late_it_is() {
        let machine = Machine { care: vec![("Dépoussiérer les filtres et les ventilateurs", Due::Overdue(12))], ..healthy() };
        assert_eq!(sentences(&machine)[4].text, "À faire : dépoussiérer les filtres et les ventilateurs (12 jours de retard). Appuie sur d dans l'onglet Entretien une fois fait.");
    }

    #[test]
    fn upkeep_without_dates_asks_for_them() {
        let machine = Machine { care: vec![("Dépoussiérer les filtres et les ventilateurs", Due::Unknown)], ..healthy() };
        assert!(sentences(&machine)[4].text.contains("1 tâche sans date"));
    }

    #[test]
    fn several_problems_are_listed_in_the_headline() {
        let alert = |title: &str| Alert { key: title.into(), level: "warn".into(), title: title.into(), detail: String::new(), since: 0, notified: true };
        let machine = Machine { alerts: vec![alert("2 service(s) en échec"), alert("Il est temps de dépoussiérer les filtres et les ventilateurs")], ..healthy() };
        assert_eq!(headline(&machine).0, "Ton PC va bien, 2 points à regarder : 2 service(s) en échec ; Il est temps de dépoussiérer les filtres et les ventilateurs.");
    }
}
