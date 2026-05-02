use podfetch_domain::sponsorblock::{SponsorBlockCategory, SponsorBlockSegment};
use reqwest::StatusCode;
use serde::Deserialize;
use sha256::digest;
use std::str::FromStr;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://sponsor.ajay.app";
const ENV_BASE_URL: &str = "SPONSORBLOCK_BASE_URL";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
pub enum SponsorBlockError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("not found")]
    NotFound,
    #[error("rate limited")]
    RateLimited,
    #[error("unexpected status: {0}")]
    UnexpectedStatus(StatusCode),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Deserialize)]
struct ApiVideoEntry {
    #[serde(rename = "videoID")]
    video_id: String,
    segments: Vec<ApiSegment>,
}

#[derive(Debug, Deserialize)]
struct ApiSegment {
    category: String,
    segment: [f64; 2],
    #[serde(rename = "UUID")]
    uuid: String,
}

pub struct SponsorBlockClient {
    base_url: String,
    http: reqwest::blocking::Client,
}

impl SponsorBlockClient {
    pub fn new() -> Self {
        let base_url = std::env::var(ENV_BASE_URL)
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();
        let http = reqwest::blocking::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("reqwest blocking client");
        Self { base_url, http }
    }

    /// Fetch SponsorBlock segments for a single YouTube video using the
    /// privacy-friendly hash-prefix endpoint. Returns only segments belonging
    /// to the requested video (the API may return multiple matches sharing the
    /// same hash prefix).
    pub fn fetch_segments(
        &self,
        video_id: &str,
        categories: &[SponsorBlockCategory],
    ) -> Result<Vec<SponsorBlockSegment>, SponsorBlockError> {
        let hash = digest(video_id);
        let prefix = &hash[..4];

        let categories_param = serde_json::to_string(
            &categories
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>(),
        )
        .map_err(|e| SponsorBlockError::InvalidResponse(e.to_string()))?;

        let url = format!("{}/api/skipSegments/{}", self.base_url, prefix);
        let response = self
            .http
            .get(&url)
            .query(&[("categories", categories_param)])
            .send()?;

        match response.status() {
            StatusCode::OK => {}
            StatusCode::NOT_FOUND => return Err(SponsorBlockError::NotFound),
            StatusCode::TOO_MANY_REQUESTS => return Err(SponsorBlockError::RateLimited),
            other => return Err(SponsorBlockError::UnexpectedStatus(other)),
        }

        let body: Vec<ApiVideoEntry> = response
            .json()
            .map_err(|e| SponsorBlockError::InvalidResponse(e.to_string()))?;

        let segments = body
            .into_iter()
            .filter(|entry| entry.video_id == video_id)
            .flat_map(|entry| entry.segments)
            .filter_map(|s| {
                let category = SponsorBlockCategory::from_str(&s.category).ok()?;
                Some(SponsorBlockSegment {
                    category,
                    start_seconds: s.segment[0],
                    end_seconds: s.segment[1],
                    uuid: s.uuid,
                })
            })
            .collect();

        Ok(segments)
    }
}

impl Default for SponsorBlockClient {
    fn default() -> Self {
        Self::new()
    }
}
