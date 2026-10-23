//! Window rules emitted for Agent Cockpit's host-side helpers.

mod common;
use common::sandbox::*;

use std::process::Command;

#[test]
fn the_cockpit_guide_gets_a_floating_opaque_window_beside_the_browser() {
    let module = repo().join("config/hyprland/.config/hypr/apps/claude.lua");
    let script = format!(r#"
package.preload.colors = function() return {{red = "", yellow = "", blue = ""}} end
hl = {{
    window_rule = function(rule)
        if rule.match.class == "^cockpit-guide$" then
            print(table.concat({{tostring(rule.float), rule.opacity, rule.size, rule.move}}, "|"))
        end
    end,
    workspace_rule = function() end,
    layer_rule = function() end,
}}
dofile({module:?})
"#, module = module.display().to_string());
    let output = Command::new("lua5.4").args(["-e", &script]).output().expect("lua5.4 runs the Hyprland config's Lua");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(),
        "true|1 1|560 (monitor_h*0.85)|(monitor_w-window_w-24) (monitor_h*0.08)");
}
