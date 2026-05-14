use podfetch_domain::audiobookshelf::playback_session::PlaybackSession;
use serde::{Deserialize, Serialize};

/// Audiobookshelf audio-track payload. Mirrors upstream
/// `PodcastEpisode.getAudioTrack` / `AudioFile.toJSON` plus the fields the
/// **Android-app's Kotlin AudioTrack data class** requires non-null
/// (`isLocal`). Missing `isLocal` makes Jackson throw
/// `MissingKotlinParameterException` and the play silently dies in the
/// OkHttp callback - that's the "spinner forever" symptom.
#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackAudioTrackDto {
    pub index: i32,
    pub ino: String,
    pub start_offset: f64,
    pub duration: f64,
    pub title: String,
    pub content_url: String,
    pub mime_type: String,
    pub codec: String,
    pub metadata: AudioTrackMetadataDto,
    pub bit_rate: i64,
    pub language: Option<String>,
    pub time_base: String,
    pub channels: i32,
    pub channel_layout: String,
    pub chapters: Vec<serde_json::Value>,
    pub embedded_cover_art: Option<String>,
    pub manually_verified: bool,
    pub invalid: bool,
    pub exclude: bool,
    /// Required non-null by the Android-app Kotlin AudioTrack data class.
    pub is_local: bool,
    pub local_file_id: Option<String>,
    pub server_index: Option<i32>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrackMetadataDto {
    pub filename: String,
    pub ext: String,
    pub path: String,
    pub rel_path: String,
    pub size: i64,
    pub mtime_ms: i64,
    pub ctime_ms: i64,
    pub birthtime_ms: i64,
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
    /// Required non-null by the Android-app Kotlin `DeviceInfo` data class
    /// (deviceId / manufacturer / model / sdkVersion / clientVersion).
    /// Null here would make Jackson abort PlaybackSession deserialisation.
    pub device_info: serde_json::Value,
    pub server_version: String,
    pub date: String,
    pub day_of_week: String,
    /// Kotlin expects this as `Long` - send an integer JSON literal, not 0.0.
    pub time_listening: i64,
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
            // Echo back what the client sent (it includes its own deviceId
            // etc.) or fall back to a fully-populated stub so Jackson never
            // has to deserialize null into the non-null DeviceInfo class.
            device_info: session
                .device_info_json
                .as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .map(merge_device_info_defaults)
                .unwrap_or_else(default_device_info),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            date,
            day_of_week,
            time_listening: session.time_listening_total.round() as i64,
            start_time: 0.0,
            current_time: session.current_time,
            started_at: session.started_at.and_utc().timestamp_millis(),
            updated_at: session.updated_at.and_utc().timestamp_millis(),
            audio_tracks,
            library_item: None,
        }
    }
}

/// Minimal `DeviceInfo` matching the Kotlin data class field set so Jackson
/// is happy. Used when the client doesn't send its own `deviceInfo` in the
/// play request body.
fn default_device_info() -> serde_json::Value {
    serde_json::json!({
        "deviceId": "podfetch",
        "manufacturer": "podfetch",
        "model": "podfetch",
        "sdkVersion": 0,
        "clientVersion": env!("CARGO_PKG_VERSION"),
    })
}

/// If the client supplied a partial `deviceInfo`, fill any missing
/// required fields so the response still deserialises on the Android side.
fn merge_device_info_defaults(client: serde_json::Value) -> serde_json::Value {
    let mut merged = default_device_info();
    if let (Some(target), Some(source)) = (merged.as_object_mut(), client.as_object()) {
        for (key, value) in source {
            if !value.is_null() {
                target.insert(key.clone(), value.clone());
            }
        }
    }
    merged
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
