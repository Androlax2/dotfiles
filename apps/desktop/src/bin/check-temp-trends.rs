//! check-temp-trends: prints the cooling verdict for ~/.local/share/templogs/temps.log.
//! Exit 0 when steady or too early to tell, 2 when a baseline rose, 1 on an unreadable log.

use desktop::temperature::{Verdict, check};

fn main() {
    let path = cockpit::paths::home().join(".local/share/templogs/temps.log");
    let log = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            println!("No log file yet.");
            return;
        }
        Err(error) => {
            eprintln!("check-temp-trends: cannot read {}: {error}", path.display());
            std::process::exit(1);
        }
    };
    match check(&log) {
        Ok(Verdict::TooEarly(message) | Verdict::Steady(message)) => println!("{message}"),
        Ok(Verdict::Warming(message)) => {
            println!("{message}");
            std::process::exit(2);
        }
        Err(error) => {
            eprintln!("check-temp-trends: {}: {error}", path.display());
            std::process::exit(1);
        }
    }
}
