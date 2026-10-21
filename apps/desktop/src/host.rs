//! Where the desktop programs find the session's folders, and how they run a command with a limit.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".into()))
}

/// $XDG_RUNTIME_DIR, or /tmp without one.
pub fn runtime_dir() -> PathBuf {
    PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into()))
}

/// A command's exit code, stdout and stderr, or None when it cannot start or outlives `timeout`.
pub fn output_within(command: &[&str], timeout: Duration) -> Option<(i32, String, String)> {
    let mut child = Command::new(command[0]).args(&command[1..]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().ok()?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(_) => return None,
        }
    }
    let output = child.wait_with_output().ok()?;
    Some((output.status.code().unwrap_or(-1), String::from_utf8_lossy(&output.stdout).into_owned(), String::from_utf8_lossy(&output.stderr).into_owned()))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    fn a_command_that_outlives_its_limit_gives_nothing() {
        assert_eq!(super::output_within(&["sleep", "2"], Duration::from_millis(50)), None);
    }

    #[test]
    fn a_command_gives_its_code_and_output() {
        assert_eq!(super::output_within(&["sh", "-c", "echo out; echo err >&2; exit 3"], Duration::from_secs(5)),
                   Some((3, "out\n".to_string(), "err\n".to_string())));
    }
}
