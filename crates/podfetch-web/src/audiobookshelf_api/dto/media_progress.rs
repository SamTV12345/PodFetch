use podfetch_domain::audiobookshelf::media_progress::MediaProgress;
use serde::Serialize;

/// audiobookshelf-shape MediaProgress payload (upstream User.toOldJSONForBrowser
/// includes this in user.mediaProgress[]). Field set mirrors
/// `server/models/MediaProgress.js` toJSON / getOldMediaProgress.
#[derive(Serialize, utoipa::ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaProgressDto {
    pub id: String,
    pub user_id: String,
    pub library_item_id: String,
    pub episode_id: Option<String>,
    pub media_item_id: String,
    pub media_item_type: String,
    pub duration: f64,
    pub progress: f64,
    pub current_time: f64,
    pub is_finished: bool,
    pub hide_from_continue_listening: bool,
    pub ebook_location: Option<String>,
    pub ebook_progress: Option<f64>,
    pub last_update: i64,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

impl From<&MediaProgress> for MediaProgressDto {
    fn from(value: &MediaProgress) -> Self {
        let media_item_type = if value.episode_id.is_some() {
            "podcastEpisode".to_string()
        } else if value.media_type == "book" {
            "book".to_string()
        } else {
            "podcastEpisode".to_string()
        };
        let media_item_id = value
            .episode_id
            .clone()
            .unwrap_or_else(|| value.library_item_id.clone());
        Self {
            id: value.id.clone(),
            user_id: value.user_id.to_string(),
            library_item_id: value.library_item_id.clone(),
            episode_id: value.episode_id.clone(),
            media_item_id,
            media_item_type,
            duration: value.duration,
            progress: value.progress,
            current_time: value.current_time,
            is_finished: value.is_finished,
            hide_from_continue_listening: value.hide_from_continue_listening,
            ebook_location: None,
            ebook_progress: None,
            last_update: value.last_update.and_utc().timestamp_millis(),
            started_at: value.started_at.and_utc().timestamp_millis(),
            finished_at: value.finished_at.map(|t| t.and_utc().timestamp_millis()),
        }
    }
}
