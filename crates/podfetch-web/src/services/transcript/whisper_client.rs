//! OpenAI-compatible Whisper transcription client.
//!
//! Sends a local audio file to `{base_url}/v1/audio/transcriptions` as a
//! multipart upload and turns the `verbose_json` response into the same
//! [`TranscriptSegment`] shape the feed-format parsers produce, so generated
//! transcripts can flow through the exact same downstream code (archiving,
//! preference, search) as downloaded ones.
//!
//! Never panics: HTTP failures, non-2xx responses, and malformed JSON all
//! come back as `Err(CustomError)`.

use common_infrastructure::config::TranscriptionConfig;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error};
use podfetch_domain::podcast_episode_transcript::TranscriptSegment;
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

/// Transcription of a long episode can take a long time; give the server
/// plenty of room rather than timing out a legitimate in-progress job.
const TRANSCRIBE_TIMEOUT: Duration = Duration::from_secs(600);

pub struct WhisperClient {
    config: TranscriptionConfig,
    client: reqwest::blocking::Client,
}

impl WhisperClient {
    pub fn new(config: TranscriptionConfig) -> Self {
        // `Client::builder().build()` only fails on conflicting TLS/proxy
        // configuration, none of which we set here; fall back to the
        // unconfigured default client rather than ever panicking in `new`.
        let client = reqwest::blocking::Client::builder()
            .timeout(TRANSCRIBE_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self { config, client }
    }

    /// POSTs the audio file at `audio_path` to the configured Whisper-compatible
    /// endpoint and returns the parsed segments alongside the detected
    /// language (when the server reports one).
    pub fn transcribe(&self, audio_path: &Path) -> Result<(Vec<TranscriptSegment>, Option<String>), CustomError> {
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", audio_path)
            .map_err(|err| {
                CustomError::from(CustomErrorInner::Conflict(
                    format!("could not read audio file {}: {err}", audio_path.display()),
                    ErrorSeverity::Warning,
                ))
            })?
            .text("model", self.config.model.clone())
            .text("response_format", "verbose_json");

        let url = format!("{}/v1/audio/transcriptions", self.config.base_url);
        let mut request = self.client.post(&url).multipart(form);
        if let Some(api_key) = &self.config.api_key {
            request = request.bearer_auth(api_key);
        }

        let response = request.send().map_err(map_reqwest_error)?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(CustomError::from(CustomErrorInner::Conflict(
                format!("whisper transcription request failed with HTTP status {status}: {body}"),
                ErrorSeverity::Warning,
            )));
        }

        let parsed: WhisperResponse = response.json().map_err(|err| {
            CustomError::from(CustomErrorInner::Conflict(
                format!("invalid whisper transcription response: {err}"),
                ErrorSeverity::Warning,
            ))
        })?;

        let segments = parsed
            .segments
            .into_iter()
            .enumerate()
            .map(|(idx, seg)| TranscriptSegment {
                idx: idx as i32,
                start_ms: Some(seconds_to_ms(seg.start)),
                end_ms: Some(seconds_to_ms(seg.end)),
                // Whisper's verbose_json has no notion of distinct speakers.
                speaker: None,
                text: seg.text,
            })
            .collect();

        Ok((segments, parsed.language))
    }
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    segments: Vec<WhisperSegment>,
}

#[derive(Debug, Deserialize)]
struct WhisperSegment {
    start: f64,
    end: f64,
    #[serde(default)]
    text: String,
}

/// Mirrors `parser::seconds_to_ms` (that helper is private to `parser.rs`):
/// rounds to the nearest millisecond with a checked conversion so an
/// adversarial/huge value can never panic.
fn seconds_to_ms(seconds: f64) -> i32 {
    let ms = (seconds * 1000.0).round();
    if ms.is_finite() {
        ms.clamp(i32::MIN as f64, i32::MAX as f64) as i32
    } else {
        0
    }
}

/// Serializes segments as a WebVTT document, for archiving a generated
/// transcript's segments next to the episode's audio file. Segments without
/// both a start and end timestamp are skipped since a VTT cue requires a
/// `start --> end` timing line. No `<v>` speaker tags are emitted: Whisper's
/// output never carries speaker information, unlike the feed-format parsers.
pub fn segments_to_vtt(segments: &[TranscriptSegment]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for segment in segments {
        let (Some(start_ms), Some(end_ms)) = (segment.start_ms, segment.end_ms) else {
            continue;
        };
        out.push_str(&format_vtt_timestamp(start_ms));
        out.push_str(" --> ");
        out.push_str(&format_vtt_timestamp(end_ms));
        out.push('\n');
        out.push_str(&segment.text);
        out.push_str("\n\n");
    }
    out
}

