//! The bar's next-event module (config/waybar/.config/waybar/scripts/calendar.sh). Waybar runs it
//! every minute, so anything it leaves running piles up (369 leftover sleeps on 2026-10-24).

mod common;
use common::sandbox::*;

#[test]
fn showing_an_event_leaves_no_background_process_behind() {
    let sandbox = Sandbox::new();
    sandbox.shim("khal", r#"echo "$(date -d '+30 min' '+%d/%m/%Y %H:%M')|$(date -d '+3 hours' '+%d/%m/%Y %H:%M')|Standup""#);
    sandbox.shim("pkill", "");
    sandbox.shim("sleep", "");
    let output = sandbox.run(waybar_scripts().join("calendar.sh"), &[], &[]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("Standup"), "{}", stdout(&output));
    // A detached waiter would call sleep after the script returns: give it time to show up.
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(sandbox.calls("sleep").is_empty(), "{:?}", sandbox.calls("sleep"));
}
