//! vitals                 the TUI
//! vitals collect         one collection (vitals-collect.timer, every minute)
//! vitals import [log]    the temperature log of log-temps.sh into the history, once
//! vitals status          the bar's dot, as waybar JSON

use serde_json::{Value, json};
use vitals::{collect, import, store::Store, tui};

/// Past this the collector is not running: the dot says so instead of a reassuring silence.
const STALE_AFTER_S: i64 = 5 * 60;

fn status() -> Value {
    let Some(state) = cockpit::util::read_json(&collect::status_path()) else {
        return json!({"text": "●", "class": "stale", "tooltip": "vitals : aucune collecte pour l'instant (vitals-collect.timer)"});
    };
    let at = state["at"].as_i64().unwrap_or(0);
    if cockpit::util::now_secs() - at > STALE_AFTER_S {
        return json!({"text": "●", "class": "stale", "tooltip": "vitals : la collecte n'a pas tourné depuis 5 minutes (vitals-collect.timer)"});
    }
    let level = state["level"].as_str().unwrap_or("ok");
    if level == "ok" {
        return json!({"text": "", "class": "ok", "tooltip": "PC : tout va bien"});
    }
    let titles: Vec<&str> = state["alerts"].as_array().map(|alerts| alerts.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
    // The chip says what is wrong; with several problems, how many.
    let text = match titles.as_slice() {
        [only] => format!("● {only}"),
        many => format!("● {} problèmes sur le PC", many.len()),
    };
    json!({"text": text, "class": level, "tooltip": format!("PC :\n{}", titles.join("\n"))})
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let result = match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => tui::run(),
        ["collect"] => collect::run(),
        ["status"] => {
            println!("{}", status());
            Ok(())
        }
        ["import", rest @ ..] if rest.len() <= 1 => {
            let path = rest.first().map(std::path::PathBuf::from).unwrap_or_else(|| cockpit::paths::home().join(".local/share/templogs/temps.log"));
            std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display())).and_then(|log| {
                let store = Store::open().map_err(|error| error.to_string())?;
                // Twice would count every sample twice in the hourly summaries.
                if store.state("imported_temps_log").map_err(|error| error.to_string())?.is_some() {
                    return Err("the temperature log was already imported".into());
                }
                let count = import::import(&store, &log, cockpit::util::now_secs()).map_err(|error| error.to_string())?;
                store.set_state("imported_temps_log", &path.display().to_string()).map_err(|error| error.to_string())?;
                println!("imported {count} samples from {}", path.display());
                Ok(())
            })
        }
        _ => Err("usage: vitals [collect | status | import [log]]".into()),
    };
    if let Err(error) = result {
        eprintln!("vitals: {error}");
        std::process::exit(1);
    }
}
