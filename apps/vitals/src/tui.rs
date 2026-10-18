//! The TUI: four tabs over the history the collector keeps, plus live sensor readings.
//!
//!   1 Overview   what is wrong now, every temperature (now, 24 h range, limit, 24 h curve), fans, load
//!   2 Cooling    CPU and GPU over 90 days (daily coolest and hottest), the idle trend the checks use
//!   3 Disks      SMART health and space
//!   4 System     failed services, kernel errors, memory, the backup
//!
//! Tab / ← → or 1-4 switch, r refreshes, q or Esc quits. Refreshes every 5 s on its own.

use crate::checks::{self, DAY, TRENDED};
use crate::facts::{self, Facts};
use crate::sensors::{self, Kind, Reading, label};
use crate::store::{Alert, Store};
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
const TABS: [&str; 4] = ["Overview", "Cooling", "Disks", "System"];
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
               long: BTreeMap::new(), idle: BTreeMap::new(), facts: Facts::default(), facts_at: None, error: None, collected_at: None }
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
            Err(error) => self.error = Some(format!("history: {error}")),
        }
    }

    fn load_history(&mut self) -> rusqlite::Result<()> {
        let store = Store::open()?;
        let now = cockpit::util::now_secs();
        self.alerts = store.alerts()?;
        self.limits = store.limits()?;
        let latest = store.latest()?;
        self.collected_at = latest.values().map(|(at, _)| *at).max();
        self.loads = latest.iter().filter(|(sensor, _)| sensor.starts_with("load:")).map(|(sensor, (_, value))| (sensor.clone(), *value)).collect();
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
    let [tabs_area, state_area] = Layout::horizontal([Constraint::Min(30), Constraint::Length(36)]).areas(area);
    let titles: Vec<Line> = TABS.iter().enumerate().map(|(index, title)| Line::from(format!("{} {title}", index + 1))).collect();
    frame.render_widget(Tabs::new(titles).select(view.tab).highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)), tabs_area);
    let state = match (view.alerts.iter().map(|alert| alert.level.as_str()).max_by_key(|level| *level == "crit"), view.collected_at) {
        (_, None) => Span::styled("no collection yet", level_style("warn")),
        (None, Some(at)) => Span::styled(format!("all fine · collected {}s ago", cockpit::util::now_secs() - at), level_style("ok")),
        (Some(level), Some(_)) => Span::styled(format!("{} problem(s)", view.alerts.len()), level_style(level)),
    };
    frame.render_widget(Paragraph::new(Line::from(state)).right_aligned(), state_area);
}

fn alerts_lines(view: &View) -> Vec<Line<'_>> {
    if view.alerts.is_empty() {
        return vec![Line::styled("Nothing wrong.", level_style("ok"))];
    }
    view.alerts.iter().flat_map(|alert| [
        Line::from(vec![Span::styled(if alert.level == "crit" { "● " } else { "▲ " }, level_style(&alert.level)),
                        Span::styled(alert.title.as_str(), Style::default().add_modifier(Modifier::BOLD))]),
        Line::styled(format!("  {}", alert.detail), dim()),
    ]).collect()
}

fn overview(frame: &mut Frame, area: Rect, view: &View) {
    let alert_height = (view.alerts.len().max(1) * 2 + 2) as u16;
    let [alerts_area, table_area] = Layout::vertical([Constraint::Length(alert_height.min(area.height / 2)), Constraint::Min(5)]).areas(area);
    frame.render_widget(Paragraph::new(alerts_lines(view)).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).title(" Now ")), alerts_area);
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
        rows.push(Row::new(vec![Cell::from(label(&reading.key)), Cell::from(format!("{:.0} RPM", reading.value)), Cell::from(""), Cell::from(""), Cell::from("")]));
    }
    for (sensor, value) in &view.loads {
        rows.push(Row::new(vec![Cell::from(label(sensor)), Cell::from(format!("{value:.0}%")), Cell::from(""), Cell::from(""), Cell::from("")]));
    }
    let table = Table::new(rows, [Constraint::Length(18), Constraint::Length(10), Constraint::Length(12), Constraint::Length(7), Constraint::Min(24)])
        .header(Row::new(["Sensor", "Now", "24 h", "Limit", "Last 24 h"]).style(dim()))
        .block(Block::default().borders(Borders::ALL).title(" Sensors "));
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
            (Some(recent), Some(before)) => format!("idle {recent:.1}°C (two weeks before: {before:.1}°C, {:+.1})", recent - before),
            (Some(recent), None) => format!("idle {recent:.1}°C (not enough history to compare yet)"),
            _ => "idle: not enough history yet".into(),
        };
        let datasets = vec![
            Dataset::default().name("hottest").marker(Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(Color::Red)).data(&hottest),
            Dataset::default().name("coolest").marker(Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(Color::Cyan)).data(&coolest),
        ];
        let chart = Chart::new(datasets)
            .block(Block::default().borders(Borders::ALL).title(format!(" {} · {CHART_DAYS} days · {idle} ", name.to_uppercase())))
            .x_axis(Axis::default().bounds([first, last.max(first + 1.0)]).labels([format!("{} days ago", (last - first) as i64), "today".into()]).style(dim()))
            .y_axis(Axis::default().bounds([20.0, top]).labels(["20°C".to_string(), format!("{:.0}°C", (20.0 + top) / 2.0), format!("{top:.0}°C")]).style(dim()));
        frame.render_widget(chart, areas[index]);
    }
}

