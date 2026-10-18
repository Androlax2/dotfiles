//! The TUI: seven tabs over the history the collector keeps, plus live sensor readings.
//!
//!   1 Overview   what is wrong now, every temperature (now, 24 h range, limit, 24 h curve), fans, load
//!   2 Cooling    CPU and GPU over 90 days (daily coolest and hottest), the idle trend the checks use
//!   3 Disks      SMART health and space
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
               games: Vec::new(), fill_forecast: BTreeMap::new() }
    }

    fn refresh(&mut self) {
        let root = sensors::sys_root();
        self.readings = sensors::read_hwmon(&root);
        if self.facts_at.is_none_or(|at| at.elapsed() >= FACTS_REFRESH) {
            self.facts = facts::gather(&root);
            self.facts_at = Some(Instant::now());
        }
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
    let [tabs_area, state_area] = Layout::horizontal([Constraint::Min(30), Constraint::Length(40)]).areas(area);
    let titles: Vec<Line> = TABS.iter().enumerate().map(|(index, title)| Line::from(format!("{} {title}", index + 1))).collect();
    frame.render_widget(Tabs::new(titles).select(view.tab).highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)), tabs_area);
    let state = match (view.alerts.iter().map(|alert| alert.level.as_str()).max_by_key(|level| *level == "crit"), view.collected_at) {
        (_, None) => Span::styled("aucune collecte pour l'instant", level_style("warn")),
        (None, Some(at)) => Span::styled(format!("tout va bien · collecté il y a {} s", cockpit::util::now_secs() - at), level_style("ok")),
        (Some(level), Some(_)) => Span::styled(format!("{} problème(s)", view.alerts.len()), level_style(level)),
    };
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

fn disks(frame: &mut Frame, area: Rect, view: &View) {
    let mut lines = Vec::new();
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
    frame.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Disques ")), area);
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

fn draw(frame: &mut Frame, view: &View) {
    let [top, body, footer] = Layout::vertical([Constraint::Length(1), Constraint::Min(5), Constraint::Length(1)]).areas(frame.area());
    header(frame, top, view);
    match view.tab {
        0 => overview(frame, body, view),
        1 => cooling(frame, body, view),
        2 => disks(frame, body, view),
        3 => system(frame, body, view),
        CARE_TAB => care_tab(frame, body, view),
        5 => games_tab(frame, body, view),
        _ => journal_tab(frame, body, view),
    }
    let keys = if view.tab == CARE_TAB { "↑↓ choisir · d fait aujourd'hui · b une semaine plus tôt · 1-7 / Tab changer d'onglet · q quitter" } else { "1-7 / Tab changer d'onglet · r rafraîchir · q quitter" };
    frame.render_widget(Paragraph::new(Line::styled(keys, dim())), footer);
}

fn event_loop(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut view = View::new();
    view.refresh();
    let mut refreshed = Instant::now();
    loop {
        terminal.draw(|frame| draw(frame, &view))?;
        let wait = REFRESH.saturating_sub(refreshed.elapsed());
        if event::poll(wait)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char(digit @ '1'..='7') => view.tab = digit as usize - '1' as usize,
                    KeyCode::Up | KeyCode::Char('k') if view.tab == CARE_TAB => view.care_selected = view.care_selected.saturating_sub(1),
                    KeyCode::Down | KeyCode::Char('j') if view.tab == CARE_TAB => view.care_selected = (view.care_selected + 1).min(care::TASKS.len() - 1),
                    KeyCode::Char('d') if view.tab == CARE_TAB => record_care(&mut view, false),
                    KeyCode::Char('b') if view.tab == CARE_TAB => record_care(&mut view, true),
                    KeyCode::Tab | KeyCode::Right => view.tab = (view.tab + 1) % TABS.len(),
                    KeyCode::BackTab | KeyCode::Left => view.tab = (view.tab + TABS.len() - 1) % TABS.len(),
                    KeyCode::Char('r') => {
                        view.facts_at = None;
                        view.refresh();
                        refreshed = Instant::now();
                    }
                    _ => {}
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
            terminal.draw(|frame| draw(frame, &view)).unwrap();
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
            terminal.draw(|frame| draw(frame, &view)).unwrap();
        }
        let screen: String = terminal.backend().buffer().content().iter().map(|cell| cell.symbol()).collect();
        assert!(screen.contains("Tab changer d'onglet"));
    }
}
