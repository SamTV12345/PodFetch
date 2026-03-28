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

impl From<podfetch_domain::podcast_settings::PodcastSetting> for PodcastSetting {
    fn from(value: podfetch_domain::podcast_settings::PodcastSetting) -> Self {
        Self {
            podcast_id: value.podcast_id,
            episode_numbering: value.episode_numbering,
            auto_download: value.auto_download,
            auto_update: value.auto_update,
            auto_cleanup: value.auto_cleanup,
            auto_cleanup_days: value.auto_cleanup_days,
            replace_invalid_characters: value.replace_invalid_characters,
            use_existing_filename: value.use_existing_filename,
            replacement_strategy: value.replacement_strategy,
            episode_format: value.episode_format,
            podcast_format: value.podcast_format,
            direct_paths: value.direct_paths,
            activated: value.activated,
            podcast_prefill: value.podcast_prefill,
        }
    }
}

impl From<PodcastSetting> for podfetch_domain::podcast_settings::PodcastSetting {
    fn from(value: PodcastSetting) -> Self {
        Self {
            podcast_id: value.podcast_id,
            episode_numbering: value.episode_numbering,
            auto_download: value.auto_download,
            auto_update: value.auto_update,
            auto_cleanup: value.auto_cleanup,
            auto_cleanup_days: value.auto_cleanup_days,
            replace_invalid_characters: value.replace_invalid_characters,
            use_existing_filename: value.use_existing_filename,
            replacement_strategy: value.replacement_strategy,
            episode_format: value.episode_format,
            podcast_format: value.podcast_format,
            direct_paths: value.direct_paths,
            activated: value.activated,
            podcast_prefill: value.podcast_prefill,
        }
    }
}
