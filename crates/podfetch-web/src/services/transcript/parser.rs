//! Parsers for the Podcasting-2.0 transcript formats (JSON, WebVTT, SRT, HTML).
//!
//! Each format is parsed by hand (no subtitle-parsing crate dependency, per the
//! project's declared plan deviation) into a flat list of [`TranscriptSegment`]s.
//! Parsing never panics: any malformed input yields a [`TranscriptParseError`].

use podfetch_domain::podcast_episode_transcript::TranscriptSegment;
use regex::Regex;
use serde::Deserialize;
use std::sync::OnceLock;

/// The transcript formats defined by the Podcasting 2.0 namespace `<podcast:transcript>` tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptFormat {
    Json,
    Vtt,
    Srt,
    Html,
}

/// Parsing failed; the caller is expected to record this as a failed transcript status.
#[derive(Debug, thiserror::Error)]
pub enum TranscriptParseError {
    #[error("transcript contains no parsable segments")]
    Empty,
    #[error("invalid json transcript: {0}")]
    Json(#[from] serde_json::Error),
}

impl TranscriptFormat {
    /// Maps a mime type (primary) or, as a fallback, a URL's file extension to a format.
    /// Returns `None` when neither is recognized.
    pub fn detect(mime_type: &str, url: Option<&str>) -> Option<Self> {
        if let Some(format) = Self::from_mime(mime_type) {
            return Some(format);
        }
        url.and_then(Self::from_extension)
    }

    /// Preference rank when several transcript formats are offered for the same episode:
    /// smaller is better. Json=0 < Vtt=1 < Srt=2 < Html=3.
    pub fn preference_rank(&self) -> u8 {
        match self {
            TranscriptFormat::Json => 0,
            TranscriptFormat::Vtt => 1,
            TranscriptFormat::Srt => 2,
            TranscriptFormat::Html => 3,
        }
    }

    fn from_mime(mime_type: &str) -> Option<Self> {
        let mime = mime_type
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        match mime.as_str() {
            "application/json" | "text/json" => Some(TranscriptFormat::Json),
            "text/vtt" | "application/vtt" => Some(TranscriptFormat::Vtt),
            "application/srt" | "text/srt" | "application/x-subrip" | "text/x-srt" => {
                Some(TranscriptFormat::Srt)
            }
            "text/html" | "application/xhtml+xml" => Some(TranscriptFormat::Html),
            _ => None,
        }
    }

    fn from_extension(url: &str) -> Option<Self> {
        let path = url.split(['?', '#']).next().unwrap_or(url);
        let ext = path.rsplit('.').next()?.to_ascii_lowercase();
        match ext.as_str() {
            "json" => Some(TranscriptFormat::Json),
            "vtt" => Some(TranscriptFormat::Vtt),
            "srt" => Some(TranscriptFormat::Srt),
            "html" | "htm" => Some(TranscriptFormat::Html),
            _ => None,
        }
    }
}

/// Parses raw transcript bytes into segments. An error means the input was not
/// parsable as the given format (never a panic); the caller sets the transcript's
/// status accordingly.
pub fn parse(format: TranscriptFormat, raw: &[u8]) -> Result<Vec<TranscriptSegment>, TranscriptParseError> {
    let segments = match format {
        TranscriptFormat::Json => parse_json(raw)?,
        TranscriptFormat::Vtt => parse_vtt(raw),
        TranscriptFormat::Srt => parse_srt(raw),
        TranscriptFormat::Html => parse_html(raw),
    };
    if segments.is_empty() {
        return Err(TranscriptParseError::Empty);
    }
    Ok(segments)
}

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct JsonTranscript {
    #[serde(default)]
    segments: Vec<JsonSegment>,
}

#[derive(Debug, Deserialize)]
struct JsonSegment {
    #[serde(default)]
    speaker: Option<String>,
    #[serde(rename = "startTime", default)]
    start_time: Option<f64>,
    #[serde(rename = "endTime", default)]
    end_time: Option<f64>,
    #[serde(default)]
    body: String,
}

fn parse_json(raw: &[u8]) -> Result<Vec<TranscriptSegment>, TranscriptParseError> {
    let doc: JsonTranscript = serde_json::from_slice(raw)?;
    Ok(doc
        .segments
        .into_iter()
        .enumerate()
        .map(|(idx, seg)| TranscriptSegment {
            idx: idx as i32,
            start_ms: seg.start_time.map(seconds_to_ms),
            end_ms: seg.end_time.map(seconds_to_ms),
            speaker: seg.speaker.filter(|s| !s.trim().is_empty()),
            text: seg.body,
        })
        .collect())
}

fn seconds_to_ms(seconds: f64) -> i32 {
    let ms = (seconds * 1000.0).round();
    if ms.is_finite() {
        ms.clamp(i32::MIN as f64, i32::MAX as f64) as i32
    } else {
        0
    }
}

// ---------------------------------------------------------------------------
// Timestamp helpers shared by VTT and SRT
// ---------------------------------------------------------------------------

fn timestamp_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(?:(\d+):)?(\d{2}):(\d{2})[.,](\d{3})$").unwrap())
}

