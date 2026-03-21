use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct PodcastSetting {
    pub podcast_id: i32,
    pub episode_numbering: bool,
    pub auto_download: bool,
    pub auto_update: bool,
    pub auto_cleanup: bool,
    pub auto_cleanup_days: i32,
    pub replace_invalid_characters: bool,
    pub use_existing_filename: bool,
    pub replacement_strategy: String,
    pub episode_format: String,
    pub podcast_format: String,
    pub direct_paths: bool,
    pub activated: bool,
    pub podcast_prefill: i32,
}

pub trait PodcastSettingsRepository: Send + Sync {
    type Error;

    fn get_settings(&self, podcast_id: i32) -> Result<Option<PodcastSetting>, Self::Error>;
    fn upsert_settings(&self, setting: PodcastSetting) -> Result<PodcastSetting, Self::Error>;
}
