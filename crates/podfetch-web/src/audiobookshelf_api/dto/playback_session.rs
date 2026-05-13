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

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSessionDto {
    pub id: String,
    pub user_id: String,
    pub library_id: Option<String>,
    pub library_item_id: String,
    pub episode_id: Option<String>,
    pub media_type: String,
    pub media_player: String,
    pub play_method: i32,
    pub duration: f64,
    pub current_time: f64,
    pub time_listening: f64,
    pub started_at: i64,
    pub updated_at: i64,
    pub display_title: Option<String>,
    pub display_author: Option<String>,
    pub cover_path: Option<String>,
    pub audio_tracks: Vec<PlaybackAudioTrackDto>,
}

impl PlaybackSessionDto {
    pub fn from_domain(session: &PlaybackSession, audio_tracks: Vec<PlaybackAudioTrackDto>) -> Self {
        Self {
            id: session.id.clone(),
            user_id: session.user_id.to_string(),
            library_id: session.library_id.clone(),
            library_item_id: session.library_item_id.clone(),
            episode_id: session.episode_id.clone(),
            media_type: session.media_type.clone(),
            media_player: "unknown".to_string(),
            play_method: session.play_method.as_i32(),
            duration: session.duration,
            current_time: session.current_time,
            time_listening: session.time_listening_total,
            started_at: session.started_at.and_utc().timestamp_millis(),
            updated_at: session.updated_at.and_utc().timestamp_millis(),
            display_title: session.display_title.clone(),
            display_author: session.display_author.clone(),
            cover_path: session.cover_path.clone(),
            audio_tracks,
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