/// Formats milliseconds as `HH:MM:SS.mmm`. Negative input (which should never
/// occur for real transcription output) is clamped to zero rather than
/// panicking on the `i64` cast.
fn format_vtt_timestamp(ms: i32) -> String {
    let total_ms = ms.max(0) as i64;
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms % 3_600_000) / 60_000;
    let seconds = (total_ms % 60_000) / 1000;
    let millis = total_ms % 1000;
    format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::transcript::parser::{self, TranscriptFormat};
    use axum::Router;
    use axum::extract::{Multipart, State};
    use axum::http::HeaderMap;
    use axum::response::IntoResponse;
    use axum::routing::post;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    // ── segments_to_vtt (Step 1a) ────────────────────────────────────────

    fn two_segments() -> Vec<TranscriptSegment> {
        vec![
            TranscriptSegment {
                idx: 0,
                start_ms: Some(500),
                end_ms: Some(4200),
                speaker: None,
                text: "Hello world".to_string(),
            },
            TranscriptSegment {
                idx: 1,
                start_ms: Some(4200),
                end_ms: Some(8000),
                speaker: None,
                text: "Nice to meet you".to_string(),
            },
        ]
    }

    #[test]
    fn segments_to_vtt_starts_with_webvtt_header_and_formats_cue_timings() {
        let vtt = segments_to_vtt(&two_segments());
        assert!(vtt.starts_with("WEBVTT"));
        assert!(
            vtt.contains("00:00:00.500 --> 00:00:04.200"),
            "expected formatted cue timing in:\n{vtt}"
        );
        assert!(vtt.contains("00:00:04.200 --> 00:00:08.000"));
        assert!(vtt.contains("Hello world"));
        assert!(vtt.contains("Nice to meet you"));
    }

    #[test]
    fn segments_to_vtt_roundtrips_through_the_vtt_parser() {
        let segments = two_segments();
        let vtt = segments_to_vtt(&segments);

        let parsed = parser::parse(TranscriptFormat::Vtt, vtt.as_bytes()).expect("generated vtt must parse");

        assert_eq!(parsed.len(), segments.len());
        for (original, reparsed) in segments.iter().zip(parsed.iter()) {
            assert_eq!(reparsed.idx, original.idx);
            assert_eq!(reparsed.start_ms, original.start_ms);
            assert_eq!(reparsed.end_ms, original.end_ms);
            assert_eq!(reparsed.speaker, None, "whisper never emits speakers");
            assert_eq!(reparsed.text, original.text);
        }
    }

    #[test]
    fn segments_to_vtt_skips_segments_without_timestamps() {
        let segments = vec![
            TranscriptSegment {
                idx: 0,
                start_ms: None,
                end_ms: None,
                speaker: None,
                text: "No timing available".to_string(),
            },
            TranscriptSegment {
                idx: 1,
                start_ms: Some(0),
                end_ms: Some(1000),
                speaker: None,
                text: "Has timing".to_string(),
            },
        ];

        let vtt = segments_to_vtt(&segments);
        assert!(!vtt.contains("No timing available"));
        assert!(vtt.contains("Has timing"));
    }

    // ── transcribe() against a mock Axum server (Step 1b/1c) ─────────────

    #[derive(Default)]
    struct CapturedRequest {
        authorization: Option<String>,
        model: Option<String>,
        saw_file_field: bool,
    }

    fn spawn_mock_server(app: Router) -> String {
        let (addr_tx, addr_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("build mock server runtime");
            rt.block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind mock whisper server");
                addr_tx.send(listener.local_addr().unwrap()).unwrap();
                axum::serve(listener, app).await.unwrap();
            });
        });
        let addr = addr_rx.recv().expect("mock server address");
        format!("http://{addr}")
    }

    const VERBOSE_JSON_RESPONSE: &str = r#"{
        "language": "english",
        "segments": [
            {"start": 0.5, "end": 4.2, "text": "Hello world"},
            {"start": 4.2, "end": 8.0, "text": "Nice to meet you"}
        ]
    }"#;

    async fn capture_and_respond(
        State(state): State<Arc<Mutex<CapturedRequest>>>,
        headers: HeaderMap,
        mut multipart: Multipart,
    ) -> impl IntoResponse {
        let authorization = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let mut model = None;
        let mut saw_file_field = false;
        while let Some(field) = multipart.next_field().await.unwrap() {
            match field.name().map(|n| n.to_string()).as_deref() {
                Some("model") => {
                    model = field.text().await.ok();
                }
                Some("file") => {
                    saw_file_field = true;
                    let _ = field.bytes().await;
                }
                _ => {
                    let _ = field.bytes().await;
                }
            }
        }

        {
            let mut captured = state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            captured.authorization = authorization;
            captured.model = model;
            captured.saw_file_field = saw_file_field;
        }

        (
            axum::http::StatusCode::OK,
            [("content-type", "application/json")],
            VERBOSE_JSON_RESPONSE,
        )
    }

    fn temp_audio_file() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("podfetch-whisper-test-{}.mp3", uuid::Uuid::new_v4()));
        let mut file = std::fs::File::create(&path).expect("create temp audio file");
        file.write_all(b"fake-audio-bytes").expect("write temp audio file");
        path
    }

    #[test]
    fn transcribe_sends_bearer_header_and_parses_segments_when_api_key_is_set() {
        let captured = Arc::new(Mutex::new(CapturedRequest::default()));
        let app = Router::new()
            .route("/v1/audio/transcriptions", post(capture_and_respond))
            .with_state(captured.clone());
        let base_url = spawn_mock_server(app);

        let config = TranscriptionConfig {
            base_url,
            api_key: Some("test-secret-key".to_string()),
            model: "whisper-1".to_string(),
        };
        let client = WhisperClient::new(config);
        let audio_path = temp_audio_file();

        let (segments, language) = client.transcribe(&audio_path).expect("transcribe must succeed");

        assert_eq!(language.as_deref(), Some("english"));
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

        let captured = captured.lock().unwrap();
        assert_eq!(
            captured.authorization.as_deref(),
            Some("Bearer test-secret-key"),
            "Authorization header must be sent when api_key is configured"
        );
        assert_eq!(captured.model.as_deref(), Some("whisper-1"));
        assert!(captured.saw_file_field, "the audio file must be uploaded as a multipart field");

        let _ = std::fs::remove_file(&audio_path);
    }

    #[test]
    fn transcribe_sends_no_authorization_header_when_api_key_is_none() {
        let captured = Arc::new(Mutex::new(CapturedRequest::default()));
        let app = Router::new()
            .route("/v1/audio/transcriptions", post(capture_and_respond))
            .with_state(captured.clone());
        let base_url = spawn_mock_server(app);

        let config = TranscriptionConfig {
            base_url,
            api_key: None,
            model: "whisper-1".to_string(),
        };
        let client = WhisperClient::new(config);
        let audio_path = temp_audio_file();

        let (segments, _language) = client.transcribe(&audio_path).expect("transcribe must succeed");
        assert_eq!(segments.len(), 2);

        let captured = captured.lock().unwrap();
        assert_eq!(
            captured.authorization, None,
            "Authorization header must be absent when no api_key is configured"
        );

        let _ = std::fs::remove_file(&audio_path);
    }

    #[test]
    fn transcribe_returns_err_on_server_500() {
        let app = Router::new().route(
            "/v1/audio/transcriptions",
            post(|| async { axum::http::StatusCode::INTERNAL_SERVER_ERROR }),
        );
        let base_url = spawn_mock_server(app);

        let config = TranscriptionConfig {
            base_url,
            api_key: None,
            model: "whisper-1".to_string(),
        };
        let client = WhisperClient::new(config);
        let audio_path = temp_audio_file();

        let result = client.transcribe(&audio_path);
        assert!(result.is_err(), "a 500 response must yield an Err, never a panic");

        let _ = std::fs::remove_file(&audio_path);
    }

    #[test]
    fn transcribe_returns_err_on_malformed_json_response() {
        let app = Router::new().route(
            "/v1/audio/transcriptions",
            post(|| async {
                (
                    axum::http::StatusCode::OK,
                    [("content-type", "application/json")],
                    "not json at all {",
                )
            }),
        );
        let base_url = spawn_mock_server(app);

        let config = TranscriptionConfig {
            base_url,
            api_key: None,
            model: "whisper-1".to_string(),
        };
        let client = WhisperClient::new(config);
        let audio_path = temp_audio_file();

        let result = client.transcribe(&audio_path);
        assert!(result.is_err(), "malformed json must yield an Err, never a panic");

        let _ = std::fs::remove_file(&audio_path);
    }
}
