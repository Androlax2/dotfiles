//! The bar's audio output switcher (config/waybar/.config/waybar/scripts/switch-audio.sh).

mod common;
use common::sandbox::*;

#[test]
fn switching_audio_refreshes_only_the_exact_waybar_process_name() {
    let sandbox = Sandbox::new();
    sandbox.shim("pactl", "");
    sandbox.shim("notify-send", "");
    sandbox.shim("pkill", "");
    let output = sandbox.run(waybar_scripts().join("switch-audio.sh"), &[], &[]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(sandbox.calls("pkill"), ["pkill -x -RTMIN+8 waybar"]);
}
