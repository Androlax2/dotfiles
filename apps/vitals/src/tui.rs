//! The TUI: seven tabs over the history the collector keeps, plus live sensor readings.
//!
//!   1 Overview   what is wrong now, every temperature (now, 24 h range, limit, 24 h curve), fans, load
//!   2 Cooling    CPU and GPU over 90 days (daily coolest and hottest), the idle trend the checks use
//!   3 Disks      SMART health, space and read-only maintenance/review status
//!   4 System     failed services, kernel errors, memory, the backup
//!   5 Care       upkeep on a schedule (dusting, radiator, water cooling, paste, restore, BIOS):
//!                ↑ ↓ pick a task, d records it done today, b moves its date a week earlier
//!   6 Games      each game session with its peaks (games.rs)
//!   7 Journal    crashes, hard stops, memory kills, GPU hangs and the alerts that ended (journal.rs)
//!
//! The Overview opens with the machine in plain words (summary.rs). Tab / ← → or 1-7 switch, r
//! refreshes, q or Esc quits. Refreshes every 5 s on its own.

use crate::care::{self, Due};
use crate::checks::{self, DAY, TRENDED};
use crate::summary::{self, Machine};
use crate::facts::{self, Facts};
use crate::games::Stats;
use crate::journal;
use crate::maintenance::{self, Check, Maintenance, RunState};
use crate::sensors::{self, Kind, Reading, label};
use crate::store::{self, Alert, EndedAlert, Game, Store};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table, Tabs, Wrap};
use ratatui::{DefaultTerminal, Frame};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const REFRESH: Duration = Duration::from_secs(5);
const FACTS_REFRESH: Duration = Duration::from_secs(30);
const TABS: [&str; 7] = ["Aperçu", "Refroidissement", "Disques", "Système", "Entretien", "Jeux", "Journal"];
const CARE_TAB: usize = 4;
const DISKS_TAB: usize = 2;
const JOURNAL_DAYS: i64 = 30;
const GAME_DAYS: i64 = 90;
const WEEK: i64 = 7 * DAY;
const SPARK: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const CHART_DAYS: i64 = 90;

fn dim() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn level_style(level: &str) -> Style {
    match level {
        "crit" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        "warn" => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::Green),
    }
}

/// A row of block characters for values, scaled between their own low and high.
pub fn sparkline(values: &[f64]) -> String {
    let (Some(low), Some(high)) = (values.iter().copied().reduce(f64::min), values.iter().copied().reduce(f64::max)) else { return String::new() };
    let span = (high - low).max(1.0);
    values.iter().map(|value| SPARK[(((value - low) / span) * 7.0).round() as usize]).collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DiskScreen { Summary, Review, Details }

#[derive(Clone, Copy, Debug, PartialEq)]
enum DiskAction { ReadReview, SpaceCheck, TurboCheck, Details }

struct View {
    tab: usize,
    readings: Vec<Reading>,
    loads: BTreeMap<String, f64>,
    alerts: Vec<Alert>,
    /// sensor → its hourly (low, mean, high) over the last 24 h
    day: BTreeMap<String, Vec<(f64, f64, f64)>>,
    limits: checks::Limits,
    /// "cpu"/"gpu" → (day → coolest, hottest) over CHART_DAYS
    long: BTreeMap<&'static str, Vec<(f64, f64, f64)>>,
    /// "cpu"/"gpu" → (median idle of the last 14 days, of the 14 before)
    idle: BTreeMap<&'static str, (Option<f64>, Option<f64>)>,
    facts: Facts,
    facts_at: Option<Instant>,
    error: Option<String>,
    collected_at: Option<i64>,
    /// Each upkeep task's last recorded date, in care::TASKS order.
    care_done: Vec<Option<i64>>,
    care_selected: usize,
    last_backup: Option<i64>,
    events: Vec<store::Event>,
    ended_alerts: Vec<EndedAlert>,
    games: Vec<Game>,
    /// mount → days until full at the pace of the last two weeks
    fill_forecast: BTreeMap<String, f64>,
    maintenance: Option<Maintenance>,
    disk_scroll: u16,
    disk_scroll_max: u16,
    disk_page: u16,
    disk_screen: DiskScreen,
    disk_selected: Option<DiskAction>,
    disk_feedback: Option<Result<&'static str, String>>,
}

fn when(at: i64) -> String {
    use chrono::TimeZone;
    chrono::Local.timestamp_opt(at, 0).single().map(|time| time.format("%d/%m %H:%M").to_string()).unwrap_or_default()
}

fn duration(seconds: i64) -> String {
    let minutes = seconds / 60;
    if minutes < 60 { format!("{minutes} min") } else { format!("{} h {:02}", minutes / 60, minutes % 60) }
}

fn degrees(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.0} °C")).unwrap_or_else(|| "–".into())
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(values[values.len() / 2])
}

impl View {
    fn new() -> View {
        View { tab: 0, readings: Vec::new(), loads: BTreeMap::new(), alerts: Vec::new(), day: BTreeMap::new(), limits: BTreeMap::new(),
               long: BTreeMap::new(), idle: BTreeMap::new(), facts: Facts::default(), facts_at: None, error: None, collected_at: None,
               care_done: vec![None; care::TASKS.len()], care_selected: 0, last_backup: None, events: Vec::new(), ended_alerts: Vec::new(),
                games: Vec::new(), fill_forecast: BTreeMap::new(), maintenance: None,
                 disk_scroll: 0, disk_scroll_max: 0, disk_page: 1,
                 disk_screen: DiskScreen::Summary, disk_selected: None, disk_feedback: None }
    }

    fn refresh_facts_with(&mut self, read: impl FnOnce() -> (Facts, Maintenance)) {
        if self.facts_at.is_none_or(|at| at.elapsed() >= FACTS_REFRESH) {
            let (facts, maintenance) = read();
            self.facts = facts;
            self.maintenance = Some(maintenance);
            self.facts_at = Some(Instant::now());
        }
    }

    fn refresh(&mut self) {
        let root = sensors::sys_root();
        self.readings = sensors::read_hwmon(&root);
        self.refresh_facts_with(|| (facts::gather(&root), maintenance::gather()));
        match self.load_history() {
            Ok(()) => self.error = None,
            Err(error) => self.error = Some(format!("historique : {error}")),
        }
    }

    fn load_history(&mut self) -> rusqlite::Result<()> {
        let store = Store::open()?;
        let now = cockpit::util::now_secs();
        self.alerts = store.alerts()?;
        self.care_done = care::TASKS.iter().map(|task| care::last_done(&store, task)).collect();
        self.last_backup = self.facts.last_backup.or_else(|| store.state("last_backup").ok().flatten().and_then(|stamp| stamp.parse().ok()));
        self.limits = store.limits()?;
        self.events = store.events(now - JOURNAL_DAYS * DAY)?;
        self.ended_alerts = store.ended_alerts(now - JOURNAL_DAYS * DAY)?;
        self.games = store.games(now - GAME_DAYS * DAY)?;
        self.fill_forecast = self.facts.disk_usage.iter().filter_map(|(mount, _)| {
            let history: Vec<(i64, f64)> = store.hours(&crate::collect::disk_key(mount), now - 14 * DAY).ok()?
                .into_iter().map(|(hour, _, mean, _)| (hour, mean)).collect();
            Some((mount.clone(), checks::days_until_full(&history)?))
        }).collect();
        let latest = store.latest()?;
        self.collected_at = latest.values().map(|(at, _)| *at).max();
        self.loads = latest.iter().filter(|(sensor, _)| ["load:", "power:", "clock:", "vram:"].iter().any(|kind| sensor.starts_with(kind))).map(|(sensor, (_, value))| (sensor.clone(), *value)).collect();
        self.day = self.readings.iter().map(|reading| {
            let hours = store.hours(&reading.key, now - DAY).unwrap_or_default();
            (reading.key.clone(), hours.into_iter().map(|(_, low, mean, high)| (low, mean, high)).collect())
        }).collect();
        for (sensor, load, name) in TRENDED {
            let mut by_day: BTreeMap<i64, (f64, f64)> = BTreeMap::new();
            for (hour, low, _, high) in store.hours(sensor, now - CHART_DAYS * DAY)? {
                let entry = by_day.entry(hour.div_euclid(DAY)).or_insert((low, high));
                entry.0 = entry.0.min(low);
                entry.1 = entry.1.max(high);
            }
            self.long.insert(name, by_day.into_iter().map(|(day, (low, high))| (day as f64, low, high)).collect());
            let minutes = store.minutes(now - 29 * DAY, &[sensor, load])?;
            let baselines = checks::idle_baselines(&minutes, sensor, load);
            let today = now.div_euclid(DAY);
            let window = |from: i64, to: i64| median(baselines.range(today - to..today - from).map(|(_, value)| *value).collect());
            self.idle.insert(name, (window(0, 14), window(14, 28)));
        }
        Ok(())
    }
}

fn header(frame: &mut Frame, area: Rect, view: &View) {
    let guided = view.tab == DISKS_TAB && view.disk_screen != DiskScreen::Details;
    let [tabs_area, state_area] = Layout::horizontal([Constraint::Min(if guided { 0 } else { 30 }), Constraint::Length(if guided { 27 } else { 40 })]).areas(area);
    let titles: Vec<Line> = TABS.iter().enumerate().map(|(index, title)| Line::from(format!("{} {title}", index + 1))).collect();
    frame.render_widget(Tabs::new(titles).select(view.tab).highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)), tabs_area);
    let severity = view.alerts.iter().map(|alert| alert.level.as_str()).max_by_key(|level| *level == "crit");
    let state = if guided {
        match (severity, view.collected_at, &view.error) {
            (Some("crit"), _, _) => Span::styled("PC : alerte importante", level_style("crit")),
            (Some(_), _, _) => Span::styled("PC : point à vérifier", level_style("warn")),
            (_, None, _) | (_, _, Some(_)) => Span::styled("PC : état incomplet", level_style("warn")),
            _ => Span::styled("PC : aucune alerte connue", level_style("ok")),
        }
    } else { match (severity, view.collected_at) {
        (_, None) => Span::styled("aucune collecte pour l'instant", level_style("warn")),
        (None, Some(at)) => Span::styled(format!("tout va bien · collecté il y a {} s", cockpit::util::now_secs() - at), level_style("ok")),
        (Some(level), Some(_)) => Span::styled(format!("{} problème(s)", view.alerts.len()), level_style(level)),
    } };
    frame.render_widget(Paragraph::new(Line::from(state)).right_aligned(), state_area);
}

fn alerts_lines(view: &View) -> Vec<Line<'_>> {
    view.alerts.iter().flat_map(|alert| [
        Line::from(vec![Span::styled(if alert.level == "crit" { "● " } else { "▲ " }, level_style(&alert.level)),
                        Span::styled(alert.title.as_str(), Style::default().add_modifier(Modifier::BOLD))]),
        Line::styled(format!("  {}", alert.detail), dim()),
    ]).collect()
}

/// What the summary needs, from what the view holds.
fn machine(view: &View) -> Machine {
    let now = cockpit::util::now_secs();
    let reading = |key: &str| view.readings.iter().find(|reading| reading.key == key).map(|reading| reading.value);
    let kernel_problems = view.facts.kernel_errors.keys().map(|category| match category.as_str() {
        "gpu" => "réinitialisations du GPU", "disk" => "erreurs d'entrée/sortie disque", "hardware" => "erreurs matérielles",
        "memory" => "programmes tués faute de mémoire", _ => "bridage thermique",
    }.to_string()).collect();
    Machine {
        alerts: view.alerts.clone(),
        cpu_idle: view.idle.get("cpu").copied().unwrap_or((None, None)),
        cpu_now: reading("temp:k10temp/Tctl"),
        pump_rpm: reading(summary::PUMP),
        // The disk in the worst shape speaks for all: a failing one first, then the most worn.
        disk: view.facts.disks.iter()
            .map(|disk| (disk.device.clone(), disk.percentage_used, disk.critical_warning == 0 && disk.available_spare >= disk.spare_threshold))
            .min_by_key(|(_, worn, healthy)| (*healthy, std::cmp::Reverse(*worn))),
        fullest: view.facts.disk_usage.iter().cloned().max_by(|a, b| a.1.total_cmp(&b.1)),
        failed_units: view.facts.failed_units.clone(),
        kernel_problems,
        backup_age_days: view.last_backup.map(|at| (now - at).div_euclid(DAY)),
        care: care::TASKS.iter().zip(&view.care_done).map(|(task, done)| (task.title, care::due(task, *done, now))).collect(),
        recent_events: view.events.iter().filter(|event| event.at >= now - 7 * DAY).map(|event| event.kind.clone()).collect(),
    }
}

