use crate::models::podcasts::Podcast;
use crate::models::settings::Setting;
use crate::models::user::User;
use crate::mutex::LockResultExt;
use crate::service::environment_service::EnvironmentService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::settings_service::SettingsService;
use crate::DbPool;
use actix_web::web::{Data, Path};
use actix_web::{get, put};
use actix_web::{web, HttpResponse};
use chrono::Local;
use std::ops::DerefMut;
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the current settings")),
tag="podcast_episodes"
)]
#[get("/settings")]
pub async fn get_settings(conn: Data<DbPool>) -> Result<HttpResponse, CustomError> {
    let settings = Setting::get_settings(conn.get().map_err(map_r2d2_error)?.deref_mut())?;
    println!("Settings: {:?}", settings);
    match settings {
        Some(settings) => Ok(HttpResponse::Ok().json(settings)),
        None => Err(CustomError::NotFound),
    }
}

#[utoipa::path(
context_path="/api/v1",
request_body=Setting,
responses(
(status = 200, description = "Updates the current settings")),
tag="settings"
)]
#[put("/settings")]
pub async fn update_settings(
    settings_service: Data<Mutex<SettingsService>>,
    settings: web::Json<Setting>,
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_admin() {
        return Err(CustomError::Forbidden);
    }

    let mut settings_service = settings_service.lock().ignore_poison();
    let settings = settings_service.update_settings(
        settings.into_inner(),
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;
    Ok(HttpResponse::Ok().json(settings))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Runs a cleanup of old episodes")),
tag="settings"
)]
#[put("/settings/runcleanup")]
pub async fn run_cleanup(
    pdservice: Data<Mutex<PodcastEpisodeService>>,
    settings_service: Data<Mutex<SettingsService>>,
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_admin() {
        return Err(CustomError::Forbidden);
    }
    let settings = settings_service
        .lock()
        .ignore_poison()
        .get_settings(conn.get().map_err(map_r2d2_error)?.deref_mut())?;
    match settings {
        Some(settings) => {
            pdservice.lock().ignore_poison().cleanup_old_episodes(
                settings.auto_cleanup_days,
                conn.get().map_err(map_r2d2_error)?.deref_mut(),
            );
            Ok(HttpResponse::Ok().finish())
        }
        None => {
            log::error!("Error getting settings");
            Err(CustomError::Unknown)
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Local,
    Online,
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the podcasts in opml format")),
tag="podcasts"
)]
#[get("/settings/opml/{type_of}")]
pub async fn get_opml(
    conn: Data<DbPool>,
    type_of: Path<Mode>,
    env_service: Data<Mutex<EnvironmentService>>,
) -> Result<HttpResponse, CustomError> {
    let env_service = env_service.lock().ignore_poison();
    let podcasts_found =
        Podcast::get_all_podcasts(conn.get().map_err(map_r2d2_error)?.deref_mut())?;

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".to_string())
        .build();
    let mut opml = XMLElement::new("opml");
    opml.add_attribute("version", "2.0");
    opml.add_child(add_header()).expect("TODO: panic message");
    opml.add_child(add_podcasts(
        podcasts_found,
        env_service,
        type_of.into_inner(),
    ))
    .map_err(|e| {
        log::error!("Error adding podcasts to opml: {}", e);
        CustomError::Unknown
    })?;

    xml.set_root_element(opml);

    let mut writer: Vec<u8> = Vec::new();
    xml.generate(&mut writer).unwrap();
    Ok(HttpResponse::Ok().body(writer))
}

fn add_header() -> XMLElement {
    let mut head = XMLElement::new("head");
    let mut title = XMLElement::new("title");
    title
        .add_text("PodFetch Feed Export".to_string())
        .expect("Error creating title");
    head.add_child(title).expect("TODO: panic message");
    let mut date_created = XMLElement::new("dateCreated");
    date_created
        .add_text(Local::now().to_rfc3339())
        .expect("Error creating dateCreated");

    head.add_child(date_created).expect("TODO: panic message");
    head
}

fn add_body() -> XMLElement {
    XMLElement::new("body")
}

fn add_podcasts(
    podcasts_found: Vec<Podcast>,
    env_service: MutexGuard<EnvironmentService>,
    type_of: Mode,
) -> XMLElement {
    let mut body = add_body();
    for podcast in podcasts_found {
        let mut outline = XMLElement::new("outline");
        if podcast.summary.is_some() {
            outline.add_attribute("text", &podcast.summary.unwrap());
        }
        outline.add_attribute("title", &podcast.name);
        outline.add_attribute("type", "rss");
        match type_of {
            Mode::Local => outline.add_attribute(
                "xmlUrl",
                &format!("{}rss/{}", &*env_service.get_server_url(), podcast.id),
            ),
            Mode::Online => outline.add_attribute("xmlUrl", &podcast.rssfeed),
        }
        body.add_child(outline).expect("TODO: panic message");
    }
    body
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Updates the name settings")),
tag="podcasts",
request_body=UpdateNameSettings
)]
#[put("/settings/name")]
pub async fn update_name(
    settings_service: Data<Mutex<SettingsService>>,
    update_information: web::Json<UpdateNameSettings>,
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_admin() {
        return Err(CustomError::Forbidden);
    }

    let mut settings_service = settings_service.lock().ignore_poison();

    let settings = settings_service.update_name(
        update_information.into_inner(),
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;
    Ok(HttpResponse::Ok().json(settings))
}

use crate::utils::error::{map_r2d2_error, CustomError};
use utoipa::ToSchema;

#[derive(Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNameSettings {
    pub use_existing_filename: bool,
    pub replace_invalid_characters: bool,
    pub replacement_strategy: ReplacementStrategy,
    pub episode_format: String,
    pub podcast_format: String,
    pub direct_paths: bool,
}

impl From<UpdateNameSettings> for Setting {
    fn from(val: UpdateNameSettings) -> Self {
        Setting {
            id: 0,
            auto_download: false,
            auto_update: false,
            use_existing_filename: val.use_existing_filename,
            replace_invalid_characters: val.replace_invalid_characters,
            replacement_strategy: val.replacement_strategy.to_string(),
            episode_format: val.episode_format,
            podcast_format: val.podcast_format,
            direct_paths: val.direct_paths,
            auto_cleanup_days: 0,
            auto_cleanup: false,
            podcast_prefill: 0,
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ReplacementStrategy {
    ReplaceWithDashAndUnderscore,
    Remove,
    ReplaceWithDash,
}

impl ToString for ReplacementStrategy {
    fn to_string(&self) -> String {
        match self {
            ReplacementStrategy::ReplaceWithDashAndUnderscore => {
                "replace-with-dash-and-underscore".to_string()
            }
            ReplacementStrategy::Remove => "remove".to_string(),
            ReplacementStrategy::ReplaceWithDash => "replace-with-dash".to_string(),
        }
    }
}

impl FromStr for ReplacementStrategy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "replace-with-dash-and-underscore" => {
                Ok(ReplacementStrategy::ReplaceWithDashAndUnderscore)
            }
            "remove" => Ok(ReplacementStrategy::Remove),
            "replace-with-dash" => Ok(ReplacementStrategy::ReplaceWithDash),
            _ => Err(()),
        }
    }
}
