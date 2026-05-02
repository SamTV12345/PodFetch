use crate::db::{Database, PersistenceError};
use diesel::insert_into;
use diesel::prelude::{AsChangeset, Identifiable, Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, RunQueryDsl};
use podfetch_domain::settings::{Setting, SettingRepository};
use podfetch_domain::sponsorblock::{categories_from_csv, categories_to_csv};

diesel::table! {
    settings (id) {
        id -> Integer,
        auto_download -> Bool,
        auto_update -> Bool,
        auto_cleanup -> Bool,
        auto_cleanup_days -> Integer,
        podcast_prefill -> Integer,
        replace_invalid_characters -> Bool,
        use_existing_filename -> Bool,
        replacement_strategy -> Text,
        episode_format -> Text,
        podcast_format -> Text,
        direct_paths -> Bool,
        auto_transcode_opus -> Bool,
        use_one_cover_for_all_episodes -> Bool,
        sponsorblock_enabled -> Bool,
        sponsorblock_categories -> Text,
    }
}

#[derive(Queryable, Insertable, Identifiable, AsChangeset, Debug, Clone)]
#[diesel(table_name = settings)]
struct SettingEntity {
    id: i32,
    auto_download: bool,
    auto_update: bool,
    auto_cleanup: bool,
    auto_cleanup_days: i32,
    podcast_prefill: i32,
    replace_invalid_characters: bool,
    use_existing_filename: bool,
    replacement_strategy: String,
    episode_format: String,
    podcast_format: String,
    direct_paths: bool,
    auto_transcode_opus: bool,
    use_one_cover_for_all_episodes: bool,
    sponsorblock_enabled: bool,
    sponsorblock_categories: String,
}

impl From<SettingEntity> for Setting {
    fn from(value: SettingEntity) -> Self {
        Self {
            id: value.id,
            auto_download: value.auto_download,
            auto_update: value.auto_update,
            auto_cleanup: value.auto_cleanup,
            auto_cleanup_days: value.auto_cleanup_days,
            podcast_prefill: value.podcast_prefill,
            replace_invalid_characters: value.replace_invalid_characters,
            use_existing_filename: value.use_existing_filename,
            replacement_strategy: value.replacement_strategy,
            episode_format: value.episode_format,
            podcast_format: value.podcast_format,
            direct_paths: value.direct_paths,
            auto_transcode_opus: value.auto_transcode_opus,
            use_one_cover_for_all_episodes: value.use_one_cover_for_all_episodes,
            sponsorblock_enabled: value.sponsorblock_enabled,
            sponsorblock_categories: categories_from_csv(&value.sponsorblock_categories),
        }
    }
}

impl From<Setting> for SettingEntity {
    fn from(value: Setting) -> Self {
        Self {
            id: value.id,
            auto_download: value.auto_download,
            auto_update: value.auto_update,
            auto_cleanup: value.auto_cleanup,
            auto_cleanup_days: value.auto_cleanup_days,
            podcast_prefill: value.podcast_prefill,
            replace_invalid_characters: value.replace_invalid_characters,
            use_existing_filename: value.use_existing_filename,
            replacement_strategy: value.replacement_strategy,
            episode_format: value.episode_format,
            podcast_format: value.podcast_format,
            direct_paths: value.direct_paths,
            auto_transcode_opus: value.auto_transcode_opus,
            use_one_cover_for_all_episodes: value.use_one_cover_for_all_episodes,
            sponsorblock_enabled: value.sponsorblock_enabled,
            sponsorblock_categories: categories_to_csv(&value.sponsorblock_categories),
        }
    }
}

pub struct DieselSettingsRepository {
    database: Database,
}

impl DieselSettingsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl SettingRepository for DieselSettingsRepository {
    type Error = PersistenceError;

    fn get_settings(&self) -> Result<Option<Setting>, Self::Error> {
        use self::settings::dsl::*;

        settings
            .first::<SettingEntity>(&mut self.database.connection()?)
            .optional()
            .map(|setting| setting.map(Into::into))
            .map_err(Into::into)
    }

    fn update_settings(&self, setting: Setting) -> Result<Setting, Self::Error> {
        use self::settings::dsl::*;

        let mut conn = self.database.connection()?;
        let setting_to_update = settings.first::<SettingEntity>(&mut conn)?;

        diesel::update(&setting_to_update)
            .set(SettingEntity::from(setting))
            .get_result::<SettingEntity>(&mut conn)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn insert_default_settings(&self) -> Result<(), Self::Error> {
        use self::settings::dsl::*;

        let mut conn = self.database.connection()?;
        insert_into(settings)
            .values((
                id.eq(1),
                auto_update.eq(true),
                auto_download.eq(true),
                auto_cleanup.eq(true),
                auto_cleanup_days.eq(30),
                podcast_prefill.eq(5),
                replace_invalid_characters.eq(false),
                use_existing_filename.eq(false),
                replacement_strategy.eq("replace-with-dash"),
                episode_format.eq("{episodeTitle}"),
                podcast_format.eq("{podcastTitle}"),
                direct_paths.eq(false),
                auto_transcode_opus.eq(false),
                use_one_cover_for_all_episodes.eq(false),
                sponsorblock_enabled.eq(false),
                sponsorblock_categories.eq("sponsor,selfpromo,interaction"),
            ))
            .execute(&mut conn)
            .map(|_| ())
            .map_err(Into::into)
    }
}
