//! The cooler animation starts without a resident GUI preview.

mod common;
use common::sandbox::*;

use std::process::Command;

#[test]
fn login_starts_only_the_trcc_daemon() {
    let module = repo().join("config/hyprland/.config/hypr/autostart.lua");
    let script = format!(r#"
local start
hl = {{
    on = function(event, callback)
        if event == "hyprland.start" then start = callback end
    end,
    exec_cmd = function(command)
        if command:find("trcc", 1, true) then print(command) end
    end,
}}
dofile({module:?})
assert(start, "Hyprland startup handler is registered")
start()
"#, module = module.display().to_string());
    let output = Command::new("lua5.4").args(["-e", &script]).output().expect("lua5.4 runs the Hyprland config's Lua");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "~/.local/bin/trcc daemon");
}