fn summary_lines(view: &View) -> Vec<Line<'static>> {
    let machine = machine(view);
    let (headline, level) = summary::headline(&machine);
    let mut lines = vec![Line::styled(headline, level_style(level).add_modifier(Modifier::BOLD)), Line::from("")];
    for sentence in summary::sentences(&machine) {
        lines.push(Line::from(vec![Span::styled(format!("{:<17}", sentence.topic), level_style(sentence.level).add_modifier(Modifier::BOLD)),
                                   Span::raw(sentence.text)]));
    }
    lines
}

fn overview(frame: &mut Frame, area: Rect, view: &View) {
    let mut summary: Vec<Line> = summary_lines(view);
    if !view.alerts.is_empty() {
        summary.push(Line::from(""));
        summary.extend(alerts_lines(view));
    }
    // Sized to its text as wrapped in this window, borders included.
    let inner_width = area.width.saturating_sub(2).max(1) as usize;
    let wrapped: usize = summary.iter().map(|line| line.width().div_ceil(inner_width).max(1)).sum();
    let height = (wrapped as u16 + 2).min(area.height * 2 / 3);
    let [summary_area, table_area] = Layout::vertical([Constraint::Length(height), Constraint::Min(5)]).areas(area);
    frame.render_widget(Paragraph::new(summary).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).title(" Ton PC ")), summary_area);
    let mut rows = Vec::new();
    let mut temperatures: Vec<&Reading> = view.readings.iter().filter(|reading| reading.kind == Kind::Temperature).collect();
    temperatures.sort_by_key(|reading| sensors::rank(&reading.key));
    for reading in temperatures {
        let day = view.day.get(&reading.key).cloned().unwrap_or_default();
        let range = match (day.iter().map(|hour| hour.0).reduce(f64::min), day.iter().map(|hour| hour.2).reduce(f64::max)) {
            (Some(low), Some(high)) => format!("{low:.0}–{high:.0}°C"),
            _ => "–".into(),
        };
        let limit = checks::high_of(&reading.key, &view.limits);
        let hot = reading.value >= limit;
        rows.push(Row::new(vec![
            Cell::from(label(&reading.key)),
            Cell::from(format!("{:.1}°C", reading.value)).style(if hot { level_style("warn") } else { Style::default().add_modifier(Modifier::BOLD) }),
            Cell::from(range).style(dim()),
            Cell::from(format!("{limit:.0}°C")).style(dim()),
            Cell::from(sparkline(&day.iter().map(|hour| hour.1).collect::<Vec<_>>())).style(Style::default().fg(Color::Cyan)),
        ]));
    }
    for reading in view.readings.iter().filter(|reading| reading.kind == Kind::Fan) {
        rows.push(Row::new(vec![Cell::from(label(&reading.key)), Cell::from(format!("{:.0} tr/min", reading.value)), Cell::from(""), Cell::from(""), Cell::from("")]));
    }
    for (sensor, value) in &view.loads {
        let shown = match sensor.split(':').next() {
            Some("power") => format!("{value:.0} W"),
            Some("clock") => format!("{value:.0} MHz"),
            _ => format!("{value:.0} %"),
        };
        rows.push(Row::new(vec![Cell::from(label(sensor)), Cell::from(shown), Cell::from(""), Cell::from(""), Cell::from("")]));
    }
    let table = Table::new(rows, [Constraint::Length(18), Constraint::Length(10), Constraint::Length(12), Constraint::Length(7), Constraint::Min(24)])
        .header(Row::new(["Capteur", "Actuel", "24 h", "Limite", "Dernières 24 h"]).style(dim()))
        .block(Block::default().borders(Borders::ALL).title(" Capteurs "));
    frame.render_widget(table, table_area);
}

fn cooling(frame: &mut Frame, area: Rect, view: &View) {
    let areas = Layout::vertical([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(area);
    for (index, (_, _, name)) in TRENDED.iter().enumerate() {
        let days = view.long.get(name).cloned().unwrap_or_default();
        let coolest: Vec<(f64, f64)> = days.iter().map(|(day, low, _)| (*day, *low)).collect();
        let hottest: Vec<(f64, f64)> = days.iter().map(|(day, _, high)| (*day, *high)).collect();
        let (first, last) = (days.first().map(|day| day.0).unwrap_or(0.0), days.last().map(|day| day.0).unwrap_or(1.0));
        let top = days.iter().map(|day| day.2).fold(50.0, f64::max).ceil() + 5.0;
        let (recent, before) = view.idle.get(name).copied().unwrap_or((None, None));
        let idle = match (recent, before) {
            (Some(recent), Some(before)) => format!("repos {recent:.1} °C (deux semaines avant : {before:.1} °C, {:+.1})", recent - before),
            (Some(recent), None) => format!("repos {recent:.1} °C (pas encore assez d'historique pour comparer)"),
            _ => "repos : pas encore assez d'historique".into(),
        };
        let datasets = vec![
            Dataset::default().name("plus chaud").marker(Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(Color::Red)).data(&hottest),
            Dataset::default().name("plus frais").marker(Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(Color::Cyan)).data(&coolest),
        ];
        let chart = Chart::new(datasets)
            .block(Block::default().borders(Borders::ALL).title(format!(" {} · {CHART_DAYS} jours · {idle} ", name.to_uppercase())))
            .x_axis(Axis::default().bounds([first, last.max(first + 1.0)]).labels([format!("il y a {} jours", (last - first) as i64), "aujourd'hui".into()]).style(dim()))
            .y_axis(Axis::default().bounds([20.0, top]).labels(["20°C".to_string(), format!("{:.0}°C", (20.0 + top) / 2.0), format!("{top:.0}°C")]).style(dim()));
        frame.render_widget(chart, areas[index]);
    }
}

fn maintenance_time(at: Option<i64>) -> String {
    at.map(when).filter(|value| !value.is_empty()).unwrap_or_else(|| "indisponible".into())
}

fn space_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 { format!("{:.2} Gio", bytes as f64 / 1_073_741_824.0) }
    else if bytes >= 1024 * 1024 { format!("{:.1} Mio", bytes as f64 / 1_048_576.0) }
    else if bytes >= 1024 { format!("{:.1} Kio", bytes as f64 / 1024.0) }
    else { format!("{bytes} octets") }
}

fn maintenance_lines(snapshot: Option<&Maintenance>) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(""), Line::styled("Maintenance", Style::default().add_modifier(Modifier::BOLD))];
    let Some(snapshot) = snapshot else {
        lines.push(Line::styled("  données de maintenance indisponibles", level_style("warn")));
        return lines;
    };
    for task in &snapshot.tasks {
        let enabled = match task.enabled { Some(true) => "activé", Some(false) => "désactivé", None => "activation inconnue" };
        let (result, style) = match task.state {
            RunState::Running => ("en cours", Style::default().fg(Color::Cyan)),
            RunState::Succeeded => ("réussi", level_style("ok")),
            RunState::Failed => ("échec", level_style("crit")),
            RunState::Unknown => ("résultat inconnu", dim()),
        };
        lines.push(Line::from(vec![Span::styled(format!("  {}", task.label), Style::default().add_modifier(Modifier::BOLD)),
                                  Span::raw(format!(" · {enabled} · ")), Span::styled(result, style)]));
        lines.push(Line::from(format!("    dernier : {} · prochain : {}", maintenance_time(task.last_run), maintenance_time(task.next_run))));
        lines.push(Line::styled(format!("    politique approuvée : {}", task.policy), dim()));
        if let Some(detail) = &task.detail { lines.push(Line::styled(format!("    {detail}"), level_style("warn"))); }
    }
    lines.push(Line::from(match &snapshot.journal_retention {
        Ok(Some(value)) => format!("  Journald · rétention configurée : {value} (rotation native)"),
        Ok(None) => "  Journald · politique native, sans durée explicite configurée".into(),
        Err(error) => format!("  Journald · rétention inconnue : {error}"),
    }));
    lines.push(Line::from(""));
    lines.push(Line::styled("Revue mensuelle", Style::default().add_modifier(Modifier::BOLD)));
    match &snapshot.review {
        Ok(Some(review)) => {
            lines.push(Line::from(format!("  {} · revue {} · {} jeux et {} projets mesurés", when(review.completed_at),
                                         if review.complete { "complète" } else { "partielle" }, review.measured_games, review.measured_checkouts)));
            for (category, entries) in [("Jeux", &review.games), ("Projets", &review.checkouts)] {
                lines.push(Line::from(format!("  {category} · les plus volumineux (jusqu'à 3)")));
                for entry in entries {
                    lines.push(Line::from(format!("    {}{} · {}", space_size(entry.allocated_bytes),
                                                 if entry.complete { "" } else { " (partiel)" }, entry.path)));
                }
                if entries.is_empty() { lines.push(Line::styled("    aucune entrée mesurée disponible", dim())); }
            }
            let skipped = review.skipped_symlinks.map(|count| format!("{count} liens ignorés (normal)"))
                .unwrap_or_else(|| "exclusions non détaillées (ancien bilan)".into());
            lines.push(Line::styled(format!("  {} erreur(s) · {skipped} · {} chemin(s) indisponible(s) · {} jeux / {} projets omis du rapport",
                                          review.errors, review.unavailable, review.omitted_games, review.omitted_checkouts),
                                    if review.complete { dim() } else { level_style("warn") }));
            for issue in &review.issues {
                lines.push(Line::styled(format!("  {} : {}", issue.error, issue.path), level_style("warn")));
            }
        }
        Ok(None) => {
            let running = snapshot.tasks.iter().any(|task| task.timer == "disk-space-review.timer" && task.state == RunState::Running);
            lines.push(Line::styled(if running { "  première revue en cours · aucun résultat publié" } else { "  aucune revue disponible" }, dim()));
        }
        Err(error) => lines.push(Line::styled(format!("  revue indisponible : {error}"), level_style("warn"))),
    }
    lines.push(Line::styled("  Estimations non additives · aucune suppression sans ton accord.", dim()));
    lines
}

