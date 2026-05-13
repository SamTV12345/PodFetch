use podfetch_domain::audiobookshelf::media_progress::MediaProgress;
use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaProgressDto {
    pub id: String,
    pub library_item_id: String,
    pub episode_id: Option<String>,
    pub media_type: String,
    pub duration: f64,
    pub progress: f64,
    pub current_time: f64,
    pub is_finished: bool,
    pub hide_from_continue_listening: bool,
    pub last_update: i64,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

impl From<&MediaProgress> for MediaProgressDto {
    fn from(value: &MediaProgress) -> Self {
        Self {
            id: value.id.clone(),
            library_item_id: value.library_item_id.clone(),
            episode_id: value.episode_id.clone(),
            media_type: value.media_type.clone(),
            duration: value.duration,
            progress: value.progress,
            current_time: value.current_time,
            is_finished: value.is_finished,
            hide_from_continue_listening: value.hide_from_continue_listening,
            last_update: value.last_update.and_utc().timestamp_millis(),
            started_at: value.started_at.and_utc().timestamp_millis(),
            finished_at: value.finished_at.map(|t| t.and_utc().timestamp_millis()),
        }
    }
}
