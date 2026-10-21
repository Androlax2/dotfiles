//! Game sessions. The collector looks once a minute: a game runs while GameMode has the machine
//! (game-performance keeps its list in $XDG_RUNTIME_DIR from a game's start to its end) or while a
//! game window has the focus. When it ends, the session keeps its peaks: the minute history they
//! come from is gone after 35 days.

use crate::checks::{Limits, Minutes, high_of};
use crate::store::{Game, Store};
use serde_json::{Value, json};

/// Until a game window is seen with its title.
pub const UNNAMED: &str = "Jeu";
/// Shorter is a launcher opened and closed, not a session.
const SHORTEST_S: i64 = 5 * 60;
pub const CPU: &str = "temp:k10temp/Tctl";
pub const GPU: &str = "temp:amdgpu/edge";
pub const HOTSPOT: &str = "temp:amdgpu/junction";
pub const VRAM: &str = "temp:amdgpu/mem";
pub const POWER: &str = "power:gpu";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stats {
    pub cpu: Option<f64>,
    pub gpu: Option<f64>,
    pub hotspot: Option<f64>,
    pub vram: Option<f64>,
    pub power: Option<f64>,
    /// Minutes with at least one temperature at or over its limit.
    pub minutes_at_limit: u64,
}

impl Stats {
    pub fn to_json(&self) -> String {
        json!({"cpu": self.cpu, "gpu": self.gpu, "hotspot": self.hotspot, "vram": self.vram, "power": self.power,
               "minutes_at_limit": self.minutes_at_limit}).to_string()
    }

    pub fn from_json(text: &str) -> Stats {
        let Ok(value) = serde_json::from_str::<Value>(text) else { return Stats::default() };
        let number = |key: &str| value.get(key).and_then(Value::as_f64);
        Stats { cpu: number("cpu"), gpu: number("gpu"), hotspot: number("hotspot"), vram: number("vram"), power: number("power"),
                minutes_at_limit: value.get("minutes_at_limit").and_then(Value::as_u64).unwrap_or(0) }
    }
}

/// The peaks of the minutes a session lasted.
pub fn stats(minutes: &Minutes, limits: &Limits) -> Stats {
    let peak = |sensor: &str| minutes.values().filter_map(|readings| readings.get(sensor).copied()).reduce(f64::max);
    let minutes_at_limit = minutes.values()
        .filter(|readings| readings.iter().any(|(sensor, value)| sensor.starts_with("temp:") && *value >= high_of(sensor, limits)))
        .count() as u64;
    Stats { cpu: peak(CPU), gpu: peak(GPU), hotspot: peak(HOTSPOT), vram: peak(VRAM), power: peak(POWER), minutes_at_limit }
}

#[derive(Debug, PartialEq)]
pub enum Step {
    Start(String),
    Rename(String),
    End,
    Stay,
}

/// What a minute changes: `playing` from GameMode or the focus, `title` the focused game's.
pub fn step(running: Option<&Game>, playing: bool, title: Option<&str>) -> Step {
    match (running, playing) {
        (None, true) => Step::Start(title.unwrap_or(UNNAMED).to_string()),
        (Some(game), true) => match title {
            Some(title) if game.name == UNNAMED => Step::Rename(title.to_string()),
            _ => Step::Stay,
        },
        (Some(_), false) => Step::End,
        (None, false) => Step::Stay,
    }
}

/// GameMode's game: game-performance's list exists from `on` to `off`.
fn game_mode_on() -> bool {
    crate::host::runtime_dir().join("game-performance.paused").exists()
}

/// Called each minute by the collector.
pub fn follow(store: &Store, now: i64) -> rusqlite::Result<()> {
    let title = crate::host::active_game();
    let playing = title.is_some() || game_mode_on();
    let running = store.running_game()?;
    match step(running.as_ref(), playing, title.as_deref()) {
        Step::Start(name) => store.start_game(now, &name),
        Step::Rename(name) => store.rename_game(running.map(|game| game.start).unwrap_or(now), &name),
        Step::End => {
            let Some(game) = running else { return Ok(()) };
            end(store, &game, now)
        }
        Step::Stay => Ok(()),
    }
}

pub fn end(store: &Store, game: &Game, now: i64) -> rusqlite::Result<()> {
    if now - game.start < SHORTEST_S {
        return store.forget_game(game.start);
    }
    let minutes: Minutes = store.minutes(game.start, &["temp:%", POWER])?.into_iter().filter(|(at, _)| *at <= now).collect();
    store.end_game(game.start, now, &stats(&minutes, &store.limits()?).to_json())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn running(name: &str) -> Game {
        Game { start: 0, end: None, name: name.into(), stats: None }
    }

    #[test]
    fn a_game_starts_with_the_focused_title() {
        assert_eq!(step(None, true, Some("ARC Raiders")), Step::Start("ARC Raiders".into()));
    }

    #[test]
    fn a_game_started_out_of_focus_gets_its_name_once_seen() {
        assert_eq!(step(None, true, None), Step::Start(UNNAMED.into()));
        assert_eq!(step(Some(&running(UNNAMED)), true, Some("ARC Raiders")), Step::Rename("ARC Raiders".into()));
    }

    #[test]
    fn a_named_game_keeps_its_name() {
        assert_eq!(step(Some(&running("ARC Raiders")), true, Some("Steam")), Step::Stay);
    }

    #[test]
    fn a_game_ends_when_game_mode_and_the_window_are_gone() {
        assert_eq!(step(Some(&running("ARC Raiders")), false, None), Step::End);
    }

    #[test]
    fn a_session_keeps_its_peaks_and_its_minutes_at_the_limit() {
        let minute = |cpu: f64, hotspot: f64| BTreeMap::from([(CPU.to_string(), cpu), (HOTSPOT.to_string(), hotspot), (POWER.to_string(), 250.0)]);
        let minutes = Minutes::from([(60, minute(70.0, 90.0)), (120, minute(78.0, 104.0)), (180, minute(74.0, 105.0))]);
        let limits = Limits::from([(HOTSPOT.to_string(), (Some(100.0), Some(110.0)))]);
        let peaks = stats(&minutes, &limits);
        assert_eq!((peaks.cpu, peaks.hotspot, peaks.power, peaks.minutes_at_limit), (Some(78.0), Some(105.0), Some(250.0), 2));
    }

    #[test]
    fn stats_survive_the_store() {
        let peaks = Stats { cpu: Some(78.0), gpu: None, hotspot: Some(92.5), vram: None, power: Some(240.0), minutes_at_limit: 3 };
        assert_eq!(Stats::from_json(&peaks.to_json()), peaks);
    }

    #[test]
    fn a_launcher_opened_for_a_minute_is_no_session() {
        let store = Store::in_memory().unwrap();
        store.start_game(1000, UNNAMED).unwrap();
        end(&store, &store.running_game().unwrap().unwrap(), 1060).unwrap();
        assert!(store.games(0).unwrap().is_empty());
    }

    #[test]
    fn an_ended_session_is_saved_with_its_peaks() {
        let store = Store::in_memory().unwrap();
        store.start_game(1000, "ARC Raiders").unwrap();
        store.record(1060, CPU, 75.0).unwrap();
        end(&store, &store.running_game().unwrap().unwrap(), 4600).unwrap();
        let game = &store.games(0).unwrap()[0];
        assert_eq!(game.end, Some(4600));
        assert_eq!(Stats::from_json(game.stats.as_deref().unwrap()).cpu, Some(75.0));
    }
}