fn wrap_disk_lines(lines: Vec<Line<'_>>, width: u16) -> Vec<Line<'static>> {
    let mut wrapped = Vec::new();
    for line in lines {
        let mut current = Line::default();
        let mut occupied = 0;
        for grapheme in line.styled_graphemes(Style::default()) {
            let size = Span::raw(grapheme.symbol).width();
            if occupied > 0 && occupied + size > usize::from(width.max(1)) {
                wrapped.push(current);
                current = Line::default();
                occupied = 0;
            }
            current.spans.push(Span::styled(grapheme.symbol.to_string(), grapheme.style));
            occupied += size;
        }
        wrapped.push(current);
    }
    wrapped
}

fn disk_details(frame: &mut Frame, area: Rect, view: &mut View) {
    let mut lines = vec![Line::styled("Rapports manuels · lecture seule", Style::default().add_modifier(Modifier::BOLD)),
                         Line::from("a : bilan d’espace · t : rapport Turbo · aucun fichier supprimé")];
    if let Some(feedback) = &view.disk_feedback {
        lines.push(match feedback {
            Ok(message) => Line::styled(*message, Style::default().fg(Color::Cyan)),
            Err(error) => Line::styled(format!("Demande non lancée : {error}"), level_style("warn")),
        });
    }
    lines.push(Line::from(""));
    if view.facts.disks.is_empty() {
        lines.push(Line::styled("Pas de données SMART : vitals-root.timer (une unité système) les écrit dans /var/lib/vitals/smart.", level_style("warn")));
    }
    for disk in &view.facts.disks {
        let healthy = disk.critical_warning == 0 && disk.available_spare >= disk.spare_threshold;
        lines.push(Line::from(vec![Span::styled(format!("{}  ", disk.device), Style::default().add_modifier(Modifier::BOLD)),
                                   Span::styled(if healthy { "en bonne santé" } else { "DÉFAILLANT" }, level_style(if healthy { "ok" } else { "crit" }))]));
        lines.push(Line::from(format!("  usure {} %   réserve {} % (seuil {} %)   erreurs de support {}   avertissement critique {:#04x}",
                                      disk.percentage_used, disk.available_spare, disk.spare_threshold, disk.media_errors, disk.critical_warning)));
        lines.push(Line::styled(format!("  lu il y a {} min", (cockpit::util::now_secs() - disk.read_at) / 60), dim()));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled("Espace", Style::default().add_modifier(Modifier::BOLD)));
    for (mount, used) in &view.facts.disk_usage {
        let filled = (*used / 5.0).round() as usize;
        let forecast = match view.fill_forecast.get(mount) {
            Some(days) if *days <= 365.0 => Span::styled(format!("   plein dans ~{days:.0} jours à ce rythme"), level_style(if *days <= 30.0 { "warn" } else { "ok" })),
            _ => Span::raw(""),
        };
        lines.push(Line::from(vec![Span::raw(format!("  {mount:<16} ")), Span::styled("█".repeat(filled), level_style(if *used >= 90.0 { "warn" } else { "ok" })),
                                   Span::styled("░".repeat(20 - filled.min(20)), dim()), Span::raw(format!(" {used:.0} %")), forecast]));
    }
    lines.extend(maintenance_lines(view.maintenance.as_ref()));
    let block = Block::default().borders(Borders::ALL).title(" Disques · ↑↓ faire défiler ");
    let inner = block.inner(area);
    let lines = wrap_disk_lines(lines, inner.width);
    view.disk_scroll_max = lines.len().saturating_sub(usize::from(inner.height)).min(usize::from(u16::MAX)) as u16;
    view.disk_scroll = view.disk_scroll.min(view.disk_scroll_max);
    view.disk_page = inner.height.saturating_sub(1).max(1);
    frame.render_widget(Paragraph::new(lines).scroll((view.disk_scroll, 0)).block(block), area);
}

struct DiskOverview {
    level: &'static str,
    title: &'static str,
    hint: String,
    primary: Option<DiskAction>,
    can_run_checks: bool,
}

const INCOMPLETE_REVIEW: &str = "Le bilan est incomplet : certains dossiers n’ont pas pu être mesurés. Aucun fichier n’a été modifié.";

fn disk_overview(view: &View) -> DiskOverview {
    let overview = |level, title, hint: String, primary, can_run_checks| DiskOverview { level, title, hint, primary: Some(primary), can_run_checks };
    let health = checks::smart(&view.facts.disks, cockpit::util::now_secs());
    if health.iter().any(|finding| finding.level == "crit") {
        return overview("crit", "Vérifier le SSD en priorité", "Vérifie ta sauvegarde avant toute autre intervention.".into(), DiskAction::Details, false);
    }
    let fullest = view.facts.disk_usage.iter().filter(|(_, used)| used.is_finite() && *used >= 0.0)
        .max_by(|left, right| left.1.total_cmp(&right.1));
    let Some((mount, used)) = fullest else {
        return overview("warn", "Mesure d’espace indisponible", "Actualise les données avec r ou consulte les détails.".into(), DiskAction::Details, true);
    };
    let has_review = view.maintenance.as_ref().is_some_and(|snapshot| matches!(snapshot.review, Ok(Some(_))));
    let first_review_running = !has_review && check_is_running(view, Check::SpaceReview);
    let space_action = if mount.starts_with("/boot") || first_review_running { DiskAction::Details }
        else if has_review { DiskAction::ReadReview }
        else { DiskAction::SpaceCheck };
    let space_hint = if first_review_running { "Bilan en cours · notification à venir.".into() }
        else { format!("{used:.0} % utilisés sur le volume le plus rempli. Vérifie avant de supprimer.") };
    if *used >= 95.0 {
        return overview("crit", "Espace presque plein", space_hint, space_action, true);
    }
    if view.facts.disks.is_empty() || !health.is_empty() {
        return overview("warn", "Santé du SSD à vérifier", "Les données de santé sont absentes, anciennes ou signalent un problème. Consulte les détails.".into(), DiskAction::Details, true);
    }
    if *used >= 85.0 {
        return overview("warn", "Espace à surveiller", space_hint, space_action, true);
    }
    let Some(snapshot) = &view.maintenance else {
        return overview("warn", "Maintenance non vérifiée", "Les résultats ne sont pas encore disponibles. Actualise avec r.".into(), DiskAction::Details, true);
    };
    if snapshot.tasks.iter().any(|task| (task.timer != "disk-space-review.timer" && task.state == RunState::Failed)
        || (task.timer == "paccache.timer" && task.enabled != Some(true))) {
        return overview("warn", "Maintenance à vérifier", "Une tâche a échoué ou le nettoyage n’est pas confirmé actif. Consulte les détails.".into(), DiskAction::Details, true);
    }
    if snapshot.tasks.iter().any(|task| task.timer == "disk-space-review.timer" && task.state == RunState::Running) {
        return DiskOverview { level: "info", title: "Rien à faire pour le moment",
            hint: "Bilan en cours. Tu peux fermer Vitals : une notification arrivera à la fin.".into(),
            primary: matches!(snapshot.review, Ok(Some(_))).then_some(DiskAction::ReadReview), can_run_checks: true };
    }
    match &snapshot.review {
        Ok(Some(review)) if review.errors > 0 && review.skipped_symlinks.is_none() => {
            return overview("warn", "Bilan à actualiser", "Les limites de cet ancien bilan sont inconnues. Prépare un nouveau bilan en lecture seule.".into(), DiskAction::SpaceCheck, true);
        }
        Ok(Some(review)) if review.errors > 0 => {
            return overview("warn", "Bilan à vérifier", INCOMPLETE_REVIEW.into(), DiskAction::ReadReview, true);
        }
        Ok(Some(review)) if !review.complete || review.unavailable > 0 => {
            return overview("warn", "Bilan incomplet", INCOMPLETE_REVIEW.into(), DiskAction::ReadReview, true);
        }
        Ok(Some(review)) if review.skipped_symlinks.is_none() => {
            return overview("warn", "Bilan à actualiser", "Les limites de cet ancien bilan sont inconnues. Prépare un nouveau bilan en lecture seule.".into(), DiskAction::SpaceCheck, true);
        }
        Ok(None) | Err(_) => {
            if snapshot.tasks.iter().any(|task| task.timer == "disk-space-review.timer" && task.state == RunState::Failed) {
                return overview("warn", "Bilan à vérifier", "L’analyse n’a pas abouti. Consulte le résultat avant de la relancer.".into(), DiskAction::Details, true);
            }
            return overview("info", "Bilan à actualiser", "Une analyse en lecture seule permettra de voir ce qui occupe le disque.".into(), DiskAction::SpaceCheck, true);
        }
        _ => {}
    }
    if snapshot.tasks.iter().any(|task| task.state == RunState::Failed) {
        return overview("warn", "Maintenance à vérifier", "Une tâche a échoué ou le nettoyage n’est pas confirmé actif. Consulte les détails.".into(), DiskAction::Details, true);
    }
    if view.fill_forecast.values().any(|days| *days <= 30.0) {
        return overview("warn", "L’espace se remplit rapidement", "Consulte le bilan avant de manquer de place.".into(), DiskAction::ReadReview, true);
    }
    overview("ok", "Aucune action urgente", format!("{used:.0} % utilisés sur le volume le plus rempli."), DiskAction::ReadReview, true)
}

fn disk_actions(view: &View) -> Vec<DiskAction> {
    disk_overview(view).primary.into_iter().collect()
}

fn disk_action_text(action: DiskAction) -> (&'static str, &'static str) {
    match action {
        DiskAction::ReadReview => ("Lire le bilan d’espace", "Voir les jeux et projets volumineux. Rien n’est supprimé ici."),
        DiskAction::SpaceCheck => ("Préparer le bilan", "Lecture seule · jusqu’à 15 min. Une notification préviendra quand ce sera prêt."),
        DiskAction::TurboCheck => ("Vérifier le cache Turbo", "Un simple rapport : aucun cache n’est effacé."),
        DiskAction::Details => ("Comprendre l’avertissement", "Lire la santé du SSD et le résultat des tâches, sans lancer de nettoyage."),
    }
}

fn check_is_running(view: &View, check: Check) -> bool {
    let timer = match check { Check::SpaceReview => "disk-space-review.timer", Check::TurboReport => "cache-maintenance-report.timer" };
    view.maintenance.as_ref().is_some_and(|snapshot| snapshot.tasks.iter().any(|task| task.timer == timer && task.state == RunState::Running))
}

fn disk_summary(frame: &mut Frame, area: Rect, view: &mut View) {
    let block = Block::default().borders(Borders::ALL).title(" Disques · l’essentiel ");
    let inner = block.inner(area);
    let status = disk_overview(view);
    let style = if status.level == "info" { Style::default().fg(Color::Cyan) } else { level_style(status.level) };
    let compact = inner.height < 18;
    let has_warning = matches!(status.level, "warn" | "crit");
    let feedback = view.disk_feedback.as_ref().filter(|_| status.primary.is_some());
    let request_failed = matches!(feedback, Some(Err(_))) && !has_warning;
    let title = if request_failed { "Vérification non lancée" } else { status.title };
    let mut lines = wrap_disk_lines(vec![Line::styled(title,
        if request_failed { level_style("warn").add_modifier(Modifier::BOLD) } else { style.add_modifier(Modifier::BOLD) })], inner.width);
    if has_warning || feedback.is_none() {
        lines.extend(wrap_disk_lines(vec![Line::from(status.hint)], inner.width).into_iter().take(if status.primary.is_some() && (compact || feedback.is_some()) { 1 } else { 2 }));
    }
    if let Some(feedback) = feedback.filter(|_| status.level != "crit") {
        let (message, style) = match feedback {
            Ok(message) => (message.to_string(), Style::default().fg(Color::Cyan)),
            Err(_) => ("Demande refusée. d : détails du problème.".into(), level_style("warn")),
        };
        lines.extend(wrap_disk_lines(vec![Line::styled(message, style)], inner.width).into_iter().take(if compact || has_warning { 1 } else { 2 }));
    }
    if let Some(action) = status.primary {
        lines.push(Line::from(""));
        let heading = if action == DiskAction::ReadReview && matches!(status.level, "ok" | "info") { "Si tu veux libérer de la place" } else { "Prochain pas" };
        lines.push(Line::styled(heading, Style::default().add_modifier(Modifier::BOLD)));
        let (label, hint) = disk_action_text(action);
        lines.extend(wrap_disk_lines(vec![Line::styled(format!("> {label}"), Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))], inner.width));
        if !compact { lines.extend(wrap_disk_lines(vec![Line::from(hint)], inner.width)); }
    }
    view.disk_scroll = 0;
    view.disk_scroll_max = 0;
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn disk_review(frame: &mut Frame, area: Rect, view: &mut View) {
    let mut lines = vec![Line::styled("Ton prochain pas : choisir un élément à vérifier", Style::default().add_modifier(Modifier::BOLD)),
                         Line::from("Un gros dossier n’est pas forcément inutile. Rien n’est supprimé ici."), Line::from("")];
    match view.maintenance.as_ref().map(|snapshot| &snapshot.review) {
        Some(Ok(Some(review))) => {
            if !review.complete || review.errors > 0 || review.unavailable > 0 {
                lines.push(Line::styled(INCOMPLETE_REVIEW, level_style("warn")));
            } else if review.skipped_symlinks.is_none() {
                lines.push(Line::styled("Les limites de cet ancien bilan sont inconnues.", level_style("warn")));
            }
            lines.push(Line::from("d : détails techniques et rapports manuels."));
            for (title, entries) in [("Jeux à vérifier", &review.games), ("Projets à vérifier", &review.checkouts)] {
                lines.push(Line::from(""));
                lines.push(Line::styled(title, Style::default().add_modifier(Modifier::BOLD)));
                for entry in entries {
                    let path = std::path::Path::new(&entry.path);
                    let basename = path.file_name().and_then(|name| name.to_str()).unwrap_or(&entry.path);
                    let name = if entry.path.contains("/.parallel/") {
                        path.parent().and_then(|parent| parent.file_name()).and_then(|name| name.to_str())
                            .map(|project| format!("{project} / {basename}")).unwrap_or_else(|| basename.to_string())
                    } else { basename.to_string() };
                    lines.push(Line::from(format!("  {name} · {}{}", space_size(entry.allocated_bytes), if entry.complete { "" } else { " (estimation partielle)" })));
                }
                if entries.is_empty() { lines.push(Line::from("  Aucun élément mesuré disponible.")); }
            }
            lines.push(Line::from(""));
            lines.push(Line::from("Jeu inutilisé ? Passe par son lanceur pour le désinstaller, après vérification."));
            lines.push(Line::from("Projet ? Vérifie le travail non enregistré et la sauvegarde avant toute décision."));
        }
        Some(Err(_)) => lines.push(Line::styled("Bilan indisponible. d : détails du problème et rapports manuels.", level_style("warn"))),
        _ => lines.push(Line::from("Aucun bilan disponible. b : retour pour préparer le bilan. d : détails.")),
    }
    let block = Block::default().borders(Borders::ALL).title(" Bilan · lecture seule ");
    let inner = block.inner(area);
    let lines = wrap_disk_lines(lines, inner.width);
    view.disk_scroll_max = lines.len().saturating_sub(usize::from(inner.height)).min(usize::from(u16::MAX)) as u16;
    view.disk_scroll = view.disk_scroll.min(view.disk_scroll_max);
    view.disk_page = inner.height.saturating_sub(1).max(1);
    frame.render_widget(Paragraph::new(lines).scroll((view.disk_scroll, 0)).block(block), area);
}

fn disks(frame: &mut Frame, area: Rect, view: &mut View) {
    match view.disk_screen {
        DiskScreen::Summary => disk_summary(frame, area, view),
        DiskScreen::Review => disk_review(frame, area, view),
        DiskScreen::Details => disk_details(frame, area, view),
    }
}

fn system(frame: &mut Frame, area: Rect, view: &View) {
    let facts = &view.facts;
    let mut lines = vec![Line::styled("Services", Style::default().add_modifier(Modifier::BOLD))];
    if facts.failed_units.is_empty() {
        lines.push(Line::styled("  aucun en échec", level_style("ok")));
    }
    lines.extend(facts.failed_units.iter().map(|unit| Line::styled(format!("  ✗ {unit}"), level_style("warn"))));
    lines.push(Line::from(""));
    lines.push(Line::styled("Erreurs du noyau (24 h)", Style::default().add_modifier(Modifier::BOLD)));
    if facts.kernel_errors.is_empty() {
        lines.push(Line::styled("  aucune de celles surveillées (réinitialisations du GPU, entrées/sorties disque, erreurs matérielles, mémoire épuisée, bridage)", level_style("ok")));
    }
    for (category, entries) in &facts.kernel_errors {
        lines.push(Line::styled(format!("  {category} : {} ligne(s)", entries.len()), level_style("warn")));
        lines.extend(entries.iter().rev().take(3).map(|line| Line::styled(format!("    {line}"), dim())));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled("Mémoire", Style::default().add_modifier(Modifier::BOLD)));
    lines.push(Line::from(format!("  {} libre   pression {}",
        facts.memory_available_percent.map(|free| format!("{free:.0} %")).unwrap_or("?".into()),
        facts.memory_pressure.map(|pressure| format!("{pressure:.1} %")).unwrap_or("?".into()))));
    lines.push(Line::from(""));
    lines.push(Line::styled("Sauvegarde", Style::default().add_modifier(Modifier::BOLD)));
    lines.push(Line::from(match facts.last_backup.or_else(|| Store::open().ok()?.state("last_backup").ok()?.and_then(|stamp| stamp.parse().ok())) {
        Some(at) => format!("  dernière sauvegarde réussie il y a {} h{}", (cockpit::util::now_secs() - at) / 3600,
                            facts.backup_result.as_ref().filter(|result| *result != "success").map(|result| format!(" · dernier passage : {result}")).unwrap_or_default()),
        None => "  aucune sauvegarde vue pour l'instant".into(),
    }));
    if let Some(error) = &view.error {
        lines.push(Line::from(""));
        lines.push(Line::styled(error.as_str(), level_style("crit")));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).title(" Système ")), area);
}

