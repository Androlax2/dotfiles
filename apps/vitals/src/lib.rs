//! vitals: watches the machine (temperatures and their trends, fans, disks, services, kernel
//! errors, memory, the backup), keeps the history, warns through the OSD and the bar, and shows it
//! all in a TUI.

pub mod care;
pub mod checks;
pub mod collect;
pub mod facts;
pub mod games;
pub mod import;
pub mod journal;
pub mod sensors;
pub mod store;
pub mod summary;
pub mod tui;
