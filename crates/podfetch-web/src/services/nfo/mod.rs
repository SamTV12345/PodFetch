use std::str::FromStr;

pub mod builders;
pub mod service;

/// Which NFO layout to emit for a podcast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NfoFormat {
    #[default]
    Off,
    Tvshow,
    Album,
}

impl FromStr for NfoFormat {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "tvshow" => Ok(NfoFormat::Tvshow),
            "album" => Ok(NfoFormat::Album),
            // "off", "", and any unknown value disable NFO generation.
            _ => Ok(NfoFormat::Off),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_and_unknown_values() {
        assert_eq!(NfoFormat::from_str("tvshow"), Ok(NfoFormat::Tvshow));
        assert_eq!(NfoFormat::from_str("album"), Ok(NfoFormat::Album));
        assert_eq!(NfoFormat::from_str("off"), Ok(NfoFormat::Off));
        assert_eq!(NfoFormat::from_str(""), Ok(NfoFormat::Off));
        assert_eq!(NfoFormat::from_str("garbage"), Ok(NfoFormat::Off));
        assert_eq!(NfoFormat::default(), NfoFormat::Off);
    }
}