fn care_tab(frame: &mut Frame, area: Rect, view: &View) {
    let now = cockpit::util::now_secs();
    let [table_area, detail_area] = Layout::vertical([Constraint::Length(care::TASKS.len() as u16 + 3), Constraint::Min(4)]).areas(area);
    let rows: Vec<Row> = care::TASKS.iter().zip(&view.care_done).enumerate().map(|(index, (task, done))| {
        let (status, level) = match care::due(task, *done, now) {
            Due::Unknown => ("pas de date : d si fait aujourd'hui, puis b par semaine en arrière".to_string(), "warn"),
            Due::In(days) => (format!("dans {}", care::days(days)), "ok"),
            Due::Overdue(days) => (format!("{} de retard", care::days(days)), "crit"),
        };
        let last = done.map(|at| format!("il y a {}", care::days((now - at).div_euclid(DAY)))).unwrap_or_else(|| "–".into());
        let style = if index == view.care_selected { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
        Row::new(vec![Cell::from(task.title), Cell::from(format!("tous les {}", care::days(task.every_days))).style(dim()),
                      Cell::from(last), Cell::from(status).style(level_style(level))]).style(style)
    }).collect();
    let table = Table::new(rows, [Constraint::Length(46), Constraint::Length(16), Constraint::Length(16), Constraint::Min(20)])
        .header(Row::new(["Tâche", "Fréquence", "Dernière fois", "État"]).style(dim()))
        .block(Block::default().borders(Borders::ALL).title(" Entretien "));
    frame.render_widget(table, table_area);
    let task = &care::TASKS[view.care_selected];
    let detail = vec![Line::styled(task.title, Style::default().add_modifier(Modifier::BOLD)), Line::from(""),
                      Line::from(vec![Span::styled("Pourquoi    ", dim()), Span::raw(task.why)]),
                      Line::from(vec![Span::styled("Comment     ", dim()), Span::raw(task.how)])];
    frame.render_widget(Paragraph::new(detail).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL)), detail_area);
}

fn games_tab(frame: &mut Frame, area: Rect, view: &View) {
    let now = cockpit::util::now_secs();
    let rows: Vec<Row> = view.games.iter().map(|game| {
        let Some(end) = game.end else {
            return Row::new(vec![Cell::from(when(game.start)), Cell::from(game.name.clone()), Cell::from(duration(now - game.start)),
                                 Cell::from("en cours").style(level_style("ok"))]);
        };
        let peaks = game.stats.as_deref().map(Stats::from_json).unwrap_or_default();
        let at_limit = Cell::from(if peaks.minutes_at_limit == 0 { "–".to_string() } else { format!("{} min", peaks.minutes_at_limit) })
            .style(level_style(if peaks.minutes_at_limit == 0 { "ok" } else { "warn" }));
        Row::new(vec![Cell::from(when(game.start)), Cell::from(game.name.clone()), Cell::from(duration(end - game.start)),
                      Cell::from(degrees(peaks.cpu)), Cell::from(degrees(peaks.gpu)), Cell::from(degrees(peaks.hotspot)), Cell::from(degrees(peaks.vram)),
                      Cell::from(peaks.power.map(|watts| format!("{watts:.0} W")).unwrap_or_else(|| "–".into())), at_limit])
    }).collect();
    let empty = rows.is_empty();
    let table = Table::new(rows, [Constraint::Length(12), Constraint::Min(18), Constraint::Length(9), Constraint::Length(8), Constraint::Length(8),
                                  Constraint::Length(12), Constraint::Length(8), Constraint::Length(10), Constraint::Length(12)])
        .header(Row::new(["Début", "Jeu", "Durée", "CPU", "GPU", "Point chaud", "VRAM", "GPU max", "À la limite"]).style(dim()))
        .block(Block::default().borders(Borders::ALL).title(format!(" Sessions de jeu · {GAME_DAYS} jours · maximums atteints ")));
    if empty {
        let text = "Aucune session pour l'instant : une partie s'enregistre dès que GameMode démarre ou qu'une fenêtre de jeu a le focus.";
        frame.render_widget(Paragraph::new(Line::styled(text, dim())).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).title(" Sessions de jeu ")), area);
        return;
    }
    frame.render_widget(table, area);
}

/// The game running at `at`, if any.
fn game_at(games: &[Game], at: i64) -> Option<&str> {
    let now = cockpit::util::now_secs();
    games.iter().find(|game| game.start <= at && at <= game.end.unwrap_or(now)).map(|game| game.name.as_str())
}

/// Events and ended alerts together, latest first: (time, level, what, detail).
fn journal_entries(view: &View) -> Vec<(i64, &'static str, String, String)> {
    let mut entries: Vec<(i64, &'static str, String, String)> = view.events.iter().map(|event| {
        let level = if event.kind == "crash" || event.kind == "thermal" { "warn" } else { "crit" };
        let during = game_at(&view.games, event.at).map(|game| format!(" · pendant {game}")).unwrap_or_default();
        (event.at, level, journal::kind_name(&event.kind).to_string(), format!("{}{during}", event.text))
    }).collect();
    entries.extend(view.ended_alerts.iter().map(|alert| {
        (alert.since, if alert.level == "crit" { "crit" } else { "warn" }, format!("Alerte · {}", alert.title),
         format!("du {} au {} ({})", when(alert.since), when(alert.until), duration(alert.until - alert.since)))
    }));
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    entries
}

fn journal_tab(frame: &mut Frame, area: Rect, view: &View) {
    let entries = journal_entries(view);
    let mut lines: Vec<Line> = Vec::new();
    if entries.is_empty() {
        lines.push(Line::styled(format!("Rien à signaler ces {JOURNAL_DAYS} derniers jours."), level_style("ok")));
    }
    for (at, level, what, detail) in entries {
        lines.push(Line::from(vec![Span::styled(format!("{}  ", when(at)), dim()), Span::styled(what, level_style(level).add_modifier(Modifier::BOLD))]));
        lines.push(Line::styled(format!("             {detail}"), dim()));
    }
    let title = format!(" Journal · {JOURNAL_DAYS} jours · plantages, arrêts brutaux, mémoire, GPU, alertes terminées ");
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).title(title)), area);
}

/// d: done today; b: the recorded date a week earlier (for a task done a while ago).
fn record_care(view: &mut View, earlier: bool) {
    let task = &care::TASKS[view.care_selected];
    let date = match (earlier, view.care_done[view.care_selected]) {
        (true, Some(done)) => done - WEEK,
        _ => cockpit::util::now_secs(),
    };
    match Store::open().and_then(|store| care::mark_done(&store, task, date)) {
        Ok(()) => view.care_done[view.care_selected] = Some(date),
        Err(error) => view.error = Some(format!("impossible d'enregistrer « {} » : {error}", task.title)),
    }
}

