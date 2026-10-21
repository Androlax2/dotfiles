//! What vitals needs from the machine around it: its folders, the clock, JSON files, commands,
//! whether a game runs, and the OSD daemon it warns through.

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".into()))
}

/// $XDG_RUNTIME_DIR, or /tmp without one.
pub fn runtime_dir() -> PathBuf {
    PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into()))
}

/// $XDG_STATE_HOME, or ~/.local/state without one.
pub fn state_home() -> PathBuf {
    std::env::var("XDG_STATE_HOME").map(PathBuf::from).unwrap_or_else(|_| home().join(".local/state"))
}

pub fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_secs() as i64).unwrap_or(0)
}

pub fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

/// Written whole or not at all: a reader (the bar) never sees half a file.
pub fn write_json(path: &Path, value: &Value) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(format!(".tmp-{}", token_hex(6)));
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    file.write_all(value.to_string().as_bytes())?;
    drop(file);
    fs::rename(&temporary, path).inspect_err(|_| {
        let _ = fs::remove_file(&temporary);
    })
}

/// A command's stdout when it succeeds.
pub fn output_of(program: &str, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program).args(arguments).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn token_hex(bytes: usize) -> String {
    let mut buffer = vec![0u8; bytes];
    if let Ok(mut random) = fs::File::open("/dev/urandom") {
        use std::io::Read;
        let _ = random.read_exact(&mut buffer);
    }
    buffer.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Whether a game has the screen: Hyprland's game mode, or a game window in front.
pub fn gaming() -> bool {
    let submap = output_of("hyprctl", &["submap"]).unwrap_or_default();
    submap.trim() == "gamemode" || active_game().is_some()
}

/// The title of the game window in front, if one is.
pub fn active_game() -> Option<String> {
    static GAME_CLASS: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^(steam_app_[0-9]+|Star Citizen|Dofus\.x64|retroarch)$").unwrap());
    let window: Value = serde_json::from_str(&output_of("hyprctl", &["-j", "activewindow"])?).ok()?;
    let class = window.get("class").and_then(Value::as_str)?;
    let title = window.get("title").and_then(Value::as_str).map(str::trim).filter(|title| !title.is_empty());
    GAME_CLASS.is_match(class).then(|| title.unwrap_or(class).to_string())
}

/// One line to the OSD daemon's host-only FIFO ($XDG_RUNTIME_DIR/osd-host.fifo, from agent-cockpit):
/// `notice {"id": "vitals-<key>", "title": …, "message": …}` or `notice-clear vitals-<key>`. False,
/// and nothing waits, when no daemon reads it.
pub fn send_notice(line: &str) -> bool {
    send_line(&runtime_dir().join("osd-host.fifo"), line)
}

fn send_line(fifo: &Path, line: &str) -> bool {
    use std::os::unix::fs::OpenOptionsExt;
    const O_NONBLOCK: i32 = 0o4000;
    let Ok(mut fifo) = fs::OpenOptions::new().write(true).custom_flags(O_NONBLOCK).open(fifo) else {
        return false;
    };
    fifo.write_all(format!("{line}\n").as_bytes()).is_ok()
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    #[test]
    fn a_notice_without_a_running_osd_is_dropped_at_once() {
        let folder = std::env::temp_dir().join(format!("vitals-notice-{}", super::token_hex(6)));
        std::fs::create_dir_all(&folder).unwrap();
        let fifo = folder.join("osd-host.fifo");
        assert!(!super::send_line(&fifo, "notice-clear vitals-x"), "no FIFO at all");
        assert!(std::process::Command::new("mkfifo").arg(&fifo).status().unwrap().success());
        let started = Instant::now();
        assert!(!super::send_line(&fifo, "notice-clear vitals-x"), "a FIFO nobody reads");
        assert!(started.elapsed() < Duration::from_secs(1));
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
