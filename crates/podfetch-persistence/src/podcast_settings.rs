use crate::db::{Database, PersistenceError};
use diesel::OptionalExtension;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_domain::podcast_settings::{PodcastSetting, PodcastSettingsRepository};
use uuid::Uuid;

diesel::table! {
    podcast_settings (podcast_id) {
        podcast_id -> Text,
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
        nfo_format -> Text,
        cover_filename -> Text,
    }
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = podcast_settings)]
struct PodcastSettingEntity {
    podcast_id: String,
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
    nfo_format: String,
    cover_filename: String,
}

impl From<PodcastSettingEntity> for PodcastSetting {
    fn from(value: PodcastSettingEntity) -> Self {
        Self {
            podcast_id: Uuid::parse_str(&value.podcast_id).expect("valid uuid in db"),
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
            nfo_format: value.nfo_format,
            cover_filename: value.cover_filename,
        }
    }
}

impl From<PodcastSetting> for PodcastSettingEntity {
    fn from(value: PodcastSetting) -> Self {
        Self {
            podcast_id: value.podcast_id.to_string(),
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
            nfo_format: value.nfo_format,
            cover_filename: value.cover_filename,
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

    fn get_settings(&self, podcast_id: Uuid) -> Result<Option<PodcastSetting>, Self::Error> {
        use self::podcast_settings::dsl as podcast_settings_dsl;

        podcast_settings_dsl::podcast_settings
            .filter(podcast_settings_dsl::podcast_id.eq(podcast_id.to_string()))
            .first::<PodcastSettingEntity>(&mut self.database.connection()?)
            .optional()
            .map(|setting| setting.map(Into::into))
            .map_err(Into::into)
    }

    fn upsert_settings(&self, setting: PodcastSetting) -> Result<PodcastSetting, Self::Error> {
        use self::podcast_settings::dsl as podcast_settings_dsl;

        let podcast_id = setting.podcast_id;
        let entity = PodcastSettingEntity::from(setting);
        if self.get_settings(podcast_id)?.is_some() {
            diesel::update(podcast_settings_dsl::podcast_settings.find(entity.podcast_id.clone()))
                .set(entity.clone())
                .execute(&mut self.database.connection()?)?;
        } else {
            diesel::insert_into(podcast_settings_dsl::podcast_settings)
                .values(entity.clone())
                .execute(&mut self.database.connection()?)?;
        }

        self.get_settings(podcast_id)?
            .ok_or_else(|| PersistenceError::Database(diesel::result::Error::NotFound))
    }
}