fn draw(frame: &mut Frame, view: &mut View) {
    let [top, body, footer] = Layout::vertical([Constraint::Length(1), Constraint::Min(5), Constraint::Length(1)]).areas(frame.area());
    header(frame, top, view);
    match view.tab {
        0 => overview(frame, body, view),
        1 => cooling(frame, body, view),
        DISKS_TAB => disks(frame, body, view),
        3 => system(frame, body, view),
        CARE_TAB => care_tab(frame, body, view),
        5 => games_tab(frame, body, view),
        _ => journal_tab(frame, body, view),
    }
    let keys = if view.tab == CARE_TAB { "↑↓ choisir · d fait aujourd'hui · b une semaine plus tôt · 1-7 / Tab changer d'onglet · q quitter" }
        else if view.tab == DISKS_TAB && view.disk_screen == DiskScreen::Summary && disk_overview(view).primary.is_none() { "d détails · r état · q quitter" }
        else if view.tab == DISKS_TAB && view.disk_screen == DiskScreen::Summary && footer.width < 70 { "Entrée · d détails · r état · q quitter" }
        else if view.tab == DISKS_TAB && view.disk_screen == DiskScreen::Summary { "Entrée ouvrir · d détails · r état · q quitter" }
        else if view.tab == DISKS_TAB && view.disk_screen == DiskScreen::Details && footer.width >= 70 { "↑↓ défiler · a bilan · t Turbo · b retour · r état · q quitter" }
        else if view.tab == DISKS_TAB && footer.width < 70 { "↑↓/jk défiler · b retour · q quitter" }
        else if view.tab == DISKS_TAB { "↑↓/jk défiler · Pg↑↓ · b retour · d détails/retour · r · q quitter" }
        else { "1-7 / Tab changer d'onglet · r rafraîchir · q quitter" };
    frame.render_widget(Paragraph::new(Line::styled(keys, dim())), footer);
}

fn handle_key(view: &mut View, key: KeyCode) -> bool {
    handle_key_with(view, key, maintenance::request_check)
}

fn move_disk_selection(view: &mut View, step: isize) {
    let actions = disk_actions(view);
    if actions.is_empty() {
        view.disk_selected = None;
        view.disk_scroll = 0;
        return;
    }
    let selected = view.disk_selected.and_then(|selected| actions.iter().position(|action| *action == selected)).unwrap_or(0);
    let selected = selected.saturating_add_signed(step).min(actions.len() - 1);
    view.disk_selected = Some(actions[selected]);
    view.disk_scroll = 0;
}

fn handle_key_with(view: &mut View, key: KeyCode, mut request: impl FnMut(Check) -> Result<(), String>) -> bool {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => return false,
        KeyCode::Char(digit @ '1'..='7') => view.tab = digit as usize - '1' as usize,
        KeyCode::Up | KeyCode::Char('k') if view.tab == CARE_TAB => view.care_selected = view.care_selected.saturating_sub(1),
        KeyCode::Down | KeyCode::Char('j') if view.tab == CARE_TAB => view.care_selected = (view.care_selected + 1).min(care::TASKS.len() - 1),
        KeyCode::Char('d') if view.tab == CARE_TAB => record_care(view, false),
        KeyCode::Char('b') if view.tab == CARE_TAB => record_care(view, true),
        KeyCode::Char('d') if view.tab == DISKS_TAB => {
            view.disk_screen = if view.disk_screen == DiskScreen::Details { DiskScreen::Summary } else { DiskScreen::Details };
            view.disk_scroll = 0;
        }
        KeyCode::Char('b') | KeyCode::Backspace if view.tab == DISKS_TAB => {
            view.disk_screen = DiskScreen::Summary;
            view.disk_scroll = 0;
        }
        KeyCode::Enter | KeyCode::Char('a') | KeyCode::Char('t') if view.tab == DISKS_TAB
            && ((key == KeyCode::Enter && view.disk_screen == DiskScreen::Summary)
                || (key != KeyCode::Enter && view.disk_screen == DiskScreen::Details)) => {
            let selected = match key {
                KeyCode::Char('a') => Some(DiskAction::SpaceCheck),
                KeyCode::Char('t') => Some(DiskAction::TurboCheck),
                _ => disk_overview(view).primary,
            };
            let Some(selected) = selected else { return true; };
            match selected {
                DiskAction::ReadReview | DiskAction::Details => {
                    view.disk_screen = if selected == DiskAction::ReadReview { DiskScreen::Review } else { DiskScreen::Details };
                    view.disk_scroll = 0;
                }
                DiskAction::SpaceCheck | DiskAction::TurboCheck => {
                    let check = if selected == DiskAction::SpaceCheck { Check::SpaceReview } else { Check::TurboReport };
                    if !disk_overview(view).can_run_checks {
                        view.disk_feedback = Some(Err("Vérifie la sauvegarde et l’alerte SSD avant toute analyse supplémentaire.".into()));
                    } else if check_is_running(view, check) {
                        view.disk_feedback = Some(Ok("Vérification déjà en cours."));
                    } else {
                        match request(check) {
                            Ok(()) => {
                                view.disk_feedback = Some(Ok("Demande envoyée · le résultat arrivera après l’analyse."));
                                view.facts_at = None;
                            }
                            Err(error) => view.disk_feedback = Some(Err(error)),
                        }
                    }
                    if view.disk_screen == DiskScreen::Details { view.disk_scroll = 0; }
                }
            }
        }
        KeyCode::Up | KeyCode::Char('k') if view.tab == DISKS_TAB && view.disk_screen == DiskScreen::Summary => move_disk_selection(view, -1),
        KeyCode::Down | KeyCode::Char('j') if view.tab == DISKS_TAB && view.disk_screen == DiskScreen::Summary => move_disk_selection(view, 1),
        KeyCode::Home | KeyCode::PageUp if view.tab == DISKS_TAB && view.disk_screen == DiskScreen::Summary => move_disk_selection(view, isize::MIN),
        KeyCode::End | KeyCode::PageDown if view.tab == DISKS_TAB && view.disk_screen == DiskScreen::Summary => move_disk_selection(view, isize::MAX),
        KeyCode::Up | KeyCode::Char('k') if view.tab == DISKS_TAB => view.disk_scroll = view.disk_scroll.saturating_sub(1),
        KeyCode::Down | KeyCode::Char('j') if view.tab == DISKS_TAB => view.disk_scroll = view.disk_scroll.saturating_add(1).min(view.disk_scroll_max),
        KeyCode::PageUp if view.tab == DISKS_TAB => view.disk_scroll = view.disk_scroll.saturating_sub(view.disk_page),
        KeyCode::PageDown if view.tab == DISKS_TAB => view.disk_scroll = view.disk_scroll.saturating_add(view.disk_page).min(view.disk_scroll_max),
        KeyCode::Home if view.tab == DISKS_TAB => view.disk_scroll = 0,
        KeyCode::End if view.tab == DISKS_TAB => view.disk_scroll = view.disk_scroll_max,
        KeyCode::Tab | KeyCode::Right => view.tab = (view.tab + 1) % TABS.len(),
        KeyCode::BackTab | KeyCode::Left => view.tab = (view.tab + TABS.len() - 1) % TABS.len(),
        KeyCode::Char('r') => view.facts_at = None,
        _ => {}
    }
    true
}

fn event_loop(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut view = View::new();
    view.refresh();
    let mut refreshed = Instant::now();
    loop {
        terminal.draw(|frame| draw(frame, &mut view))?;
        let wait = REFRESH.saturating_sub(refreshed.elapsed());
        if event::poll(wait)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if !handle_key(&mut view, key.code) { return Ok(()); }
                if view.facts_at.is_none() {
                    view.refresh();
                    refreshed = Instant::now();
                }
            }
        }
        if refreshed.elapsed() >= REFRESH {
            view.refresh();
            refreshed = Instant::now();
        }
    }
}

