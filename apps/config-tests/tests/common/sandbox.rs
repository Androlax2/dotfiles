//! A scratch home, runtime and state folder, and fake commands on PATH that log how they were
//! called: the scripts under test see only this environment, the host stays untouched.
#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// The dotfiles checkout these tests belong to.
pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

pub fn waybar_scripts() -> PathBuf {
    repo().join("config/waybar/.config/waybar/scripts")
}

/// An executable file with `content`, written by a short-lived `sh`, never by this process: tests
/// run in parallel threads, and a file this process holds open for writing is inherited by any
/// child another thread forks meanwhile, which makes running it fail with "Text file busy".
pub fn write_executable(path: &Path, content: &str) {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut writer = Command::new("sh").args(["-c", "cat > \"$1\" && chmod 755 \"$1\"", "sh"]).arg(path).stdin(Stdio::piped()).spawn().unwrap();
    writer.stdin.take().unwrap().write_all(content.as_bytes()).unwrap();
    assert!(writer.wait().unwrap().success(), "cannot write {}", path.display());
}

fn token_hex(bytes: usize) -> String {
    let mut buffer = vec![0u8; bytes];
    if let Ok(mut random) = fs::File::open("/dev/urandom") {
        use std::io::Read;
        let _ = random.read_exact(&mut buffer);
    }
    buffer.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub struct Sandbox {
    pub root: PathBuf,
    pub home: PathBuf,
    pub runtime: PathBuf,
    pub state: PathBuf,
    pub shims: PathBuf,
    pub log: PathBuf,
    /// Added to (and overriding) the test process's own environment for every command.
    pub env: HashMap<String, String>,
}

impl Sandbox {
    pub fn new() -> Sandbox {
        let root = std::env::temp_dir().join(format!("config-test-{}", token_hex(6)));
        let (home, runtime, state, shims) = (root.join("home"), root.join("runtime"), root.join("state"), root.join("shims"));
        for path in [&home, &runtime, &state, &shims] {
            fs::create_dir_all(path).unwrap();
        }
        let log = root.join("calls.log");
        fs::write(&log, "").unwrap();
        let path = format!("{}:{}", shims.display(), std::env::var("PATH").unwrap_or_default());
        let env = HashMap::from([
            ("HOME".to_string(), home.display().to_string()),
            ("XDG_RUNTIME_DIR".into(), runtime.display().to_string()),
            ("XDG_STATE_HOME".into(), state.display().to_string()),
            ("PATH".into(), path),
            ("SHIM_LOG".into(), log.display().to_string()),
        ]);
        Sandbox { root, home, runtime, state, shims, log, env }
    }

    /// A fake command. `body` is bash; $SHIM_LOG receives one line per call by convention.
    pub fn shim(&self, name: &str, body: &str) {
        write_executable(&self.shims.join(name), &format!("#!/usr/bin/env bash\necho \"{name} $*\" >> \"$SHIM_LOG\"\n{body}\n"));
    }

    /// The logged calls of one command, "name arguments" each.
    pub fn calls(&self, name: &str) -> Vec<String> {
        let prefix = format!("{name} ");
        fs::read_to_string(&self.log).unwrap_or_default().lines().filter(|line| line.starts_with(&prefix)).map(str::to_string).collect()
    }

    /// `program` with the sandbox environment, plus `extra` on top.
    pub fn command(&self, program: impl AsRef<std::ffi::OsStr>, extra: &[(&str, &str)]) -> Command {
        let mut command = Command::new(program);
        command.envs(&self.env).envs(extra.iter().copied()).stdin(Stdio::null());
        command
    }

    /// Runs `program arguments` to the end, output captured.
    pub fn run(&self, program: impl AsRef<std::ffi::OsStr>, arguments: &[&str], extra: &[(&str, &str)]) -> Output {
        self.command(program, extra).args(arguments).output().unwrap()
    }

    /// Like `run`, with `input` on stdin.
    pub fn run_with_input(&self, program: impl AsRef<std::ffi::OsStr>, arguments: &[&str], extra: &[(&str, &str)], input: &str) -> Output {
        use std::io::Write;
        let mut child = self.command(program, extra).args(arguments).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().unwrap();
        if let Err(error) = child.stdin.take().unwrap().write_all(input.as_bytes()) {
            // Commands can reject a request before consuming stdin; keep their diagnostics.
            assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe, "cannot write child stdin: {error}");
        }
        child.wait_with_output().unwrap()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
