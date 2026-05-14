use podfetch_domain::audiobookshelf::playback_session::PlaybackSession;
use serde::{Deserialize, Serialize};

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackAudioTrackDto {
    pub index: i32,
    pub start_offset: f64,
    pub duration: f64,
    pub title: String,
    pub content_url: String,
    pub mime_type: String,
    pub codec: String,
    pub metadata: AudioTrackMetadataDto,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrackMetadataDto {
    pub filename: String,
    pub ext: String,
    pub path: String,
    pub rel_path: String,
}

/// 100 % audiobookshelf-shape per upstream
/// `PlaybackSession.toJSONForClient()` (`server/objects/PlaybackSession.js`).
/// Some fields are omitted when null per upstream (e.g. `coverAspectRatio`,
/// `libraryItem`); we always serialise them as null to keep the shape
/// stable for clients that index by key.
#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSessionDto {
    pub id: String,
    pub user_id: String,
    pub library_id: Option<String>,
    pub library_item_id: String,
    pub book_id: Option<String>,
    pub episode_id: Option<String>,
    pub media_type: String,
    pub media_metadata: serde_json::Value,
    pub chapters: Vec<serde_json::Value>,
    pub display_title: Option<String>,
    pub display_author: Option<String>,
    pub cover_path: Option<String>,
    pub duration: f64,
    pub play_method: i32,
    pub media_player: String,
    pub device_info: Option<serde_json::Value>,
    pub server_version: String,
    pub date: String,
    pub day_of_week: String,
    pub time_listening: f64,
    pub start_time: f64,
    pub current_time: f64,
    pub started_at: i64,
    pub updated_at: i64,
    pub audio_tracks: Vec<PlaybackAudioTrackDto>,
    pub library_item: Option<serde_json::Value>,
}

impl PlaybackSessionDto {
    pub fn from_domain(
        session: &PlaybackSession,
        audio_tracks: Vec<PlaybackAudioTrackDto>,
    ) -> Self {
        let started_at = session.started_at.and_utc();
        let date = started_at.format("%Y-%m-%d").to_string();
        let day_of_week = started_at.format("%A").to_string();
        let book_id = match session.media_type.as_str() {
            "book" => Some(session.library_item_id.clone()),
            _ => None,
        };
        Self {
            id: session.id.clone(),
            user_id: session.user_id.to_string(),
            library_id: session.library_id.clone(),
            library_item_id: session.library_item_id.clone(),
            book_id,
            episode_id: session.episode_id.clone(),
            media_type: session.media_type.clone(),
            media_metadata: session
                .media_metadata_json
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_else(|| serde_json::json!({})),
            chapters: Vec::new(),
            display_title: session.display_title.clone(),
            display_author: session.display_author.clone(),
            cover_path: session.cover_path.clone(),
            duration: session.duration,
            play_method: session.play_method.as_i32(),
            media_player: "unknown".to_string(),
            device_info: session
                .device_info_json
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok()),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            date,
            day_of_week,
            time_listening: session.time_listening_total,
            start_time: 0.0,
            current_time: session.current_time,
            started_at: session.started_at.and_utc().timestamp_millis(),
            updated_at: session.updated_at.and_utc().timestamp_millis(),
            audio_tracks,
            library_item: None,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayRequestBody {
    pub media_player: Option<String>,
    pub force_transcode: Option<bool>,
    pub force_direct_play: Option<bool>,
    pub supported_mime_types: Option<Vec<String>>,
    pub device_info: Option<serde_json::Value>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SyncRequestBody {
    pub current_time: f64,
    #[serde(default)]
    pub time_listened: f64,
    #[serde(default)]
    pub duration: f64,
}
