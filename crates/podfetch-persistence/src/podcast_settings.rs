use crate::db::{Database, PersistenceError};
use diesel::OptionalExtension;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_domain::podcast_settings::{PodcastSetting, PodcastSettingsRepository};
use podfetch_domain::sponsorblock::{categories_from_csv, categories_to_csv};

diesel::table! {
    podcast_settings (podcast_id) {
        podcast_id -> Integer,
        episode_numbering -> Bool,
        auto_download -> Bool,
        auto_update -> Bool,
        auto_cleanup -> Bool,
        auto_cleanup_days -> Integer,
        replace_invalid_characters -> Bool,
        use_existing_filename -> Bool,
        replacement_strategy -> Text,
        episode_format -> Text,
        podcast_format -> Text,
        direct_paths -> Bool,
        activated -> Bool,
        podcast_prefill -> Integer,
        use_one_cover_for_all_episodes -> Bool,
        sponsorblock_enabled -> Bool,
        sponsorblock_categories -> Text,
    }
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = podcast_settings)]
struct PodcastSettingEntity {
    podcast_id: i32,
    episode_numbering: bool,
    auto_download: bool,
    auto_update: bool,
    auto_cleanup: bool,
    auto_cleanup_days: i32,
    replace_invalid_characters: bool,
    use_existing_filename: bool,
    replacement_strategy: String,
    episode_format: String,
    podcast_format: String,
    direct_paths: bool,
    activated: bool,
    podcast_prefill: i32,
    use_one_cover_for_all_episodes: bool,
    sponsorblock_enabled: bool,
    sponsorblock_categories: String,
}

impl From<PodcastSettingEntity> for PodcastSetting {
    fn from(value: PodcastSettingEntity) -> Self {
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
            use_one_cover_for_all_episodes: value.use_one_cover_for_all_episodes,
            sponsorblock_enabled: value.sponsorblock_enabled,
            sponsorblock_categories: categories_from_csv(&value.sponsorblock_categories),
        }
    }
}

impl From<PodcastSetting> for PodcastSettingEntity {
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
            use_one_cover_for_all_episodes: value.use_one_cover_for_all_episodes,
            sponsorblock_enabled: value.sponsorblock_enabled,
            sponsorblock_categories: categories_to_csv(&value.sponsorblock_categories),
        }
    }
}

pub struct DieselPodcastSettingsRepository {
    database: Database,
}

impl DieselPodcastSettingsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl PodcastSettingsRepository for DieselPodcastSettingsRepository {
    type Error = PersistenceError;

    fn get_settings(&self, podcast_id: i32) -> Result<Option<PodcastSetting>, Self::Error> {
        use self::podcast_settings::dsl as podcast_settings_dsl;

        podcast_settings_dsl::podcast_settings
            .filter(podcast_settings_dsl::podcast_id.eq(podcast_id))
            .first::<PodcastSettingEntity>(&mut self.database.connection()?)
            .optional()
            .map(|setting| setting.map(Into::into))
            .map_err(Into::into)
    }

    fn upsert_settings(&self, setting: PodcastSetting) -> Result<PodcastSetting, Self::Error> {
        use self::podcast_settings::dsl as podcast_settings_dsl;

        let entity = PodcastSettingEntity::from(setting);
        if self.get_settings(entity.podcast_id)?.is_some() {
            diesel::update(podcast_settings_dsl::podcast_settings.find(entity.podcast_id))
                .set(entity.clone())
                .execute(&mut self.database.connection()?)?;
        } else {
            diesel::insert_into(podcast_settings_dsl::podcast_settings)
                .values(entity.clone())
                .execute(&mut self.database.connection()?)?;
        }

        self.get_settings(entity.podcast_id)?
            .ok_or_else(|| PersistenceError::Database(diesel::result::Error::NotFound))
    }
}