/// Parses `HH:MM:SS.mmm`/`HH:MM:SS,mmm` or the hour-less `MM:SS.mmm` form into milliseconds.
fn parse_timestamp(s: &str) -> Option<i32> {
    let caps = timestamp_regex().captures(s.trim())?;
    let hours: i64 = match caps.get(1) {
        Some(m) => m.as_str().parse().ok()?,
        None => 0,
    };
    let minutes: i64 = caps[2].parse().ok()?;
    let seconds: i64 = caps[3].parse().ok()?;
    let millis: i64 = caps[4].parse().ok()?;

    // Use checked arithmetic to prevent overflow panic on adversarial input.
    let total_ms = hours
        .checked_mul(60)?
        .checked_add(minutes)?
        .checked_mul(60)?
        .checked_add(seconds)?
        .checked_mul(1000)?
        .checked_add(millis)?;

    i32::try_from(total_ms).ok()
}

/// Parses a `start --> end[ cue-settings]` line into (start_ms, end_ms).
fn parse_time_range(line: &str) -> Option<(i32, i32)> {
    let mut parts = line.splitn(2, "-->");
    let start = parts.next()?.trim();
    let rest = parts.next()?.trim();
    let end = rest.split_whitespace().next()?;
    let start_ms = parse_timestamp(start)?;
    let end_ms = parse_timestamp(end)?;
    Some((start_ms, end_ms))
}

fn normalize_line_endings(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).replace("\r\n", "\n").replace('\r', "\n")
}

// ---------------------------------------------------------------------------
// VTT
// ---------------------------------------------------------------------------

fn voice_tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^<v\s+([^>]+)>").unwrap())
}

/// Extracts a `<v Speaker>` voice tag prefix and strips it (and any closing `</v>`) from the text.
fn extract_voice(text: &str) -> (Option<String>, String) {
    if let Some(caps) = voice_tag_regex().captures(text) {
        let speaker = caps[1].trim().to_string();
        let stripped = voice_tag_regex().replace(text, "").replace("</v>", "");
        (Some(speaker), stripped)
    } else {
        (None, text.to_string())
    }
}

fn parse_vtt(raw: &[u8]) -> Vec<TranscriptSegment> {
    let normalized = normalize_line_endings(raw);
    let mut segments = Vec::new();

    for block in normalized.split("\n\n") {
        let block = block.trim();
        if block.is_empty() || block.starts_with("WEBVTT") || block.starts_with("NOTE") || block.starts_with("STYLE")
        {
            continue;
        }

        let mut lines = block.lines();
        let mut timing_line = None;
        for line in lines.by_ref() {
            if line.contains("-->") {
                timing_line = Some(line);
                break;
            }
        }
        let Some(timing_line) = timing_line else {
            continue;
        };
        let Some((start_ms, end_ms)) = parse_time_range(timing_line) else {
            continue;
        };

        let text_lines: Vec<&str> = lines.collect();
        let raw_text = text_lines.join(" ");
        let (speaker, text) = extract_voice(raw_text.trim());
        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }

        segments.push(TranscriptSegment {
            idx: segments.len() as i32,
            start_ms: Some(start_ms),
            end_ms: Some(end_ms),
            speaker,
            text,
        });
    }

    segments
}

// ---------------------------------------------------------------------------
// SRT
// ---------------------------------------------------------------------------

