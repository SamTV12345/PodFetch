use crate::audiobookshelf_api::dto::media_progress::MediaProgressDto;
use podfetch_domain::user::User;
use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsDto {
    pub download: bool,
    pub update: bool,
    pub delete: bool,
    pub upload: bool,
    pub access_all_libraries: bool,
    pub access_all_tags: bool,
    pub access_explicit_content: bool,
}

impl PermissionsDto {
    pub fn for_role(role: &str) -> Self {
        let is_admin = role.eq_ignore_ascii_case("admin");
        Self {
            download: true,
            update: is_admin,
            delete: is_admin,
            upload: is_admin,
            access_all_libraries: true,
            access_all_tags: true,
            access_explicit_content: true,
        }
    }
}

/// 100 % audiobookshelf-shape user payload. Field set mirrors
/// `User.toOldJSONForBrowser()` in `server/models/User.js` so the mobile
/// apps don't crash on missing keys.
#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AbsUserDto {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    #[serde(rename = "type")]
    pub user_type: String,
    pub token: String,
    pub is_old_token: bool,
    pub is_active: bool,
    pub is_locked: bool,
    pub last_seen: i64,
    pub created_at: i64,
    pub permissions: PermissionsDto,
    pub libraries_accessible: Vec<String>,
    pub item_tags_selected: Vec<String>,
    pub bookmarks: Vec<serde_json::Value>,
    pub series_hide_from_continue_listening: Vec<String>,
    #[serde(rename = "hasOpenIDLink")]
    pub has_open_id_link: bool,
    pub media_progress: Vec<MediaProgressDto>,
}

impl AbsUserDto {
    pub fn from_user(user: &User, media_progress: Vec<MediaProgressDto>) -> Self {
        let user_type = if user.role.eq_ignore_ascii_case("admin") {
            "root"
        } else {
            "user"
        };
        let now_ms = chrono::Utc::now().timestamp_millis();
        Self {
            id: user.id.to_string(),
            username: user.username.clone(),
            email: None,
            user_type: user_type.to_string(),
            token: user.api_key.clone().unwrap_or_default(),
            is_old_token: false,
            is_active: true,
            is_locked: false,
            last_seen: now_ms,
            created_at: user.created_at.and_utc().timestamp_millis(),
            permissions: PermissionsDto::for_role(&user.role),
            libraries_accessible: Vec::new(),
            item_tags_selected: Vec::new(),
            bookmarks: Vec::new(),
            series_hide_from_continue_listening: Vec::new(),
            has_open_id_link: false,
            media_progress,
        }
    }
}

/// audiobookshelf ServerSettings.toJSONForBrowser() shape. Only the fields
/// the mobile apps actually read are populated; everything else can grow
/// later if a client complains about a missing key.
#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerSettingsDto {
    pub id: String,
    pub version: String,
    pub build_number: i64,
    pub language: String,
    pub date_format: String,
    pub time_format: String,
    pub auth_active_auth_methods: Vec<String>,
    pub chromecast_enabled: bool,
    pub bookshelf_view: i32,
    pub home_bookshelf_view: i32,
    pub sorting_ignore_prefix: bool,
    pub sorting_prefixes: Vec<String>,
    pub allow_iframe: bool,
    pub log_level: i32,
    pub allowed_origins: Vec<String>,
    pub auth_login_custom_message: Option<String>,
    pub server_version: String,
}

impl ServerSettingsDto {
    pub fn default_settings() -> Self {
        Self {
            id: "server-settings".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_number: 0,
            language: "en-us".to_string(),
            date_format: "MM/dd/yyyy".to_string(),
            time_format: "h:mma".to_string(),
            auth_active_auth_methods: vec!["local".to_string()],
            chromecast_enabled: false,
            bookshelf_view: 1,
            home_bookshelf_view: 1,
            sorting_ignore_prefix: false,
            sorting_prefixes: vec!["the".to_string(), "a".to_string()],
            allow_iframe: false,
            log_level: 2,
            allowed_origins: Vec::new(),
            auth_login_custom_message: None,
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Login / authorize response. Field order + spelling mirrors upstream
/// `Auth.getUserLoginResponsePayload`. The capitalised `Source` is
/// upstream's `global.Source` and is preserved as-is - mobile apps read
/// it with that exact key.
#[derive(Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub user: AbsUserDto,
    #[serde(rename = "userDefaultLibraryId")]
    pub user_default_library_id: Option<String>,
    #[serde(rename = "serverSettings")]
    pub server_settings: ServerSettingsDto,
    #[serde(rename = "ereaderDevices")]
    pub ereader_devices: Vec<serde_json::Value>,
    #[serde(rename = "Source")]
    pub source: String,
}
