//! Extracts a YouTube video ID from an RSS item. SponsorBlock data is keyed by
//! YouTube video ID, so this gates the whole feature: no ID -> not a YouTube
//! episode -> no SponsorBlock lookup.

/// Extract an 11-character YouTube video ID from the most reliable source on the
/// item, in priority order: `<link>`, then `<guid>`, then the enclosure URL.
pub fn extract_youtube_video_id(
    link: Option<&str>,
    guid: Option<&str>,
    enclosure_url: Option<&str>,
) -> Option<String> {
    for candidate in [link, guid, enclosure_url].into_iter().flatten() {
        if let Some(id) = parse_video_id(candidate) {
            return Some(id);
        }
    }
    None
}

/// Parse a single string for a YouTube video ID. Handles watch URLs, youtu.be,
/// /embed/, music.youtube.com, the `yt:video:ID` guid form, and a bare ID.
fn parse_video_id(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // yt:video:VIDEOID  (YouTube channel/uploads feed guid form)
    if let Some(rest) = trimmed.strip_prefix("yt:video:") {
        return valid_id(rest);
    }

    // URL forms.
    if let Ok(url) = url::Url::parse(trimmed) {
        let host = url.host_str().unwrap_or("").trim_start_matches("www.");
        match host {
            "youtu.be" => {
                if let Some(seg) = url.path_segments().and_then(|mut s| s.next()) {
                    return valid_id(seg);
                }
            }
            "youtube.com" | "m.youtube.com" | "music.youtube.com" => {
                // /watch?v=ID
                if let Some((_, v)) = url.query_pairs().find(|(k, _)| k == "v") {
                    return valid_id(&v);
                }
                // /embed/ID  or  /shorts/ID  or  /v/ID
                let segs: Vec<&str> = url.path_segments().map(|s| s.collect()).unwrap_or_default();
                if let [kind, id, ..] = segs.as_slice()
                    && matches!(*kind, "embed" | "shorts" | "v")
                {
                    return valid_id(id);
                }
            }
            _ => {}
        }
        return None;
    }

    // Bare 11-char id (last resort).
    valid_id(trimmed)
}

/// A YouTube video ID is exactly 11 chars of [A-Za-z0-9_-].
fn valid_id(candidate: &str) -> Option<String> {
    let id = candidate.trim();
    if id.len() == 11 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
        Some(id.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_url_in_link() {
        assert_eq!(
            extract_youtube_video_id(Some("https://www.youtube.com/watch?v=dQw4w9WgXcQ"), None, None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn youtu_be_short_url() {
        assert_eq!(
            extract_youtube_video_id(Some("https://youtu.be/dQw4w9WgXcQ?t=10"), None, None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn embed_url() {
        assert_eq!(
            extract_youtube_video_id(Some("https://www.youtube.com/embed/dQw4w9WgXcQ"), None, None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn music_youtube_url() {
        assert_eq!(
            extract_youtube_video_id(Some("https://music.youtube.com/watch?v=dQw4w9WgXcQ"), None, None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn yt_video_guid_form() {
        assert_eq!(
            extract_youtube_video_id(None, Some("yt:video:dQw4w9WgXcQ"), None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn falls_back_to_enclosure_when_link_and_guid_have_no_id() {
        assert_eq!(
            extract_youtube_video_id(
                Some("https://example.com/post/123"),
                Some("urn:uuid:abc"),
                Some("https://youtu.be/dQw4w9WgXcQ")
            ),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn non_youtube_returns_none() {
        assert_eq!(
            extract_youtube_video_id(
                Some("https://anchor.fm/s/123/podcast/play/456/episode.mp3"),
                Some("9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"),
                Some("https://anchor.fm/s/123/audio.mp3")
            ),
            None
        );
    }

    #[test]
    fn malformed_id_returns_none() {
        // Too short.
        assert_eq!(extract_youtube_video_id(Some("https://youtu.be/abc"), None, None), None);
        // Wrong characters.
        assert_eq!(extract_youtube_video_id(None, Some("yt:video:dQw4w9WgX!Q"), None), None);
    }
}