fn parse_srt(raw: &[u8]) -> Vec<TranscriptSegment> {
    let normalized = normalize_line_endings(raw);
    let mut segments = Vec::new();

    for block in normalized.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let mut lines = block.lines();
        let first = match lines.next() {
            Some(l) => l,
            None => continue,
        };
        // The cue index line (a plain number) is optional; skip it if present.
        let timing_line = if first.contains("-->") {
            first
        } else {
            match lines.next() {
                Some(l) => l,
                None => continue,
            }
        };
        let Some((start_ms, end_ms)) = parse_time_range(timing_line) else {
            continue;
        };

        let text_lines: Vec<&str> = lines.collect();
        let text = text_lines.join(" ").trim().to_string();
        if text.is_empty() {
            continue;
        }

        segments.push(TranscriptSegment {
            idx: segments.len() as i32,
            start_ms: Some(start_ms),
            end_ms: Some(end_ms),
            speaker: None,
            text,
        });
    }

    segments
}

// ---------------------------------------------------------------------------
// HTML
// ---------------------------------------------------------------------------

fn p_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<p[^>]*>(.*?)</p>").unwrap())
}

fn cite_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<cite[^>]*>(.*?)</cite>").unwrap())
}

fn time_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<time[^>]*>(.*?)</time>").unwrap())
}

fn tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<[^>]+>").unwrap())
}

/// Parses `H:MM:SS` or `M:SS` (as used inside `<time>`) into milliseconds.
fn parse_html_time(s: &str) -> Option<i32> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    let (hours, minutes, seconds): (i64, i64, i64) = match parts.as_slice() {
        [h, m, s] => (h.parse().ok()?, m.parse().ok()?, s.parse().ok()?),
        [m, s] => (0, m.parse().ok()?, s.parse().ok()?),
        _ => return None,
    };

    // Use checked arithmetic to prevent overflow panic on adversarial input.
    let total_ms = hours
        .checked_mul(60)?
        .checked_add(minutes)?
        .checked_mul(60)?
        .checked_add(seconds)?
        .checked_mul(1000)?;

    i32::try_from(total_ms).ok()
}

