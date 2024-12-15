use diesel::{AsChangeset, Identifiable, Insertable, Queryable};

#[derive(
    Queryable,
    Insertable,
    Debug,
    Clone,
    Identifiable,
    AsChangeset,
    Default,
)]
#[diesel(primary_key(podcast_id))]
pub struct PodcastSettingEntity {
    #[diesel(sql_type = Integer)]
    pub podcast_id: i32,
    #[diesel(sql_type = Bool)]
    pub episode_numbering: bool,
    #[diesel(sql_type = Bool)]
    pub auto_download: bool,
    #[diesel(sql_type = Bool)]
    pub auto_update: bool,
    #[diesel(sql_type = Bool)]
    pub auto_cleanup: bool,
    #[diesel(sql_type = Integer)]
    pub auto_cleanup_days: i32,
    #[diesel(sql_type = Bool)]
    pub replace_invalid_characters: bool,
    #[diesel(sql_type = Bool)]
    pub use_existing_filename: bool,
    #[diesel(sql_type = Text)]
    pub replacement_strategy: String,
    #[diesel(sql_type = Text)]
    pub episode_format: String,
    #[diesel(sql_type = Text)]
    pub podcast_format: String,
    #[diesel(sql_type = Bool)]
    pub direct_paths: bool,
    #[diesel(sql_type = Bool)]
    pub activated: bool,
    #[diesel(sql_type = Integer)]
    pub podcast_prefill: i32,
}