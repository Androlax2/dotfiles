//! Load the real tmux configuration on a private server, never the host's server.
mod common;
use common::sandbox::*;
use std::fs;
use std::process::Output;

struct Tmux {
    sandbox: Sandbox,
}

impl Tmux {
    fn new() -> Self {
        let sandbox = Sandbox::new();
        let theme = sandbox.home.join(".config/tmux");
        fs::create_dir_all(&theme).unwrap();
        fs::copy(repo().join("config/tmux/.config/tmux/tokyonight_night.tmux"), theme.join("tokyonight_night.tmux")).unwrap();
        let tmux = Self { sandbox };
        tmux.run(&["-f", "/dev/null", "new-session", "-d", "-s", "bootstrap", "sleep 60"]);
        tmux.run(&["source-file", repo().join("config/tmux/.config/tmux/tmux.conf").to_str().unwrap()]);
        tmux
    }

    fn run(&self, arguments: &[&str]) -> Output {
        let output = self.sandbox.command("/usr/bin/tmux", &[])
            .env_remove("TMUX")
            .args(["-S", self.sandbox.root.join("tmux.sock").to_str().unwrap()])
            .args(arguments).output().unwrap();
        assert!(output.status.success(), "{arguments:?}: {}", stderr(&output));
        output
    }
}

impl Drop for Tmux {
    fn drop(&mut self) {
        let _ = self.sandbox.command("/usr/bin/tmux", &[])
            .args(["-S", self.sandbox.root.join("tmux.sock").to_str().unwrap(), "kill-server"])
            .output();
    }
}

#[test]
fn the_tmux_server_passes_a_panes_clipboard_copies_to_the_terminal() {
    // opencode copies only through OSC 52; the default (external) drops a pane's OSC 52.
    let tmux = Tmux::new();
    let output = tmux.run(&["show-options", "-s", "-v", "set-clipboard"]);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "on");
}
