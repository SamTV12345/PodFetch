use actix_web::{post, web, HttpRequest, HttpResponse};
use sha256::digest;
use uuid::Uuid;
use crate::adapters::audiobookshelf::models::login::{AudioBookShelfPermissions, AudioBookShelfUser, AudioBookshelfServerSettings, LoginResponse};
use crate::models::podcasts::Podcast;
use crate::models::settings::Setting;
use crate::models::user::User;
use crate::utils::error::{CustomError, CustomErrorInner};

#[derive(Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[post("/login")]
pub async fn login_audiobookshelf(data: web::Json<LoginData>) -> Result<HttpResponse, CustomError> {
    let mut user = User::find_by_username(&data.username)?;

    match user.password {
        Some(ref password)=>{
            if digest(data.password.clone()) != *password {
                return Err(CustomErrorInner::Forbidden.into())
            }
        },
        _ => return Err(CustomErrorInner::Forbidden.into())
    }

    if user.api_key.is_none() {
        user.api_key = Some(Uuid::new_v4().to_string());
        User::update_user(&user)?;
    }

    generate_response(&user).map(|response| HttpResponse::Ok().json(response))
}

#[post("/api/authorize")]
pub async fn login_audiobookshelf_redundant(req: HttpRequest) -> Result<HttpResponse,
    CustomError> {
    let authorization_header = req.headers().get("Authorization").ok_or
    (CustomErrorInner::Forbidden)?.to_str().map_err(|_| CustomErrorInner::Forbidden)?;
    let auth_vec = authorization_header.split_whitespace().collect::<Vec<&str>>();
    let token = auth_vec.get(1).ok_or(CustomErrorInner::Forbidden)?;

    let user = User::find_by_api_key(token)?.ok_or(CustomErrorInner::Forbidden)?;

    generate_response(&user).map(|response| HttpResponse::Ok().json(response))
}


fn generate_response(user: &User) -> Result<LoginResponse, CustomError> {
    let podcasts = Podcast::get_podcasts(&user.username)?;

    Ok(LoginResponse {
        user: get_user_config(user),
        user_default_library_id: podcasts.first().map(|p| p.id.to_string()),
        server_settings: generate_server_settings()?,
        source: "docker".to_string(),
    })
}


fn generate_server_settings() -> Result<AudioBookshelfServerSettings, CustomError> {
    let settings = Setting::get_settings()?.unwrap();

    Ok(AudioBookshelfServerSettings{
        id: settings.id.to_string(),
        scanner_finds_covers: false,
        scanner_cover_provider: "google".to_string(),
        scanner_parse_subtitle: false,
        scanner_prefer_matched_metadata: false,
        scanner_disable_watcher: true,
        store_cover_with_item: false,
        store_metadata_with_item: false,
        metadata_file_format: "json".to_string(),
        rate_limit_login_requests: 10,
        rate_limit_login_window: 600000,
        backup_schedule: "30 1 * * *".to_string(),
        backups_to_keep: 0,
        max_backup_size: 0,
        logger_daily_logs_to_keep: 0,
        logger_scanner_logs_to_keep: 0,
        home_bookshelf_view: 0,
        bookshelf_view: 0,
        sorting_ignore_prefix: false,
        sorting_prefixes: vec![],
        chromecast_enabled: false,
        date_format: "MM/dd/yy".to_string(),
        time_format: "HH:mm".to_string(),
        language: "en-us".to_string(),
        log_level: 2,
        version: "2.2.5".to_string(),
    })
}


fn get_user_config(user: &User) -> AudioBookShelfUser {
    AudioBookShelfUser {
        id: user.id.to_string(),
        username: user.username.to_string(),
        r#type: "root".to_string(),
        token: user.api_key.clone().unwrap(),
        // TODO add data here
        media_progress: vec![],
        series_hide_from_continue_listening: vec![],
        bookmarks: vec![],
        is_active: true,
        is_locked: false,
        last_seen: None,
        created_at: 0.0,
        permissions: AudioBookShelfPermissions {
            download: true,
            update: true,
            delete: true,
            upload: true,
            access_all_libraries: true,
            access_all_tags: true,
            access_explicit_content: true,
        },
        item_tags_accessible: vec![],
        libraries_accessible: vec![],
    }
}