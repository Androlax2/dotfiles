//! The history: one SQLite file in ~/.local/state/vitals (VITALS_DIR elsewhere, tests).
//!
//!   samples   every reading, once a minute, kept 35 days: the trends compare the last two weeks
//!             with the two before, at idle and under load, which needs each minute's load
//!   hourly    low / mean / high per sensor and hour, kept for good: the long charts
//!   limits    the driver's own limits per sensor, from the last reading
//!   alerts    what is wrong now, and whether Theo was told
//!   alert_history   the alerts that ended, with when they started and ended
//!   games     game sessions: name, start, end, and the peaks reached (JSON, written at the end)
//!   events    crashes, hard stops, memory kills, GPU hangs: what the Journal tab lists

use rusqlite::{Connection, OptionalExtension, params};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const KEEP_SAMPLES_S: i64 = 35 * 86400;

pub fn directory() -> PathBuf {
    std::env::var("VITALS_DIR").map(PathBuf::from).unwrap_or_else(|_| crate::host::state_home().join("vitals"))
}

pub struct Store {
    pub connection: Connection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EndedAlert {
    pub level: String,
    pub title: String,
    pub since: i64,
    pub until: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub start: i64,
    /// None while it is still running.
    pub end: Option<i64>,
    pub name: String,
    /// The peaks of the session (games::Stats as JSON), written when it ends.
    pub stats: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub at: i64,
    /// crash, unclean, memory, gpu, disk, hardware, thermal
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Alert {
    pub key: String,
    /// "warn" or "crit"
    pub level: String,
    pub title: String,
    pub detail: String,
    pub since: i64,
    pub notified: bool,
}

impl Store {
    pub fn open() -> rusqlite::Result<Store> {
        let folder = directory();
        std::fs::create_dir_all(&folder).map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        Self::at(folder.join("vitals.sqlite"))
    }

    pub fn at(path: PathBuf) -> rusqlite::Result<Store> {
        let connection = Connection::open(path)?;
        // The collector writes while the TUI reads.
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        Self::with(connection)
    }

    pub fn in_memory() -> rusqlite::Result<Store> {
        Self::with(Connection::open_in_memory()?)
    }

    fn with(connection: Connection) -> rusqlite::Result<Store> {
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS samples (at INTEGER NOT NULL, sensor TEXT NOT NULL, value REAL NOT NULL);
             CREATE INDEX IF NOT EXISTS samples_by_sensor ON samples (sensor, at);
             CREATE TABLE IF NOT EXISTS hourly (hour INTEGER NOT NULL, sensor TEXT NOT NULL, low REAL NOT NULL, mean REAL NOT NULL,
                                                high REAL NOT NULL, count INTEGER NOT NULL, PRIMARY KEY (hour, sensor));
             CREATE TABLE IF NOT EXISTS limits (sensor TEXT PRIMARY KEY, high REAL, critical REAL);
             CREATE TABLE IF NOT EXISTS alerts (key TEXT PRIMARY KEY, level TEXT NOT NULL, title TEXT NOT NULL, detail TEXT NOT NULL,
                                                since INTEGER NOT NULL, notified INTEGER NOT NULL);
             CREATE TABLE IF NOT EXISTS state (name TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS alert_history (key TEXT NOT NULL, level TEXT NOT NULL, title TEXT NOT NULL,
                                                       since INTEGER NOT NULL, until INTEGER NOT NULL);
             CREATE TABLE IF NOT EXISTS games (start INTEGER PRIMARY KEY, end INTEGER, name TEXT NOT NULL, stats TEXT);
             CREATE TABLE IF NOT EXISTS events (at INTEGER NOT NULL, kind TEXT NOT NULL, text TEXT NOT NULL, PRIMARY KEY (at, kind, text));")?;
        Ok(Store { connection })
    }

    pub fn record(&self, at: i64, sensor: &str, value: f64) -> rusqlite::Result<()> {
        self.connection.execute("INSERT INTO samples (at, sensor, value) VALUES (?1, ?2, ?3)", params![at, sensor, value])?;
        self.record_hour(at, sensor, value)
    }

    pub fn set_limits(&self, sensor: &str, high: Option<f64>, critical: Option<f64>) -> rusqlite::Result<()> {
        self.connection.execute("INSERT INTO limits (sensor, high, critical) VALUES (?1, ?2, ?3)
                                 ON CONFLICT (sensor) DO UPDATE SET high = ?2, critical = ?3", params![sensor, high, critical])?;
        Ok(())
    }

    pub fn limits(&self) -> rusqlite::Result<BTreeMap<String, (Option<f64>, Option<f64>)>> {
        let mut statement = self.connection.prepare("SELECT sensor, high, critical FROM limits")?;
        let rows = statement.query_map([], |row| Ok((row.get(0)?, (row.get(1)?, row.get(2)?))))?;
        rows.collect()
    }

    pub fn forget_before(&self, at: i64) -> rusqlite::Result<()> {
        self.connection.execute("DELETE FROM samples WHERE at < ?1", params![at])?;
        Ok(())
    }

    /// One sensor's samples since `since`, oldest first.
    pub fn series(&self, sensor: &str, since: i64) -> rusqlite::Result<Vec<(i64, f64)>> {
        let mut statement = self.connection.prepare_cached("SELECT at, value FROM samples WHERE sensor = ?1 AND at >= ?2 ORDER BY at")?;
        let rows = statement.query_map(params![sensor, since], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    /// The samples since `since` of the sensors matching one of the SQL LIKE `patterns`, by
    /// minute: what was read together. Only what a check needs: 29 days of every sensor would be
    /// close to a million rows a minute.
    pub fn minutes(&self, since: i64, patterns: &[&str]) -> rusqlite::Result<BTreeMap<i64, BTreeMap<String, f64>>> {
        let mut minutes: BTreeMap<i64, BTreeMap<String, f64>> = BTreeMap::new();
        let mut statement = self.connection.prepare_cached("SELECT at, sensor, value FROM samples WHERE sensor LIKE ?1 AND at >= ?2")?;
        for pattern in patterns {
            let rows = statement.query_map(params![pattern, since], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, f64>(2)?)))?;
            for row in rows {
                let (at, sensor, value) = row?;
                minutes.entry(at - at.rem_euclid(60)).or_default().insert(sensor, value);
            }
        }
        Ok(minutes)
    }

    /// Only the hourly summary: an imported sample older than the minute history keeps.
    pub fn record_hour(&self, at: i64, sensor: &str, value: f64) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO hourly (hour, sensor, low, mean, high, count) VALUES (?1, ?2, ?3, ?3, ?3, 1)
             ON CONFLICT (hour, sensor) DO UPDATE SET low = min(low, ?3), high = max(high, ?3),
                 mean = (mean * count + ?3) / (count + 1), count = count + 1",
            params![at - at.rem_euclid(3600), sensor, value])?;
        Ok(())
    }

    /// low / mean / high per hour for one sensor since `since`.
    pub fn hours(&self, sensor: &str, since: i64) -> rusqlite::Result<Vec<(i64, f64, f64, f64)>> {
        let mut statement = self.connection.prepare_cached("SELECT hour, low, mean, high FROM hourly WHERE sensor = ?1 AND hour >= ?2 ORDER BY hour")?;
        let rows = statement.query_map(params![sensor, since], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?;
        rows.collect()
    }

    pub fn latest(&self) -> rusqlite::Result<BTreeMap<String, (i64, f64)>> {
        let mut statement = self.connection.prepare(
            "SELECT sensor, at, value FROM samples WHERE at = (SELECT max(at) FROM samples)")?;
        let rows = statement.query_map([], |row| Ok((row.get(0)?, (row.get(1)?, row.get(2)?))))?;
        rows.collect()
    }

    pub fn alerts(&self) -> rusqlite::Result<Vec<Alert>> {
        let mut statement = self.connection.prepare("SELECT key, level, title, detail, since, notified FROM alerts ORDER BY level = 'crit' DESC, since")?;
        let rows = statement.query_map([], |row| Ok(Alert { key: row.get(0)?, level: row.get(1)?, title: row.get(2)?, detail: row.get(3)?,
                                                            since: row.get(4)?, notified: row.get::<_, i64>(5)? != 0 }))?;
        rows.collect()
    }

    pub fn save_alert(&self, alert: &Alert) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO alerts (key, level, title, detail, since, notified) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (key) DO UPDATE SET level = ?2, title = ?3, detail = ?4, notified = ?6",
            params![alert.key, alert.level, alert.title, alert.detail, alert.since, alert.notified as i64])?;
        Ok(())
    }

    pub fn drop_alert(&self, key: &str) -> rusqlite::Result<()> {
        self.connection.execute("DELETE FROM alerts WHERE key = ?1", params![key])?;
        Ok(())
    }

    pub fn save_ended_alert(&self, alert: &Alert, until: i64) -> rusqlite::Result<()> {
        self.connection.execute("INSERT INTO alert_history (key, level, title, since, until) VALUES (?1, ?2, ?3, ?4, ?5)",
                                params![alert.key, alert.level, alert.title, alert.since, until])?;
        Ok(())
    }

    /// The alerts that ended after `since`, latest first.
    pub fn ended_alerts(&self, since: i64) -> rusqlite::Result<Vec<EndedAlert>> {
        let mut statement = self.connection.prepare("SELECT level, title, since, until FROM alert_history WHERE until >= ?1 ORDER BY until DESC")?;
        let rows = statement.query_map(params![since], |row| Ok(EndedAlert { level: row.get(0)?, title: row.get(1)?, since: row.get(2)?, until: row.get(3)? }))?;
        rows.collect()
    }

    pub fn start_game(&self, start: i64, name: &str) -> rusqlite::Result<()> {
        self.connection.execute("INSERT OR IGNORE INTO games (start, name) VALUES (?1, ?2)", params![start, name])?;
        Ok(())
    }

    pub fn rename_game(&self, start: i64, name: &str) -> rusqlite::Result<()> {
        self.connection.execute("UPDATE games SET name = ?2 WHERE start = ?1", params![start, name])?;
        Ok(())
    }

    pub fn end_game(&self, start: i64, end: i64, stats: &str) -> rusqlite::Result<()> {
        self.connection.execute("UPDATE games SET end = ?2, stats = ?3 WHERE start = ?1", params![start, end, stats])?;
        Ok(())
    }

    pub fn forget_game(&self, start: i64) -> rusqlite::Result<()> {
        self.connection.execute("DELETE FROM games WHERE start = ?1", params![start])?;
        Ok(())
    }

    pub fn running_game(&self) -> rusqlite::Result<Option<Game>> {
        Ok(self.games(0)?.into_iter().find(|game| game.end.is_none()))
    }

    /// The sessions that started after `since`, latest first.
    pub fn games(&self, since: i64) -> rusqlite::Result<Vec<Game>> {
        let mut statement = self.connection.prepare_cached("SELECT start, end, name, stats FROM games WHERE start >= ?1 ORDER BY start DESC")?;
        let rows = statement.query_map(params![since], |row| Ok(Game { start: row.get(0)?, end: row.get(1)?, name: row.get(2)?, stats: row.get(3)? }))?;
        rows.collect()
    }

    /// True when the event is new: the root data is read again every minute and repeats itself.
    pub fn add_event(&self, event: &Event) -> rusqlite::Result<bool> {
        Ok(self.connection.execute("INSERT OR IGNORE INTO events (at, kind, text) VALUES (?1, ?2, ?3)", params![event.at, event.kind, event.text])? > 0)
    }

    /// The events after `since`, latest first.
    pub fn events(&self, since: i64) -> rusqlite::Result<Vec<Event>> {
        let mut statement = self.connection.prepare("SELECT at, kind, text FROM events WHERE at >= ?1 ORDER BY at DESC")?;
        let rows = statement.query_map(params![since], |row| Ok(Event { at: row.get(0)?, kind: row.get(1)?, text: row.get(2)? }))?;
        rows.collect()
    }

    pub fn state(&self, name: &str) -> rusqlite::Result<Option<String>> {
        self.connection.query_row("SELECT value FROM state WHERE name = ?1", params![name], |row| row.get(0)).optional()
    }

    pub fn set_state(&self, name: &str, value: &str) -> rusqlite::Result<()> {
        self.connection.execute("INSERT INTO state (name, value) VALUES (?1, ?2) ON CONFLICT (name) DO UPDATE SET value = ?2", params![name, value])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory() -> Store {
        Store::in_memory().unwrap()
    }

    #[test]
    fn each_hour_keeps_its_low_mean_and_high() {
        let store = memory();
        for (minute, value) in [(0, 50.0), (1, 60.0), (2, 40.0)] {
            store.record(7200 + minute * 60, "temp:cpu", value).unwrap();
        }
        assert_eq!(store.hours("temp:cpu", 0).unwrap(), [(7200, 40.0, 50.0, 60.0)]);
    }

    #[test]
    fn old_samples_go_and_the_hours_stay() {
        let store = memory();
        store.record(100, "temp:cpu", 50.0).unwrap();
        store.record(90_000, "temp:cpu", 52.0).unwrap();
        store.forget_before(1000).unwrap();
        assert_eq!(store.series("temp:cpu", 0).unwrap(), [(90_000, 52.0)]);
        assert_eq!(store.hours("temp:cpu", 0).unwrap().len(), 2);
    }

    #[test]
    fn an_event_seen_twice_is_kept_once() {
        let store = memory();
        let event = Event { at: 10, kind: "crash".into(), text: "game".into() };
        assert!(store.add_event(&event).unwrap());
        assert!(!store.add_event(&event).unwrap());
        assert_eq!(store.events(0).unwrap().len(), 1);
    }

    #[test]
    fn a_game_runs_until_it_ends() {
        let store = memory();
        store.start_game(100, "Jeu").unwrap();
        store.rename_game(100, "ARC Raiders").unwrap();
        assert_eq!(store.running_game().unwrap().map(|game| game.name), Some("ARC Raiders".into()));
        store.end_game(100, 4000, "{}").unwrap();
        assert_eq!(store.running_game().unwrap(), None);
        assert_eq!(store.games(0).unwrap()[0].end, Some(4000));
    }

    #[test]
    fn an_alert_keeps_the_time_it_started() {
        let store = memory();
        let mut alert = Alert { key: "cpu".into(), level: "warn".into(), title: "t".into(), detail: "d".into(), since: 10, notified: false };
        store.save_alert(&alert).unwrap();
        alert.since = 99;
        alert.notified = true;
        store.save_alert(&alert).unwrap();
        assert_eq!(store.alerts().unwrap()[0].since, 10);
        assert!(store.alerts().unwrap()[0].notified);
    }
}
