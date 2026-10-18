//! What the now-playing card shows of a player, from `playerctl --follow metadata` lines.

/// The fields asked of playerctl, tab-separated, in `Track::parse` order.
pub const FIELDS: &str = "{{status}}\t{{artist}}\t{{title}}\t{{album}}\t{{mpris:artUrl}}\t{{mpris:length}}";
/// Firefox reports a 1 s length and no position for web players (Apple Music): a length this short
/// is not real, and the progress row hides.
const SHORTEST_REAL_LENGTH_US: u64 = 2_000_000;

#[derive(Debug, Default, PartialEq)]
pub struct Track {
    pub is_playing: bool,
    pub artist: String,
    pub title: String,
    pub album: String,
    pub art_url: String,
    /// Microseconds; None when the player gives no real length.
    pub length: Option<u64>,
}

impl Track {
    pub fn parse(line: &str) -> Track {
        let mut fields = line.split('\t').map(str::to_string);
        let mut next = || fields.next().unwrap_or_default();
        let (status, artist, title, album, art_url, length) = (next(), next(), next(), next(), next(), next());
        Track {
            is_playing: status == "Playing",
            artist,
            title,
            album,
            art_url,
            length: length.parse::<u64>().ok().filter(|length| *length > SHORTEST_REAL_LENGTH_US),
        }
    }
}

/// m:ss
pub fn format_time(microseconds: f64) -> String {
    let seconds = (microseconds / 1_000_000.0).max(0.0) as u64;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_line_gives_every_field() {
        let track = Track::parse("Playing\tDaft Punk\tVeridis Quo\tDiscovery\tfile:///tmp/cover.jpg\t345000000");
        assert_eq!(track, Track { is_playing: true, artist: "Daft Punk".into(), title: "Veridis Quo".into(), album: "Discovery".into(),
                                  art_url: "file:///tmp/cover.jpg".into(), length: Some(345_000_000) });
    }

    #[test]
    fn a_one_second_length_is_no_length() {
        assert_eq!(Track::parse("Playing\ta\tb\tc\td\t1000000").length, None);
    }

    #[test]
    fn a_short_line_leaves_the_rest_empty() {
        let track = Track::parse("Paused\tartist");
        assert_eq!((track.is_playing, track.title.as_str(), track.length), (false, "", None));
    }

    #[test]
    fn times_show_minutes_and_padded_seconds() {
        assert_eq!(format_time(65_400_000.0), "1:05");
        assert_eq!(format_time(0.0), "0:00");
    }
}