fn disks(frame: &mut Frame, area: Rect, view: &View) {
    let mut lines = Vec::new();
    if view.facts.disks.is_empty() {
        lines.push(Line::styled("No SMART data: vitals-root.timer (a system unit) writes it to /var/lib/vitals/smart.", level_style("warn")));
    }
    for disk in &view.facts.disks {
        let healthy = disk.critical_warning == 0 && disk.available_spare >= disk.spare_threshold;
        lines.push(Line::from(vec![Span::styled(format!("{}  ", disk.device), Style::default().add_modifier(Modifier::BOLD)),
                                   Span::styled(if healthy { "healthy" } else { "FAILING" }, level_style(if healthy { "ok" } else { "crit" }))]));
        lines.push(Line::from(format!("  worn {}%   spare {}% (threshold {}%)   media errors {}   critical warning {:#04x}",
                                      disk.percentage_used, disk.available_spare, disk.spare_threshold, disk.media_errors, disk.critical_warning)));
        lines.push(Line::styled(format!("  read {} min ago", (cockpit::util::now_secs() - disk.read_at) / 60), dim()));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled("Space", Style::default().add_modifier(Modifier::BOLD)));
    for (mount, used) in &view.facts.disk_usage {
        let filled = (*used / 5.0).round() as usize;
        lines.push(Line::from(vec![Span::raw(format!("  {mount:<16} ")), Span::styled("█".repeat(filled), level_style(if *used >= 90.0 { "warn" } else { "ok" })),
                                   Span::styled("░".repeat(20 - filled.min(20)), dim()), Span::raw(format!(" {used:.0}%"))]));
    }
    frame.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Disks ")), area);
}

fn system(frame: &mut Frame, area: Rect, view: &View) {
    let facts = &view.facts;
    let mut lines = vec![Line::styled("Services", Style::default().add_modifier(Modifier::BOLD))];
    if facts.failed_units.is_empty() {
        lines.push(Line::styled("  none failed", level_style("ok")));
    }
    lines.extend(facts.failed_units.iter().map(|unit| Line::styled(format!("  ✗ {unit}"), level_style("warn"))));
    lines.push(Line::from(""));
    lines.push(Line::styled("Kernel errors (24 h)", Style::default().add_modifier(Modifier::BOLD)));
    if facts.kernel_errors.is_empty() {
        lines.push(Line::styled("  none of the kinds watched (GPU resets, disk I/O, machine checks, throttling)", level_style("ok")));
    }
    for (category, entries) in &facts.kernel_errors {
        lines.push(Line::styled(format!("  {category}: {} line(s)", entries.len()), level_style("warn")));
        lines.extend(entries.iter().rev().take(3).map(|line| Line::styled(format!("    {line}"), dim())));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled("Memory", Style::default().add_modifier(Modifier::BOLD)));
    lines.push(Line::from(format!("  {} free   pressure {}",
        facts.memory_available_percent.map(|free| format!("{free:.0}%")).unwrap_or("?".into()),
        facts.memory_pressure.map(|pressure| format!("{pressure:.1}%")).unwrap_or("?".into()))));
    lines.push(Line::from(""));
    lines.push(Line::styled("Backup", Style::default().add_modifier(Modifier::BOLD)));
    lines.push(Line::from(match facts.last_backup.or_else(|| Store::open().ok()?.state("last_backup").ok()?.and_then(|stamp| stamp.parse().ok())) {
        Some(at) => format!("  last good backup {} h ago{}", (cockpit::util::now_secs() - at) / 3600,
                            facts.backup_result.as_ref().filter(|result| *result != "success").map(|result| format!(" · last run: {result}")).unwrap_or_default()),
        None => "  no backup seen yet".into(),
    }));
    if let Some(error) = &view.error {
        lines.push(Line::from(""));
        lines.push(Line::styled(error.as_str(), level_style("crit")));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).title(" System ")), area);
}

fn draw(frame: &mut Frame, view: &View) {
    let [top, body, footer] = Layout::vertical([Constraint::Length(1), Constraint::Min(5), Constraint::Length(1)]).areas(frame.area());
    header(frame, top, view);
    match view.tab {
        0 => overview(frame, body, view),
        1 => cooling(frame, body, view),
        2 => disks(frame, body, view),
        _ => system(frame, body, view),
    }
    frame.render_widget(Paragraph::new(Line::styled("1-4 / Tab switch · r refresh · q quit", dim())), footer);
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
                    KeyCode::Char(digit @ '1'..='4') => view.tab = digit as usize - '1' as usize,
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
        assert!(screen.contains("1-4 / Tab switch"));
    }
}
