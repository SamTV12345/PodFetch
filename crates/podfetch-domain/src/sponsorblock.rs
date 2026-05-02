use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SponsorBlockCategory {
    Sponsor,
    Selfpromo,
    Interaction,
    Intro,
    Outro,
    Preview,
    MusicOfftopic,
    Filler,
}

impl SponsorBlockCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SponsorBlockCategory::Sponsor => "sponsor",
            SponsorBlockCategory::Selfpromo => "selfpromo",
            SponsorBlockCategory::Interaction => "interaction",
            SponsorBlockCategory::Intro => "intro",
            SponsorBlockCategory::Outro => "outro",
            SponsorBlockCategory::Preview => "preview",
            SponsorBlockCategory::MusicOfftopic => "music_offtopic",
            SponsorBlockCategory::Filler => "filler",
        }
    }

    pub fn all() -> [Self; 8] {
        [
            SponsorBlockCategory::Sponsor,
            SponsorBlockCategory::Selfpromo,
            SponsorBlockCategory::Interaction,
            SponsorBlockCategory::Intro,
            SponsorBlockCategory::Outro,
            SponsorBlockCategory::Preview,
            SponsorBlockCategory::MusicOfftopic,
            SponsorBlockCategory::Filler,
        ]
    }
}

impl FromStr for SponsorBlockCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sponsor" => Ok(SponsorBlockCategory::Sponsor),
            "selfpromo" => Ok(SponsorBlockCategory::Selfpromo),
            "interaction" => Ok(SponsorBlockCategory::Interaction),
            "intro" => Ok(SponsorBlockCategory::Intro),
            "outro" => Ok(SponsorBlockCategory::Outro),
            "preview" => Ok(SponsorBlockCategory::Preview),
            "music_offtopic" => Ok(SponsorBlockCategory::MusicOfftopic),
            "filler" => Ok(SponsorBlockCategory::Filler),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SponsorBlockSegment {
    pub category: SponsorBlockCategory,
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub uuid: String,
}

/// Encode a list of categories as a comma-separated string for DB storage.
pub fn categories_to_csv(categories: &[SponsorBlockCategory]) -> String {
    categories
        .iter()
        .map(|c| c.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

/// Decode a CSV-encoded category list. Unknown entries are silently skipped.
pub fn categories_from_csv(s: &str) -> Vec<SponsorBlockCategory> {
    s.split(',')
        .filter_map(|piece| {
            let trimmed = piece.trim();
            if trimmed.is_empty() {
                None
            } else {
                SponsorBlockCategory::from_str(trimmed).ok()
            }
        })
        .collect()
}

/// Default skippable categories on first install.
pub fn default_categories() -> Vec<SponsorBlockCategory> {
    vec![
        SponsorBlockCategory::Sponsor,
        SponsorBlockCategory::Selfpromo,
        SponsorBlockCategory::Interaction,
    ]
}

/// Extract a YouTube video ID from a URL or Atom-feed GUID. Pure function.
///
/// Recognises:
/// - `https://www.youtube.com/watch?v=VIDEOID(...&...)`
/// - `https://youtu.be/VIDEOID(?si=...)`
/// - `https://www.youtube.com/embed/VIDEOID`
/// - `yt:video:VIDEOID` (Atom-feed GUID format)
pub fn extract_youtube_id(url: &str, guid: Option<&str>) -> Option<String> {
    fn is_valid_id(s: &str) -> bool {
        s.len() == 11
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }

    fn extract_after(url: &str, marker: &str) -> Option<String> {
        let lower = url.to_ascii_lowercase();
        let idx = lower.find(marker)?;
        let after = &url[idx + marker.len()..];
        let candidate = after.split(['?', '#', '/', '&']).next().unwrap_or("");
        if is_valid_id(candidate) {
            Some(candidate.to_string())
        } else {
            None
        }
    }

    fn from_url(url: &str) -> Option<String> {
        let lower = url.to_ascii_lowercase();
        if lower.contains("youtu.be/") {
            if let Some(id) = extract_after(url, "youtu.be/") {
                return Some(id);
            }
        }
        if lower.contains("youtube.com/") || lower.contains("youtube-nocookie.com/") {
            if let Some(id) = extract_after(url, "v=") {
                return Some(id);
            }
            if let Some(id) = extract_after(url, "/embed/") {
                return Some(id);
            }
            if let Some(id) = extract_after(url, "/shorts/") {
                return Some(id);
            }
        }
        None
    }

    fn from_guid(guid: &str) -> Option<String> {
        if let Some(rest) = guid.strip_prefix("yt:video:") {
            if is_valid_id(rest) {
                return Some(rest.to_string());
            }
        }
        None
    }

    from_url(url).or_else(|| guid.and_then(from_guid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_youtube_watch_url() {
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ", None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn extracts_youtu_be_url() {
        assert_eq!(
            extract_youtube_id("https://youtu.be/dQw4w9WgXcQ", None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn extracts_youtu_be_with_query() {
        assert_eq!(
            extract_youtube_id("https://youtu.be/dQw4w9WgXcQ?si=abc", None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn extracts_watch_url_with_extra_params() {
        assert_eq!(
            extract_youtube_id(
                "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLabc&index=2",
                None
            ),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn extracts_embed_url() {
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/embed/dQw4w9WgXcQ", None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn extracts_shorts_url() {
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/shorts/dQw4w9WgXcQ", None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn extracts_guid() {
        assert_eq!(
            extract_youtube_id("https://example.com/feed", Some("yt:video:dQw4w9WgXcQ")),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn rejects_non_youtube_url() {
        assert_eq!(
            extract_youtube_id("https://anchor.fm/example/episodes/foo", None),
            None
        );
    }

    #[test]
    fn rejects_short_id() {
        assert_eq!(
            extract_youtube_id("https://youtu.be/short", None),
            None
        );
    }

    #[test]
    fn rejects_long_id() {
        assert_eq!(
            extract_youtube_id("https://youtu.be/dQw4w9WgXcQ123", None),
            None
        );
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(extract_youtube_id("", None), None);
        assert_eq!(extract_youtube_id("not a url", None), None);
    }

    #[test]
    fn category_roundtrip() {
        for c in SponsorBlockCategory::all() {
            assert_eq!(
                SponsorBlockCategory::from_str(c.as_str()),
                Ok(c),
                "roundtrip failed for {c:?}"
            );
        }
    }

    #[test]
    fn category_rejects_garbage() {
        assert!(SponsorBlockCategory::from_str("nonsense").is_err());
        assert!(SponsorBlockCategory::from_str("").is_err());
        assert!(SponsorBlockCategory::from_str("Sponsor").is_err()); // case-sensitive
    }

    #[test]
    fn csv_roundtrip() {
        let cats = vec![
            SponsorBlockCategory::Sponsor,
            SponsorBlockCategory::Filler,
            SponsorBlockCategory::Outro,
        ];
        let csv = categories_to_csv(&cats);
        assert_eq!(csv, "sponsor,filler,outro");
        assert_eq!(categories_from_csv(&csv), cats);
    }

    #[test]
    fn csv_handles_garbage_and_empty() {
        assert_eq!(categories_from_csv(""), Vec::<SponsorBlockCategory>::new());
        assert_eq!(
            categories_from_csv("sponsor,nonsense, ,outro"),
            vec![
                SponsorBlockCategory::Sponsor,
                SponsorBlockCategory::Outro
            ]
        );
    }
}