pub fn run() -> Result<(), String> {
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal);
    ratatui::restore();
    result.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn maintenance_view() -> View {
        use crate::maintenance::{Maintenance, Review, ReviewEntry, RunState, Task};
        let mut view = View::new();
        view.tab = 2;
        view.disk_screen = DiskScreen::Details;
        view.maintenance = Some(Maintenance {
            tasks: vec![Task { timer: "paccache.timer".into(), label: "Paquets", policy: "2 versions + 30 jours",
                enabled: Some(true), last_run: Some(1000), next_run: Some(2000), state: RunState::Succeeded, detail: None }],
            journal_retention: Ok(Some("30day".into())),
            review: Ok(Some(Review { completed_at: 1000, complete: false, measured_games: 4, measured_checkouts: 2,
                omitted_games: 0, omitted_checkouts: 0, errors: 1, issues: Vec::new(), skipped_symlinks: None, unavailable: 2,
                games: vec![ReviewEntry { path: "/games/large".into(), allocated_bytes: 1073741824, complete: false }],
                checkouts: vec![ReviewEntry { path: "/projects/personal".into(), allocated_bytes: 2147483648, complete: true }] })),
        });
        view
    }

    fn compact_disk_view() -> View {
        let mut view = maintenance_view();
        view.disk_screen = DiskScreen::Summary;
        view.facts.disk_usage = vec![("/".into(), 73.0)];
        view.facts.disks = vec![crate::facts::Smart { device: "nvme0n1".into(), available_spare: 100,
            spare_threshold: 10, read_at: cockpit::util::now_secs(), ..Default::default() }];
        let report = view.maintenance.as_mut().unwrap().review.as_mut().unwrap().as_mut().unwrap();
        report.errors = 0;
        report.skipped_symlinks = Some(92733);
        report.complete = true;
        report.unavailable = 0;
        report.games[0].complete = true;
        view
    }

    #[test]
    fn the_default_disk_view_has_one_recommendation_and_no_wall_of_details() {
        let mut view = compact_disk_view();
        let screen = render(&mut view, 100, 26);
        for expected in ["Aucune action urgente", "Si tu veux libérer de la place", "Lire le bilan"] {
            assert!(screen.contains(expected), "missing {expected}: {screen}");
        }
        assert!(!screen.contains("politique approuvée"));
        assert!(!screen.contains("avertissement critique"));
        assert!(!screen.contains("92733 erreur"));
        assert!(!screen.contains("cache Turbo"));
        assert!(!screen.contains("Actualiser le bilan"));
        assert!(!screen.contains("Prochain nettoyage"));
        assert!(!screen.contains("Prochain pas"));
        assert_eq!(screen.matches("> ").count(), 1);
        assert_eq!(View::new().disk_screen, DiskScreen::Summary);
    }

    #[test]
    fn expected_skips_are_not_errors_but_real_failures_remain_actionable() {
        let mut view = compact_disk_view();
        let screen = render(&mut view, 100, 26);
        assert!(screen.contains("Aucune action urgente"));
        view.maintenance.as_mut().unwrap().review.as_mut().unwrap().as_mut().unwrap().errors = 1;
        let screen = render(&mut view, 100, 26);
        assert!(screen.contains("Bilan à vérifier"));
        assert!(!screen.contains("Aucune action urgente"));
        view.maintenance.as_mut().unwrap().review.as_mut().unwrap().as_mut().unwrap().skipped_symlinks = None;
        let screen = render(&mut view, 100, 26);
        assert!(screen.contains("Bilan à actualiser"));
    }

    #[test]
    fn space_and_hardware_warnings_take_priority_over_optional_checks() {
        let mut view = compact_disk_view();
        for (used, expected) in [(85.0, "Espace à surveiller"), (95.0, "Espace presque plein")] {
            view.facts.disk_usage[0].1 = used;
            assert!(render(&mut view, 100, 26).contains(expected));
        }
        view.facts.disks[0].critical_warning = 1;
        assert!(render(&mut view, 100, 26).contains("Vérifier le SSD en priorité"));
        assert_eq!(disk_actions(&view), [DiskAction::Details]);
    }

    #[test]
    fn the_main_view_never_claims_health_without_space_or_ssd_information() {
        let mut view = compact_disk_view();
        view.facts.disk_usage.clear();
        assert!(!render(&mut view, 100, 26).contains("Aucune action urgente"));
        view.facts.disk_usage.push(("/".into(), 73.0));
        view.facts.disks.clear();
        assert!(!render(&mut view, 100, 26).contains("Aucune action urgente"));
    }

    #[test]
    fn reading_the_review_and_toggling_details_never_start_a_check() {
        let mut view = compact_disk_view();
        assert!(handle_key_with(&mut view, KeyCode::Enter, |_| panic!("reading the report must not start a service")));
        assert_eq!(view.disk_screen, DiskScreen::Review);
        let screen = render(&mut view, 100, 26);
        assert!(screen.contains("Jeux à vérifier"));
        assert!(screen.contains("large"));
        handle_key_with(&mut view, KeyCode::Char('d'), |_| panic!("details must not start a service"));
        assert_eq!(view.disk_screen, DiskScreen::Details);
        assert!(render(&mut view, 120, 40).contains("politique approuvée"));
        handle_key_with(&mut view, KeyCode::Char('d'), |_| panic!("returning must not start a service"));
        assert_eq!(view.disk_screen, DiskScreen::Summary);
    }

    #[test]
    fn entering_a_check_requests_only_that_check_and_never_claims_it_completed() {
        use crate::maintenance::Check;
        let mut view = compact_disk_view();
        view.maintenance.as_mut().unwrap().review = Ok(None);
        let mut requested = Vec::new();
        handle_key_with(&mut view, KeyCode::Enter, |check| { requested.push(check); Ok(()) });
        assert_eq!(requested, [Check::SpaceReview]);
        assert!(view.facts_at.is_none());
        assert!(render(&mut view, 100, 26).contains("Demande envoyée"));
        handle_key_with(&mut view, KeyCode::Char('d'), |_| panic!("details must not launch"));
        handle_key_with(&mut view, KeyCode::Char('t'), |check| { requested.push(check); Ok(()) });
        assert_eq!(requested, [Check::SpaceReview, Check::TurboReport]);
    }

    #[test]
    fn a_running_check_is_not_requested_again_and_request_errors_are_visible() {
        let mut view = compact_disk_view();
        let snapshot = view.maintenance.as_mut().unwrap();
        let mut task = snapshot.tasks[0].clone();
        task.timer = "disk-space-review.timer".into();
        task.state = RunState::Running;
        snapshot.tasks.push(task);
        view.disk_screen = DiskScreen::Details;
        handle_key_with(&mut view, KeyCode::Char('a'), |_| panic!("already running"));
        assert!(render(&mut view, 100, 26).contains("déjà en cours"));
        view.maintenance.as_mut().unwrap().tasks.last_mut().unwrap().state = RunState::Failed;
        handle_key_with(&mut view, KeyCode::Char('a'), |_| Err("permission refusée".into()));
        assert!(render(&mut view, 100, 26).contains("permission refusée"));
    }

    #[test]
    fn summary_navigation_cannot_select_hidden_actions_on_small_terminals() {
        let mut view = compact_disk_view();
        let first = render(&mut view, 40, 12);
        assert!(first.contains("Aucune action urgente"), "{first}");
        handle_key_with(&mut view, KeyCode::End, |_| panic!("navigation must not launch"));
        let screen = render(&mut view, 40, 12);
        assert!(screen.contains("Lire le bilan"), "{screen}");
        assert!(!screen.contains("cache Turbo"), "{screen}");
        assert!(screen.contains("q quitter"));
        assert!(screen.contains("r état"));
        handle_key_with(&mut view, KeyCode::Home, |_| panic!("navigation must not launch"));
        handle_key_with(&mut view, KeyCode::Enter, |_| panic!("only the visible recommendation may run"));
        assert_eq!(view.disk_screen, DiskScreen::Review);
    }

    #[test]
    fn request_failure_feedback_stays_visible_on_a_small_terminal() {
        let mut view = compact_disk_view();
        view.disk_feedback = Some(Err("permission refusée".into()));
        let screen = render(&mut view, 40, 12);
        assert!(screen.contains("Vérification non lancée"), "{screen}");
        assert!(screen.contains("Demande refusée"), "{screen}");
        assert!(!screen.contains("permission refusée"), "{screen}");
        assert!(screen.contains("q quitter"));
        view.disk_screen = DiskScreen::Details;
        assert!(render(&mut view, 40, 12).replace('│', "").contains("permission refusée"));
    }

    #[test]
    fn enter_never_runs_an_action_which_is_hidden_behind_details() {
        let mut view = compact_disk_view();
        view.disk_selected = Some(DiskAction::SpaceCheck);
        handle_key_with(&mut view, KeyCode::Char('d'), |_| panic!("toggle must not launch"));
        handle_key_with(&mut view, KeyCode::Enter, |_| panic!("hidden action must not launch"));
        assert_eq!(view.disk_screen, DiskScreen::Details);
    }

    #[test]
    fn numbered_tab_shortcuts_do_not_run_disk_actions() {
        let mut view = compact_disk_view();
        handle_key_with(&mut view, KeyCode::Char('1'), |_| panic!("tab shortcuts must not launch checks"));
        assert_eq!(view.tab, 0);
    }

    #[test]
    fn access_errors_stay_explicit_but_diagnostics_are_only_in_details() {
        let mut view = compact_disk_view();
        let report = view.maintenance.as_mut().unwrap().review.as_mut().unwrap().as_mut().unwrap();
        report.errors = 1;
        report.issues = vec![crate::maintenance::ReviewIssue { path: "/project/storage/testing".into(), error: "[Errno 13] Permission denied".into() }];
        let screen = render(&mut view, 100, 26);
        assert!(screen.contains("Bilan à vérifier"), "{screen}");
        assert!(!screen.contains("1 points"), "{screen}");
        handle_key_with(&mut view, KeyCode::Enter, |_| panic!("reading issues must not launch"));
        let screen = render(&mut view, 120, 40);
        assert!(screen.contains("bilan est incomplet"), "{screen}");
        assert!(screen.contains("Jeux à vérifier"), "{screen}");
        assert!(!screen.contains("/project/storage/testing"), "{screen}");
        assert!(!screen.contains("4 jeux"), "{screen}");
        assert!(!screen.contains("92733"), "{screen}");
        handle_key_with(&mut view, KeyCode::Char('d'), |_| panic!("details must not launch"));
        let screen = render(&mut view, 120, 40);
        assert!(screen.contains("/project/storage/testing"), "{screen}");
    }

    #[test]
    fn readable_parallel_checkout_names_keep_the_project_context() {
        let mut view = compact_disk_view();
        view.maintenance.as_mut().unwrap().review.as_mut().unwrap().as_mut().unwrap().checkouts[0].path =
            "/home/example/IdeaProjects/.parallel/dotfiles/parallel-1924".into();
        view.disk_screen = DiskScreen::Review;
        let screen = render(&mut view, 120, 40);
        assert!(screen.contains("dotfiles / parallel-1924"), "{screen}");
    }

    #[test]
    fn stale_secondary_selection_never_overrides_the_summary_recommendation() {
        for selection in [DiskAction::SpaceCheck, DiskAction::TurboCheck] {
            let mut view = compact_disk_view();
            view.disk_selected = Some(selection);
            handle_key_with(&mut view, KeyCode::Enter, |_| panic!("stale selection must not launch"));
            assert_eq!(view.disk_screen, DiskScreen::Review);
            view.disk_screen = DiskScreen::Summary;
            for key in [KeyCode::Down, KeyCode::End, KeyCode::Up, KeyCode::Home, KeyCode::Char('a'), KeyCode::Char('t'), KeyCode::Char('r')] {
                handle_key_with(&mut view, key, |_| panic!("summary navigation must not launch"));
            }
            handle_key_with(&mut view, KeyCode::Enter, |_| panic!("stale selection must not launch"));
            assert_eq!(view.disk_screen, DiskScreen::Review);
        }
    }

    #[test]
    fn details_manual_controls_dispatch_only_the_two_reports() {
        let mut view = compact_disk_view();
        view.disk_screen = DiskScreen::Details;
        let screen = render(&mut view, 100, 26);
        for label in ["a : bilan d’espace", "t : rapport Turbo", "lecture seule"] {
            assert!(screen.contains(label), "missing {label}: {screen}");
        }
        let mut requested = Vec::new();
        for key in ['a', 't'] {
            handle_key_with(&mut view, KeyCode::Char(key), |check| { requested.push(check); Ok(()) });
        }
        assert_eq!(requested, [Check::SpaceReview, Check::TurboReport]);
        assert!(render(&mut view, 100, 26).contains("Demande envoyée"));
    }

    #[test]
    fn details_manual_controls_cannot_bypass_critical_or_running_guards() {
        for (key, timer) in [('a', "disk-space-review.timer"), ('t', "cache-maintenance-report.timer")] {
            let mut view = compact_disk_view();
            view.disk_screen = DiskScreen::Details;
            view.facts.disks[0].critical_warning = 1;
            handle_key_with(&mut view, KeyCode::Char(key), |_| panic!("critical SSD must block reports"));
            assert!(render(&mut view, 120, 40).contains("non lancée"));
            view.facts.disks[0].critical_warning = 0;
            let snapshot = view.maintenance.as_mut().unwrap();
            let mut task = snapshot.tasks[0].clone();
            task.timer = timer.into();
            task.state = RunState::Running;
            snapshot.tasks.push(task);
            handle_key_with(&mut view, KeyCode::Char(key), |_| panic!("already running"));
            assert!(render(&mut view, 120, 40).contains("déjà en cours"));
        }
    }

    #[test]
    fn summary_header_keeps_critical_machine_alerts_without_technical_totals() {
        let mut view = compact_disk_view();
        view.alerts.push(Alert { level: "crit".into(), title: "CPU trop chaud".into(), detail: "test".into(), since: 1000, key: "cpu".into(), notified: false });
        for collected_at in [None, Some(cockpit::util::now_secs())] {
            view.collected_at = collected_at;
            let screen = render(&mut view, 100, 26);
            assert!(screen.contains("PC : alerte importante"), "{screen}");
            assert!(!screen.contains("problème(s)"));
            let backend = ratatui::backend::TestBackend::new(40, 12);
            let mut terminal = ratatui::Terminal::new(backend).unwrap();
            terminal.draw(|frame| draw(frame, &mut view)).unwrap();
            let top = &terminal.backend().buffer().content()[..40];
            assert!(top.iter().any(|cell| cell.fg == Color::Red));
        }
        view.disk_screen = DiskScreen::Details;
        assert!(render(&mut view, 100, 26).contains("1 problème(s)"));
    }

    #[test]
    fn guided_review_does_not_leak_invalid_report_errors() {
        let mut view = compact_disk_view();
        view.maintenance.as_mut().unwrap().review = Err("invalid /raw/report.json".into());
        view.disk_screen = DiskScreen::Review;
        let screen = render(&mut view, 100, 26);
        assert!(screen.contains("Bilan indisponible"));
        assert!(!screen.contains("/raw/report.json"));
        assert!(screen.contains("d : détails"));
    }

    #[test]
    fn partial_and_unknown_reports_are_explained_without_counts_or_paths() {
        for legacy in [false, true] {
            let mut view = compact_disk_view();
            let report = view.maintenance.as_mut().unwrap().review.as_mut().unwrap().as_mut().unwrap();
            report.complete = false;
            report.unavailable = 7;
            report.skipped_symlinks = if legacy { None } else { Some(92733) };
            for size in [(100, 26), (40, 12)] {
                let screen = render(&mut view, size.0, size.1);
                assert!(!screen.contains("Aucune action urgente"), "{screen}");
                assert!(!screen.contains("92733"));
            }
            view.disk_screen = DiskScreen::Review;
            let screen = render(&mut view, 100, 26);
            assert!(screen.contains("incomplet"), "{screen}");
            assert!(!screen.contains("7 chemin"));
            assert!(!screen.contains("4 jeux"));
        }
    }

    #[test]
    fn request_feedback_never_replaces_real_space_or_ssd_warnings() {
        for (used, expected) in [(85.0, "Espace à surveiller"), (95.0, "Espace presque plein")] {
            for feedback in [Ok("Demande envoyée · le résultat arrivera après l’analyse."), Err("raw /private/error".into())] {
                let mut view = compact_disk_view();
                view.facts.disk_usage[0].1 = used;
                view.disk_feedback = Some(feedback);
                for (width, height) in [(100, 26), (40, 12)] {
                    let screen = render(&mut view, width, height);
                    assert!(screen.contains(expected), "{screen}");
                    assert_eq!(screen.matches("> ").count(), 1, "{screen}");
                    assert!(!screen.contains("/private/error"));
                }
                view.facts.disks[0].critical_warning = 1;
                let screen = render(&mut view, 40, 12);
                assert!(screen.contains("Vérifier le SSD en priorité"), "{screen}");
                assert!(screen.contains("sauvegarde"), "{screen}");
            }
        }
    }

    #[test]
    fn all_guided_states_keep_at_most_one_next_step_and_refresh_never_starts_a_scan() {
        for state in ["healthy", "space-warning", "space-critical", "ssd-critical", "ssd-stale", "ssd-missing", "report-missing", "report-invalid", "report-running"] {
            let mut view = compact_disk_view();
            match state {
                "space-warning" => view.facts.disk_usage[0] = ("/private/mount".into(), 85.0),
                "space-critical" => view.facts.disk_usage[0] = ("/private/mount".into(), 95.0),
                "ssd-critical" => view.facts.disks[0].critical_warning = 1,
                "ssd-stale" => view.facts.disks[0].read_at -= 3 * 3600,
                "ssd-missing" => view.facts.disks.clear(),
                "report-missing" => view.maintenance.as_mut().unwrap().review = Ok(None),
                "report-invalid" => view.maintenance.as_mut().unwrap().review = Err("invalid /raw/report".into()),
                "report-running" => {
                    let snapshot = view.maintenance.as_mut().unwrap();
                    snapshot.review = Ok(None);
                    snapshot.tasks[0].timer = "disk-space-review.timer".into();
                    snapshot.tasks[0].state = RunState::Running;
                }
                _ => {}
            }
            for (width, height) in [(100, 26), (40, 12)] {
                let screen = render(&mut view, width, height);
                assert_eq!(screen.matches("> ").count(), if state == "report-running" { 0 } else { 1 }, "{state}: {screen}");
                for hidden in ["/private/mount", "/raw/report", "92733", "prochain :", "problème(s)"] {
                    assert!(!screen.contains(hidden), "{state}: {screen}");
                }
                assert!(screen.contains("q quitter"), "{state}: {screen}");
            }
            handle_key_with(&mut view, KeyCode::Char('r'), |_| panic!("refresh never starts a scan"));
            for digit in '1'..='7' {
                handle_key_with(&mut view, KeyCode::Char(digit), |_| panic!("tab never starts a scan"));
                assert_eq!(view.tab, digit as usize - '1' as usize);
            }
        }
    }

    #[test]
    fn healthy_and_running_previous_reports_are_optional_to_read() {
        for running in [false, true] {
            let mut view = compact_disk_view();
            if running {
                let snapshot = view.maintenance.as_mut().unwrap();
                let mut task = snapshot.tasks[0].clone();
                task.timer = "disk-space-review.timer".into();
                task.state = RunState::Running;
                snapshot.tasks.push(task);
            }
            for (width, height) in [(100, 26), (40, 12)] {
                let screen = render(&mut view, width, height);
                assert!(screen.contains("Si tu veux libérer de la place"), "{screen}");
                assert!(!screen.contains("Prochain pas"), "{screen}");
                assert!(screen.contains("Lire le bilan"), "{screen}");
                assert_eq!(screen.matches("> ").count(), 1, "{screen}");
            }
            handle_key_with(&mut view, KeyCode::Enter, |_| panic!("optional reading never starts a scan"));
            assert_eq!(view.disk_screen, DiskScreen::Review);
        }
    }

    #[test]
    fn a_running_first_review_reassures_without_an_action_or_enter_instruction() {
        let mut view = compact_disk_view();
        let snapshot = view.maintenance.as_mut().unwrap();
        snapshot.review = Ok(None);
        let mut task = snapshot.tasks[0].clone();
        task.timer = "disk-space-review.timer".into();
        task.state = RunState::Running;
        snapshot.tasks.push(task);
        view.disk_selected = Some(DiskAction::TurboCheck);
        for feedback in [None, Some(Ok("Vérification déjà en cours.")), Some(Err("previous refusal".into()))] {
            view.disk_feedback = feedback;
            for (width, height) in [(100, 26), (40, 12)] {
                let screen = render(&mut view, width, height);
                let text = screen.replace('│', "");
                for expected in ["Rien à faire pour le moment", "fermer Vitals", "notification", "d détails", "r état", "q quitter"] {
                    assert!(text.contains(expected), "missing {expected}: {screen}");
                }
                for hidden in ["> ", "Préparer le bilan", "Entrée", "Prochain pas", "previous refusal", "déjà en cours"] {
                    assert!(!screen.contains(hidden), "unexpected {hidden}: {screen}");
                }
            }
            let previous_feedback = view.disk_feedback.clone();
            for key in [KeyCode::Enter, KeyCode::Home, KeyCode::End, KeyCode::Down, KeyCode::Up, KeyCode::PageDown, KeyCode::PageUp, KeyCode::Enter, KeyCode::Char('a'), KeyCode::Char('t'), KeyCode::Char('r')] {
                handle_key_with(&mut view, key, |_| panic!("waiting must not launch a hidden check"));
                assert_eq!(view.disk_screen, DiskScreen::Summary);
                assert_eq!(view.disk_feedback, previous_feedback);
            }
        }
        for (used, warning) in [(85.0, "Espace à surveiller"), (95.0, "Espace presque plein")] {
            view.facts.disk_usage[0].1 = used;
            let screen = render(&mut view, 40, 12);
            assert!(screen.contains(warning), "{screen}");
            assert_eq!(screen.matches("> ").count(), 1, "{screen}");
            assert!(screen.contains("Entrée"), "{screen}");
        }
        view.facts.disks[0].critical_warning = 1;
        let screen = render(&mut view, 40, 12);
        assert!(screen.contains("Vérifier le SSD en priorité"), "{screen}");
        assert_eq!(screen.matches("> ").count(), 1, "{screen}");
        handle_key_with(&mut view, KeyCode::Enter, |_| panic!("SSD warning opens details only"));
        assert_eq!(view.disk_screen, DiskScreen::Details);
    }

    fn space_warning_view(used: f64, report_state: &str) -> View {
        let mut view = compact_disk_view();
        view.facts.disk_usage[0].1 = used;
        let snapshot = view.maintenance.as_mut().unwrap();
        match report_state {
            "usable" => {}
            "missing" => snapshot.review = Ok(None),
            "invalid" => snapshot.review = Err("invalid /raw/report".into()),
            "running" => {
                snapshot.review = Ok(None);
                let mut task = snapshot.tasks[0].clone();
                task.timer = "disk-space-review.timer".into();
                task.state = RunState::Running;
                snapshot.tasks.push(task);
            }
            _ => panic!("unknown report fixture: {report_state}"),
        }
        view
    }

    #[test]
    fn space_warnings_offer_a_useful_action_for_the_available_report_state() {
        for (used, warning) in [(85.0, "Espace à surveiller"), (95.0, "Espace presque plein")] {
            for (state, action) in [("usable", "Lire le bilan"), ("missing", "Préparer le bilan"), ("invalid", "Préparer le bilan"), ("running", "Comprendre l’avertissement")] {
                let mut view = space_warning_view(used, state);
                for (width, height) in [(100, 26), (40, 12)] {
                    let screen = render(&mut view, width, height);
                    assert!(screen.contains(warning), "{used}/{state}: {screen}");
                    assert!(screen.contains(action), "{used}/{state}: {screen}");
                    assert_eq!(screen.matches("> ").count(), 1, "{used}/{state}: {screen}");
                    assert!(!screen.contains("/raw/report"), "{screen}");
                    if state == "running" {
                        assert!(screen.contains("Bilan en cours"), "{screen}");
                        assert!(screen.contains("notification"), "{screen}");
                        assert!(!screen.contains("Préparer le bilan"), "{screen}");
                    }
                    let backend = ratatui::backend::TestBackend::new(width, height);
                    let mut terminal = ratatui::Terminal::new(backend).unwrap();
                    terminal.draw(|frame| draw(frame, &mut view)).unwrap();
                    let headline_start = usize::from(width) * 2 + 1;
                    assert_eq!(terminal.backend().buffer().content()[headline_start].fg, if used == 95.0 { Color::Red } else { Color::Yellow });
                }
            }
        }
    }

    #[test]
    fn space_warning_enter_reads_prepares_or_waits_without_a_dead_end() {
        for used in [85.0, 95.0] {
            for state in ["usable", "missing", "invalid", "running"] {
                let mut view = space_warning_view(used, state);
                let mut requested = Vec::new();
                handle_key_with(&mut view, KeyCode::Enter, |check| { requested.push(check); Ok(()) });
                match state {
                    "usable" => {
                        assert!(requested.is_empty());
                        assert_eq!(view.disk_screen, DiskScreen::Review);
                        assert!(render(&mut view, 100, 26).contains("large"));
                    }
                    "missing" | "invalid" => {
                        assert_eq!(requested, [Check::SpaceReview], "{used}/{state}");
                        assert_eq!(view.disk_screen, DiskScreen::Summary);
                        assert!(!render(&mut view, 100, 26).contains("revue complète"));
                    }
                    "running" => {
                        assert!(requested.is_empty());
                        assert_eq!(view.disk_screen, DiskScreen::Details);
                        assert!(render(&mut view, 120, 40).contains("première revue en cours"));
                    }
                    _ => unreachable!(),
                }
                for mount in ["/boot", "/boot/efi"] {
                    let mut view = space_warning_view(used, state);
                    view.facts.disk_usage[0].0 = mount.into();
                    handle_key_with(&mut view, KeyCode::Enter, |_| panic!("boot warnings only open Details"));
                    assert_eq!(view.disk_screen, DiskScreen::Details);
                }
                let mut view = space_warning_view(used, state);
                view.facts.disks[0].critical_warning = 1;
                handle_key_with(&mut view, KeyCode::Enter, |_| panic!("SSD warning only opens Details"));
                assert_eq!(view.disk_screen, DiskScreen::Details);
                handle_key_with(&mut view, KeyCode::Char('a'), |_| panic!("SSD critical blocks preparation"));
            }
        }
    }

    #[test]
    fn overfull_root_usage_keeps_its_critical_warning_and_useful_action() {
        for used in [101.0, 104.0] {
            for (state, action) in [("usable", "Lire le bilan"), ("missing", "Préparer le bilan")] {
                let mut view = space_warning_view(used, state);
                view.facts.disk_usage.push(("/boot".into(), 12.0));
                for (width, height) in [(100, 26), (40, 12)] {
                    let screen = render(&mut view, width, height);
                    assert!(screen.contains("Espace presque plein"), "{used}/{state}: {screen}");
                    assert!(screen.contains(action), "{screen}");
                    assert!(!screen.contains("Aucune action urgente"), "{screen}");
                    let backend = ratatui::backend::TestBackend::new(width, height);
                    let mut terminal = ratatui::Terminal::new(backend).unwrap();
                    terminal.draw(|frame| draw(frame, &mut view)).unwrap();
                    assert_eq!(terminal.backend().buffer().content()[usize::from(width) * 2 + 1].fg, Color::Red);
                }
                let mut requested = Vec::new();
                handle_key_with(&mut view, KeyCode::Enter, |check| { requested.push(check); Ok(()) });
                if state == "usable" {
                    assert!(requested.is_empty());
                    assert_eq!(view.disk_screen, DiskScreen::Review);
                } else {
                    assert_eq!(requested, [Check::SpaceReview]);
                    assert_eq!(view.disk_screen, DiskScreen::Summary);
                }
            }
        }
    }

    #[test]
    fn nonfinite_and_negative_usage_do_not_become_space_measurements() {
        for used in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
            let mut view = compact_disk_view();
            view.facts.disk_usage = vec![("/".into(), used)];
            let screen = render(&mut view, 100, 26);
            assert!(screen.contains("Mesure d’espace indisponible"), "{screen}");
            assert!(!screen.contains("Aucune action urgente"), "{screen}");
            view.facts.disk_usage.push(("/boot".into(), 12.0));
            assert!(render(&mut view, 100, 26).contains("Aucune action urgente"));
        }
    }

    #[test]
    fn review_header_uses_plain_machine_state_without_counts_or_collection_timing() {
        for (severity, collected, history_failed, expected, color) in [
            (None, true, false, "PC : aucune alerte connue", Color::Green),
            (Some("warn"), true, false, "PC : point à vérifier", Color::Yellow),
            (Some("crit"), true, false, "PC : alerte importante", Color::Red),
            (Some("crit"), false, true, "PC : alerte importante", Color::Red),
            (None, false, false, "PC : état incomplet", Color::Yellow),
            (None, true, true, "PC : état incomplet", Color::Yellow),
        ] {
            let mut view = compact_disk_view();
            view.disk_screen = DiskScreen::Review;
            view.collected_at = collected.then(|| cockpit::util::now_secs() - 123);
            view.error = history_failed.then(|| "history failure /raw/database".into());
            if let Some(level) = severity {
                let alert = Alert { level: level.into(), title: "Machine warning".into(), detail: "raw diagnostic".into(), since: 1000, key: "cpu".into(), notified: false };
                view.alerts = vec![alert.clone(), alert];
            }
            for (width, height) in [(100, 26), (40, 12)] {
                let backend = ratatui::backend::TestBackend::new(width, height);
                let mut terminal = ratatui::Terminal::new(backend).unwrap();
                terminal.draw(|frame| draw(frame, &mut view)).unwrap();
                let cells = &terminal.backend().buffer().content()[..usize::from(width)];
                let header: String = cells.iter().map(|cell| cell.symbol()).collect();
                assert!(header.contains(expected), "missing {expected}: {header}");
                assert!(cells.iter().any(|cell| cell.fg == color));
                for diagnostic in ["problème(s)", "collecté", "123 s", "/raw/database"] {
                    assert!(!header.contains(diagnostic), "unexpected {diagnostic}: {header}");
                }
            }
        }
    }

    #[test]
    fn technical_header_remains_unchanged_in_details_and_other_tabs() {
        let mut view = compact_disk_view();
        view.collected_at = Some(cockpit::util::now_secs());
        for (tab, screen) in [(DISKS_TAB, DiskScreen::Details), (0, DiskScreen::Review), (CARE_TAB, DiskScreen::Review)] {
            view.tab = tab;
            view.disk_screen = screen;
            let screen = render(&mut view, 100, 26);
            assert!(screen.chars().take(100).collect::<String>().contains("collecté il y a"), "{screen}");
        }
    }

    fn render(view: &mut View, width: u16, height: u16) -> String {
        let backend = ratatui::backend::TestBackend::new(width, height);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, view)).unwrap();
        terminal.backend().buffer().content().iter().map(|cell| cell.symbol()).collect()
    }

    #[test]
    fn disks_show_native_maintenance_and_the_partial_monthly_review() {
        let mut view = maintenance_view();
        let screen = render(&mut view, 120, 40);
        for expected in ["Maintenance", "Paquets", "activé", "réussi", "dernier", "prochain",
                         "30day", "Revue mensuelle", "partielle", "4 jeux", "2 projets",
                         "/games/large", "partiel", "/projects/personal", "accord"] {
            assert!(screen.contains(expected), "missing {expected}: {screen}");
        }
    }

    #[test]
    fn disks_distinguish_running_unknown_and_failed_tasks() {
        use crate::maintenance::RunState;
        let mut view = maintenance_view();
        for (state, label) in [(RunState::Running, "en cours"), (RunState::Failed, "échec"), (RunState::Unknown, "inconnu")] {
            view.maintenance.as_mut().unwrap().tasks[0].state = state;
            view.maintenance.as_mut().unwrap().tasks[0].enabled = None;
            let screen = render(&mut view, 120, 40);
            assert!(screen.contains(label));
            assert!(screen.contains("activation inconnue"));
        }
    }

    #[test]
    fn a_failed_task_uses_the_existing_critical_style_in_the_rendered_buffer() {
        let mut view = maintenance_view();
        view.maintenance.as_mut().unwrap().tasks[0].state = RunState::Failed;
        let backend = ratatui::backend::TestBackend::new(120, 40);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, &mut view)).unwrap();
        assert!(terminal.backend().buffer().content().iter().any(|cell| cell.symbol() == "é" && cell.fg == Color::Red));
    }

    #[test]
    fn pending_review_is_not_displayed_as_an_empty_success() {
        let mut view = maintenance_view();
        let snapshot = view.maintenance.as_mut().unwrap();
        snapshot.review = Ok(None);
        snapshot.tasks[0].timer = "disk-space-review.timer".into();
        snapshot.tasks[0].state = crate::maintenance::RunState::Running;
        assert!(render(&mut view, 100, 40).contains("première revue en cours"));
        view.maintenance.as_mut().unwrap().tasks[0].state = crate::maintenance::RunState::Unknown;
        assert!(render(&mut view, 100, 40).contains("aucune revue disponible"));
    }

    #[test]
    fn a_short_terminal_can_scroll_to_the_last_review_entry_and_keep_the_footer() {
        let mut view = maintenance_view();
        render(&mut view, 80, 12);
        assert!(handle_key(&mut view, KeyCode::End));
        let screen = render(&mut view, 80, 12);
        assert!(screen.contains("/projects/personal"), "{screen}");
        assert!(screen.contains("défiler"));
        assert!(screen.contains("q quitter"));
        let end = view.disk_scroll;
        assert!(end > 0);
        handle_key(&mut view, KeyCode::Down);
        assert_eq!(view.disk_scroll, end);
        handle_key(&mut view, KeyCode::PageUp);
        assert!(view.disk_scroll < end);
        handle_key(&mut view, KeyCode::Home);
        assert_eq!(view.disk_scroll, 0);
        handle_key(&mut view, KeyCode::PageDown);
        assert!(view.disk_scroll > 0);
    }

    #[test]
    fn long_review_paths_wrap_and_remain_reachable_on_a_narrow_terminal() {
        let mut view = maintenance_view();
        view.maintenance.as_mut().unwrap().review.as_mut().unwrap().as_mut().unwrap().checkouts[0].path =
            format!("/projects/{}/visible-tail", "long-directory-".repeat(8));
        render(&mut view, 40, 12);
        handle_key(&mut view, KeyCode::End);
        let screen = render(&mut view, 40, 12);
        assert!(screen.replace('│', "").contains("visible-tail"), "{screen}");
        assert!(screen.contains("q quitter"));
    }

    #[test]
    fn disk_scrolling_does_not_change_care_selection_and_care_keys_still_work() {
        let mut view = maintenance_view();
        view.care_selected = 1;
        render(&mut view, 80, 12);
        handle_key(&mut view, KeyCode::Down);
        assert_eq!(view.care_selected, 1);
        let disk_scroll = view.disk_scroll;
        view.tab = CARE_TAB;
        handle_key(&mut view, KeyCode::Down);
        assert_eq!(view.care_selected, 2);
        assert_eq!(view.disk_scroll, disk_scroll);
    }

    #[test]
    fn invalid_review_and_unavailable_snapshot_are_visible_not_successful() {
        let mut view = maintenance_view();
        view.maintenance.as_mut().unwrap().review = Err("date invalide".into());
        let screen = render(&mut view, 120, 40);
        assert!(screen.contains("revue indisponible : date invalide"));
        view.maintenance = None;
        let screen = render(&mut view, 100, 30);
        assert!(screen.contains("données de maintenance indisponibles"));
    }

    #[test]
    fn completed_review_does_not_display_a_partial_label() {
        let mut view = maintenance_view();
        let review = view.maintenance.as_mut().unwrap().review.as_mut().unwrap().as_mut().unwrap();
        review.complete = true;
        review.games[0].complete = true;
        review.errors = 0;
        let screen = render(&mut view, 120, 40);
        assert!(screen.contains("revue complète"));
        assert!(!screen.contains("(partiel)"));
    }

    #[test]
    fn maintenance_reuses_the_facts_cache_until_manual_refresh() {
        let mut view = maintenance_view();
        view.facts_at = Some(Instant::now());
        view.refresh_facts_with(|| panic!("fresh cached facts must not be read again"));
        assert!(handle_key(&mut view, KeyCode::Char('r')));
        assert!(view.facts_at.is_none());
        let mut replacement = view.maintenance.clone().unwrap();
        replacement.journal_retention = Ok(Some("60day".into()));
        view.refresh_facts_with(|| (Facts::default(), replacement));
        assert_eq!(view.maintenance.as_ref().unwrap().journal_retention.as_ref().unwrap().as_deref(), Some("60day"));
        assert!(view.facts_at.is_some());
    }

    #[test]
    fn a_sparkline_spans_its_own_range() {
        assert_eq!(sparkline(&[40.0, 50.0, 60.0]), "▁▅█");
        assert_eq!(sparkline(&[]), "");
    }

    fn event(at: i64, kind: &str) -> store::Event {
        store::Event { at, kind: kind.into(), text: "ArcRaiders.exe (SIGSEGV)".into() }
    }

    #[test]
    fn a_crash_during_a_game_names_the_game() {
        let mut view = View::new();
        view.games = vec![Game { start: 1000, end: Some(5000), name: "ARC Raiders".into(), stats: None }];
        view.events = vec![event(2000, "crash")];
        assert!(journal_entries(&view)[0].3.ends_with(" · pendant ARC Raiders"), "{:?}", journal_entries(&view));
    }

    #[test]
    fn the_journal_lists_the_latest_first() {
        let mut view = View::new();
        view.events = vec![event(2000, "crash")];
        view.ended_alerts = vec![EndedAlert { level: "warn".into(), title: "CPU trop chaud : 91 °C".into(), since: 3000, until: 3600 }];
        let order: Vec<i64> = journal_entries(&view).into_iter().map(|entry| entry.0).collect();
        assert_eq!(order, [3000, 2000]);
    }

    #[test]
    fn the_most_worn_disk_speaks_for_all() {
        let mut view = View::new();
        let disk = |device: &str, worn: u64| crate::facts::Smart { device: device.into(), percentage_used: worn, available_spare: 100, spare_threshold: 10, ..Default::default() };
        view.facts.disks = vec![disk("nvme0n1", 2), disk("nvme1n1", 40)];
        assert_eq!(machine(&view).disk.map(|disk| disk.0), Some("nvme1n1".into()));
    }

    #[test]
    fn every_tab_draws_with_a_game_and_an_event() {
        let backend = ratatui::backend::TestBackend::new(120, 30);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut view = View::new();
        let peaks = Stats { cpu: Some(78.0), hotspot: Some(95.0), minutes_at_limit: 2, ..Default::default() };
        view.games = vec![Game { start: 1000, end: Some(8000), name: "ARC Raiders".into(), stats: Some(peaks.to_json()) }];
        view.events = vec![event(2000, "unclean")];
        for tab in 0..TABS.len() {
            view.tab = tab;
            terminal.draw(|frame| draw(frame, &mut view)).unwrap();
        }
        let screen: String = terminal.backend().buffer().content().iter().map(|cell| cell.symbol()).collect();
        assert!(screen.contains("Arrêt brutal"));
    }

    #[test]
    fn every_tab_draws_without_history() {
        let backend = ratatui::backend::TestBackend::new(100, 30);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut view = View::new();
        for tab in 0..TABS.len() {
            view.tab = tab;
            terminal.draw(|frame| draw(frame, &mut view)).unwrap();
        }
        let screen: String = terminal.backend().buffer().content().iter().map(|cell| cell.symbol()).collect();
        assert!(screen.contains("Tab changer d'onglet"));
    }
}
