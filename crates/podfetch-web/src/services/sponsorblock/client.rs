//! Thin SponsorBlock API client built on the project's existing async reqwest
//! client. Uses the privacy-preserving hash-prefix endpoint so the exact video
//! ID never leaves this server.

use common_infrastructure::error::{map_reqwest_error, CustomError};
use common_infrastructure::http::{get_http_client, COMMON_USER_AGENT};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use reqwest::header::USER_AGENT;
use serde::Deserialize;

/// Categories PodFetch knows how to skip. Anything else returned by the API is
/// ignored at parse time.
pub const SUPPORTED_CATEGORIES: [&str; 8] = [
    "sponsor",
    "selfpromo",
    "interaction",
    "intro",
    "outro",
    "preview",
    "filler",
    "music_offtopic",
];

/// A single segment as PodFetch stores/uses it (milliseconds).
#[derive(Debug, Clone, PartialEq)]
pub struct FetchedSegment {
    pub uuid: String,
    pub category: String,
    pub action_type: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub votes: i32,
    pub locked: bool,
    /// The video duration SponsorBlock recorded (seconds); 0.0 if unknown.
    pub video_duration_secs: f64,
}

// ---- Raw API shapes (hash-prefix endpoint) ----

#[derive(Debug, Deserialize)]
struct RawVideo {
    #[serde(rename = "videoID")]
    video_id: String,
    segments: Vec<RawSegment>,
}

#[derive(Debug, Deserialize)]
struct RawSegment {
    #[serde(rename = "UUID")]
    uuid: String,
    category: String,
    #[serde(rename = "actionType")]
    action_type: String,
    /// [start, end] in floating-point seconds.
    segment: [f64; 2],
    #[serde(default)]
    votes: i32,
    #[serde(default)]
    locked: i32,
    #[serde(rename = "videoDuration", default)]
    video_duration: f64,
}

/// Parse the hash-prefix endpoint response, keeping only segments for `video_id`
/// whose category is supported. Pure function — unit tested without network.
pub fn parse_hash_response(body: &str, video_id: &str) -> Result<Vec<FetchedSegment>, CustomError> {
    let videos: Vec<RawVideo> = serde_json::from_str(body).map_err(|e| {
        common_infrastructure::error::CustomErrorInner::Conflict(
            format!("Failed to parse SponsorBlock response: {e}"),
            common_infrastructure::error::ErrorSeverity::Warning,
        )
    })?;

    let mut out = Vec::new();
    for video in videos.into_iter().filter(|v| v.video_id == video_id) {
        for seg in video.segments {
            if !SUPPORTED_CATEGORIES.contains(&seg.category.as_str()) {
                continue;
            }
            if seg.action_type != "skip" {
                continue;
            }
            let start_ms = (seg.segment[0] * 1000.0).round() as i64;
            let end_ms = (seg.segment[1] * 1000.0).round() as i64;
            if end_ms <= start_ms {
                continue; // drop zero/negative-length segments
            }
            out.push(FetchedSegment {
                uuid: seg.uuid,
                category: seg.category,
                action_type: seg.action_type,
                start_ms,
                end_ms,
                votes: seg.votes,
                locked: seg.locked != 0,
                video_duration_secs: seg.video_duration,
            });
        }
    }
    Ok(out)
}

/// Base URL, overridable via `SPONSORBLOCK_API_URL` for self-hosted mirrors.
fn base_url() -> String {
    std::env::var("SPONSORBLOCK_API_URL")
        .unwrap_or_else(|_| "https://sponsor.ajay.app".to_string())
}

/// Query SponsorBlock for one video. Returns an empty Vec on 404 (no data).
pub async fn fetch_segments(video_id: &str) -> Result<Vec<FetchedSegment>, CustomError> {
    let hash = sha256::digest(video_id);
    let prefix = &hash[..4];
    let categories = serde_json::to_string(&SUPPORTED_CATEGORIES).unwrap();
    let url = format!("{}/api/skipSegments/{}", base_url(), prefix);

    let client = get_http_client(&ENVIRONMENT_SERVICE);

    let resp = client
        .get(&url)
        .header(USER_AGENT, COMMON_USER_AGENT)
        .query(&[("categories", categories.as_str()), ("actionTypes", "[\"skip\"]")])
        .send()
        .await
        .map_err(map_reqwest_error)?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(Vec::new());
    }
    if !resp.status().is_success() {
        return Err(common_infrastructure::error::CustomErrorInner::Conflict(
            format!("SponsorBlock returned status {}", resp.status()),
            common_infrastructure::error::ErrorSeverity::Warning,
        )
        .into());
    }

    let body = resp.text().await.map_err(map_reqwest_error)?;
    parse_hash_response(&body, video_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"[
      {"videoID":"dQw4w9WgXcQ","hash":"e0c4...","segments":[
        {"UUID":"seg-1","category":"sponsor","actionType":"skip","segment":[30.5,45.0],"votes":12,"locked":1,"videoDuration":600.0},
        {"UUID":"seg-2","category":"music_offtopic","actionType":"skip","segment":[0.0,0.0],"votes":0,"locked":0,"videoDuration":600.0},
        {"UUID":"seg-3","category":"chapter","actionType":"chapter","segment":[100.0,120.0],"votes":3,"locked":0,"videoDuration":600.0}
      ]},
      {"videoID":"other00video","hash":"e0c4...","segments":[
        {"UUID":"x","category":"sponsor","actionType":"skip","segment":[1.0,2.0],"votes":0,"locked":0,"videoDuration":10.0}
      ]}
    ]"#;

    #[test]
    fn parses_and_filters_to_target_video() {
        let segs = parse_hash_response(SAMPLE, "dQw4w9WgXcQ").unwrap();
        // seg-1 kept; seg-2 dropped (zero length); seg-3 dropped (unsupported
        // category); the "other00video" entry dropped (wrong video).
        assert_eq!(segs.len(), 1);
        let s = &segs[0];
        assert_eq!(s.uuid, "seg-1");
        assert_eq!(s.category, "sponsor");
        assert_eq!(s.start_ms, 30500);
        assert_eq!(s.end_ms, 45000);
        assert_eq!(s.votes, 12);
        assert!(s.locked);
        assert_eq!(s.video_duration_secs, 600.0);
    }

    #[test]
    fn unknown_video_yields_empty() {
        assert!(parse_hash_response(SAMPLE, "no-such-id").unwrap().is_empty());
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(parse_hash_response("not json", "x").is_err());
    }

    #[test]
    fn non_skip_action_type_is_dropped() {
        let body = r#"[
          {"videoID":"vid00000001","hash":"h","segments":[
            {"UUID":"poi","category":"sponsor","actionType":"poi","segment":[5.0,5.0],"votes":1,"locked":0,"videoDuration":100.0},
            {"UUID":"mute","category":"sponsor","actionType":"mute","segment":[10.0,20.0],"votes":1,"locked":0,"videoDuration":100.0},
            {"UUID":"keep","category":"sponsor","actionType":"skip","segment":[30.0,40.0],"votes":1,"locked":0,"videoDuration":100.0}
          ]}
        ]"#;
        let segs = parse_hash_response(body, "vid00000001").unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].uuid, "keep");
    }
}
