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

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AbsUserDto {
    pub id: String,
    pub username: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub token: String,
    pub is_active: bool,
    pub permissions: PermissionsDto,
    pub libraries_accessible: Vec<String>,
    pub media_progress: Vec<MediaProgressDto>,
}

impl AbsUserDto {
    pub fn from_user(user: &User, media_progress: Vec<MediaProgressDto>) -> Self {
        let user_type = if user.role.eq_ignore_ascii_case("admin") {
            "root"
        } else {
            "user"
        };
        Self {
            id: user.id.to_string(),
            username: user.username.clone(),
            user_type: user_type.to_string(),
            token: user.api_key.clone().unwrap_or_default(),
            is_active: true,
            permissions: PermissionsDto::for_role(&user.role),
            libraries_accessible: Vec::new(),
            media_progress,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerSettingsDto {
    pub server_version: String,
    pub language: String,
    pub date_format: String,
    pub time_format: String,
}

impl ServerSettingsDto {
    pub fn default_settings() -> Self {
        Self {
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            language: "en-us".to_string(),
            date_format: "MM/dd/yyyy".to_string(),
            time_format: "h:mma".to_string(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub user: AbsUserDto,
    pub user_default_library_id: Option<String>,
    pub server_settings: ServerSettingsDto,
}