fn decode_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn parse_html(raw: &[u8]) -> Vec<TranscriptSegment> {
    let text = String::from_utf8_lossy(raw);
    let mut segments = Vec::new();

    for caps in p_regex().captures_iter(&text) {
        let inner = &caps[1];

        let speaker = cite_regex().captures(inner).and_then(|c| {
            let raw_speaker = decode_entities(c[1].trim().trim_end_matches(':').trim());
            if raw_speaker.is_empty() {
                None
            } else {
                Some(raw_speaker)
            }
        });

        let start_ms = time_regex()
            .captures(inner)
            .and_then(|c| parse_html_time(c[1].trim()));

        let mut remainder = cite_regex().replace(inner, "").to_string();
        remainder = time_regex().replace(&remainder, "").to_string();
        remainder = tag_regex().replace_all(&remainder, "").to_string();
        let text_val = decode_entities(remainder.trim());
        if text_val.is_empty() {
            continue;
        }

        segments.push(TranscriptSegment {
            idx: segments.len() as i32,
            start_ms,
            end_ms: None,
            speaker,
            text: text_val,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- detect() -----------------------------------------------------------

    #[test]
    fn detect_matches_known_mime_types() {
        assert_eq!(TranscriptFormat::detect("text/vtt", None), Some(TranscriptFormat::Vtt));
        assert_eq!(
            TranscriptFormat::detect("application/json", None),
            Some(TranscriptFormat::Json)
        );
        assert_eq!(
            TranscriptFormat::detect("application/srt", None),
            Some(TranscriptFormat::Srt)
        );
        assert_eq!(TranscriptFormat::detect("text/html", None), Some(TranscriptFormat::Html));
    }

    #[test]
    fn detect_falls_back_to_url_extension_when_mime_is_generic() {
        assert_eq!(
            TranscriptFormat::detect("text/plain", Some("https://x/y.srt")),
            Some(TranscriptFormat::Srt)
        );
    }

    #[test]
    fn detect_returns_none_for_unsupported_mime_and_no_url() {
        assert_eq!(TranscriptFormat::detect("application/pdf", None), None);
    }

    #[test]
    fn detect_returns_none_when_neither_mime_nor_extension_match() {
        assert_eq!(
            TranscriptFormat::detect("application/pdf", Some("https://x/y.pdf")),
            None
        );
    }

    // -- preference_rank() ---------------------------------------------------

    #[test]
    fn preference_rank_orders_json_lowest_and_html_highest() {
        assert!(TranscriptFormat::Json.preference_rank() < TranscriptFormat::Vtt.preference_rank());
        assert!(TranscriptFormat::Vtt.preference_rank() < TranscriptFormat::Srt.preference_rank());
        assert!(TranscriptFormat::Srt.preference_rank() < TranscriptFormat::Html.preference_rank());
    }

    // -- JSON ------------------------------------------------------------------

    #[test]
    fn parses_json_segments() {
        let raw = include_bytes!("fixtures/sample.json");
        let segments = parse(TranscriptFormat::Json, raw).expect("sample.json should parse");

        assert_eq!(segments.len(), 2);

        assert_eq!(segments[0].idx, 0);
        assert_eq!(segments[0].start_ms, Some(500));
        assert_eq!(segments[0].end_ms, Some(4200));
        assert_eq!(segments[0].speaker, Some("Alice".to_string()));
        assert_eq!(segments[0].text, "Hello world");

        assert_eq!(segments[1].idx, 1);
        assert_eq!(segments[1].start_ms, Some(4200));
        assert_eq!(segments[1].end_ms, Some(8000));
        assert_eq!(segments[1].speaker, Some("Bob".to_string()));
        assert_eq!(segments[1].text, "Nice to meet you");
    }

    #[test]
    fn empty_json_returns_empty_error() {
        let raw = include_bytes!("fixtures/empty.json");
        let result = parse(TranscriptFormat::Json, raw);
        assert!(matches!(result, Err(TranscriptParseError::Empty)));
    }

    #[test]
    fn malformed_json_returns_json_error() {
        let raw = b"not json at all {";
        let result = parse(TranscriptFormat::Json, raw);
        assert!(matches!(result, Err(TranscriptParseError::Json(_))));
    }

    // -- VTT -------------------------------------------------------------------

    #[test]
    fn parses_vtt_segments_with_voice_tags_and_multiline_text() {
        let raw = include_bytes!("fixtures/sample.vtt");
        let segments = parse(TranscriptFormat::Vtt, raw).expect("sample.vtt should parse");

        assert_eq!(segments.len(), 2);

        assert_eq!(segments[0].idx, 0);
        assert_eq!(segments[0].start_ms, Some(500));
        assert_eq!(segments[0].end_ms, Some(4200));
        assert_eq!(segments[0].speaker, Some("Alice".to_string()));
        assert_eq!(segments[0].text, "Hello world");

        assert_eq!(segments[1].idx, 1);
        assert_eq!(segments[1].start_ms, Some(4200));
        assert_eq!(segments[1].end_ms, Some(8000));
        assert_eq!(segments[1].speaker, Some("Bob".to_string()));
        assert_eq!(segments[1].text, "Nice to meet you that continues on a second line");
    }

    #[test]
    fn vtt_time_without_hours_is_supported() {
        let raw = b"WEBVTT\n\n01:02.500 --> 01:05.000\nShort form time\n";
        let segments = parse(TranscriptFormat::Vtt, raw).expect("should parse");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start_ms, Some(62500));
        assert_eq!(segments[0].end_ms, Some(65000));
    }

    #[test]
    fn vtt_handles_crlf_line_endings() {
        let raw = b"WEBVTT\r\n\r\n00:00:01.000 --> 00:00:02.000\r\nHello\r\n";
        let segments = parse(TranscriptFormat::Vtt, raw).expect("should parse");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start_ms, Some(1000));
        assert_eq!(segments[0].end_ms, Some(2000));
        assert_eq!(segments[0].text, "Hello");
    }

    #[test]
    fn broken_vtt_returns_error() {
        let raw = include_bytes!("fixtures/broken.vtt");
        let result = parse(TranscriptFormat::Vtt, raw);
        assert!(result.is_err());
    }

    // -- SRT -------------------------------------------------------------------

    #[test]
    fn parses_srt_segments() {
        let raw = include_bytes!("fixtures/sample.srt");
        let segments = parse(TranscriptFormat::Srt, raw).expect("sample.srt should parse");

        assert_eq!(segments.len(), 2);

        assert_eq!(segments[0].idx, 0);
        assert_eq!(segments[0].start_ms, Some(500));
        assert_eq!(segments[0].end_ms, Some(4200));
        assert_eq!(segments[0].speaker, None);
        assert_eq!(segments[0].text, "Hello world");

        assert_eq!(segments[1].idx, 1);
        assert_eq!(segments[1].start_ms, Some(4200));
        assert_eq!(segments[1].end_ms, Some(8000));
        assert_eq!(segments[1].text, "Nice to meet you");
    }

    #[test]
    fn srt_handles_crlf_line_endings() {
        let raw = b"1\r\n00:00:01,000 --> 00:00:02,000\r\nHello\r\n";
        let segments = parse(TranscriptFormat::Srt, raw).expect("should parse");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start_ms, Some(1000));
        assert_eq!(segments[0].text, "Hello");
    }

    // -- HTML ------------------------------------------------------------------

    #[test]
    fn parses_html_segments_decodes_entities_and_handles_missing_time() {
        let raw = include_bytes!("fixtures/sample.html");
        let segments = parse(TranscriptFormat::Html, raw).expect("sample.html should parse");

        assert_eq!(segments.len(), 3);

        assert_eq!(segments[0].idx, 0);
        assert_eq!(segments[0].start_ms, Some(0));
        assert_eq!(segments[0].speaker, Some("Alice".to_string()));
        assert_eq!(segments[0].text, "Hello world");

        assert_eq!(segments[1].idx, 1);
        assert_eq!(segments[1].start_ms, Some(4000));
        assert_eq!(segments[1].speaker, Some("Bob".to_string()));
        assert_eq!(segments[1].text, "Nice to meet you & you too");

        assert_eq!(segments[2].idx, 2);
        assert_eq!(segments[2].start_ms, None);
        assert_eq!(segments[2].speaker, None);
        assert_eq!(segments[2].text, "No time here, just narration.");
    }

    // -- Overflow safety (checked arithmetic for adversarial timestamps) --------

    #[test]
    fn vtt_with_overflow_hour_value_skips_cue_and_returns_empty_error() {
        // 18-digit hour value would overflow i64 arithmetic without checked ops.
        // The malformed cue should be skipped; if no valid cues remain, parse returns Empty.
        let raw = b"WEBVTT\n\n200000000000000000:00:00.000 --> 00:00:04.200\nShould skip\n";
        let result = parse(TranscriptFormat::Vtt, raw);
        // Cue is skipped due to malformed timestamp; no other valid cues, so Empty error.
        assert!(matches!(result, Err(TranscriptParseError::Empty)));
    }

    #[test]
    fn html_with_overflow_hour_value_sets_start_ms_none_no_panic() {
        // <time>99999999999:00:00</time> would overflow i64 arithmetic.
        // With checked ops, parse_html_time returns None, and segment gets start_ms: None.
        let raw = b"<p><time>99999999999:00:00</time> Some text</p>";
        let segments = parse(TranscriptFormat::Html, raw).expect("should parse without panic");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start_ms, None, "overflow should yield None, not panic");
        assert_eq!(segments[0].text, "Some text");
    }

    #[test]
    fn srt_timestamp_just_above_i32_max_ms_skips_cue_as_malformed() {
        // 600 hours = 2160000 seconds = 2,160,000,000 milliseconds (> i32::MAX = 2,147,483,647).
        // After checked arithmetic succeeds but i32::try_from fails, cue is skipped.
        let raw = b"1\n600:00:00,000 --> 00:00:05,000\nText\n";
        let result = parse(TranscriptFormat::Srt, raw);
        // Cue is skipped (timestamp out of i32 range); no valid cues remain, so Empty.
        assert!(matches!(result, Err(TranscriptParseError::Empty)));
    }

    // -- error propagation across formats ---------------------------------------

    #[test]
    fn empty_input_for_vtt_srt_html_is_an_error() {
        assert!(matches!(
            parse(TranscriptFormat::Vtt, b"WEBVTT\n"),
            Err(TranscriptParseError::Empty)
        ));
        assert!(matches!(parse(TranscriptFormat::Srt, b""), Err(TranscriptParseError::Empty)));
        assert!(matches!(
            parse(TranscriptFormat::Html, b"<p></p>"),
            Err(TranscriptParseError::Empty)
        ));
    }
}
