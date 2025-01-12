use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub user: AudioBookShelfUser,
    pub user_default_library_id: Option<String>,
    pub server_settings: AudioBookshelfServerSettings,
    pub source: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AudioBookShelfUser {
    pub(crate) id: String,
    pub(crate)username: String,
    pub(crate)r#type: String,
    pub(crate)token: String,
    pub(crate)media_progress: Vec<MediaProgress>,
    pub(crate)series_hide_from_continue_listening: Vec<String>,
    pub(crate)bookmarks: Vec<String>,
    pub(crate) is_active: bool,
    pub(crate)is_locked: bool,
    pub(crate)last_seen: Option<f64>,
    pub(crate)created_at: f64,
    pub(crate)permissions: AudioBookShelfPermissions,
    pub(crate)libraries_accessible: Vec<String>,
    pub(crate)item_tags_accessible: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AudioBookShelfPermissions {
    pub download: bool,
    pub update: bool,
    pub delete: bool,
    pub upload: bool,
    pub access_all_libraries: bool,
    pub access_all_tags: bool,
    pub access_explicit_content: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MediaProgress {
    pub id: String,
    pub library_item_id: String,
    pub episode_id: String,
    pub duration: f64,
    pub progress: f64,
    pub current_time: f64,
    pub  is_finished: bool,
    pub hide_from_continue_listening: bool,
    pub last_update: f64,
    pub started_at: f64,
    pub finished_at: Option<f64>
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AudioBookshelfServerSettings {
    pub id: String,
    pub scanner_finds_covers: bool,
    pub scanner_cover_provider: String,
    pub scanner_parse_subtitle: bool,
    pub scanner_prefer_matched_metadata: bool,
    pub scanner_disable_watcher: bool,
    pub store_cover_with_item: bool,
    pub store_metadata_with_item: bool,
    pub metadata_file_format: String,
    pub rate_limit_login_requests: i32,
    pub rate_limit_login_window: i32,
    pub backup_schedule: String,
    pub backups_to_keep: i32,
    pub max_backup_size: i32,
    pub logger_daily_logs_to_keep: i32,
    pub logger_scanner_logs_to_keep: i32,
    pub home_bookshelf_view: u8,
    pub bookshelf_view: u8,
    pub sorting_ignore_prefix: bool,
    pub sorting_prefixes: Vec<String>,
    pub chromecast_enabled: bool,
    pub date_format: String,
    pub time_format: String,
    pub language: String,
    pub log_level: u8,
    pub version: String
}