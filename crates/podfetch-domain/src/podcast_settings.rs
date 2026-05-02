#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
    pub use_one_cover_for_all_episodes: bool,
    pub sponsorblock_enabled: bool,
    pub sponsorblock_categories: Vec<crate::sponsorblock::SponsorBlockCategory>,
}

pub trait PodcastSettingsRepository: Send + Sync {
    type Error;

    fn get_settings(&self, podcast_id: i32) -> Result<Option<PodcastSetting>, Self::Error>;
    fn upsert_settings(&self, setting: PodcastSetting) -> Result<PodcastSetting, Self::Error>;
}
