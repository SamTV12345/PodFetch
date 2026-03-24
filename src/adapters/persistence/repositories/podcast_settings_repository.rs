use common_infrastructure::error::CustomError;
use podfetch_domain::podcast_settings::{PodcastSetting, PodcastSettingsRepository};
use podfetch_persistence::db::Database;
use podfetch_persistence::podcast_settings::DieselPodcastSettingsRepository;

pub struct PodcastSettingsRepositoryImpl {
    inner: DieselPodcastSettingsRepository,
}

impl PodcastSettingsRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPodcastSettingsRepository::new(database),
        }
    }
}

impl PodcastSettingsRepository for PodcastSettingsRepositoryImpl {
    type Error = CustomError;

    fn get_settings(&self, podcast_id: i32) -> Result<Option<PodcastSetting>, Self::Error> {
        self.inner.get_settings(podcast_id).map_err(Into::into)
    }

    fn upsert_settings(&self, setting: PodcastSetting) -> Result<PodcastSetting, Self::Error> {
        self.inner.upsert_settings(setting).map_err(Into::into)
    }
}

