use crate::models::podcasts::Podcast;
use crate::models::settings::Setting;
use crate::models::user::User;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::settings_service::SettingsService;
use actix_web::web::{Path, ReqData};
use actix_web::{get, put};
use actix_web::{web, HttpResponse};
use chrono::Local;
use std::fmt::Display;
use std::str::FromStr;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the current settings")),
tag="podcast_episodes"
)]
#[get("/settings")]
pub async fn get_settings(
    requester: web::ReqData<User>,
) -> Result<HttpResponse, CustomError> {
    if !requester.is_admin() {
        return Err(CustomError::Forbidden);
    }
    let settings = Setting::get_settings()?;
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
    settings: web::Json<Setting>,
    requester: ReqData<User>,
) -> Result<HttpResponse, CustomError> {
    if !requester.is_admin() {
        return Err(CustomError::Forbidden);
    }
    let settings = SettingsService::update_settings(settings.into_inner())?;
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
    requester: web::ReqData<User>,
) -> Result<HttpResponse, CustomError> {
    if !requester.is_admin() {
        return Err(CustomError::Forbidden);
    }
    let settings = SettingsService::get_settings()?;
    match settings {
        Some(settings) => {
            PodcastEpisodeService::cleanup_old_episodes(settings.auto_cleanup_days);
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
    requester: ReqData<User>,
    type_of: Path<Mode>,
) -> Result<HttpResponse, CustomError> {
    let podcasts_found = Podcast::get_all_podcasts()?;

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".to_string())
        .build();
    let mut opml = XMLElement::new("opml");
    opml.add_attribute("version", "2.0");
    opml.add_child(add_header()).expect("TODO: panic message");
    opml.add_child(add_podcasts(
        podcasts_found,
        type_of.into_inner(),
        requester,
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
    type_of: Mode,
    requester: ReqData<User>,
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
            Mode::Local => {
                let mut local_url = format!(
                    "{}rss/{}",
                    &*ENVIRONMENT_SERVICE.get_server_url(),
                    podcast.id
                );

                    if let Some(api_key) = &requester.api_key {
                        local_url = format!("{}?apiKey={}", local_url, api_key);
                    }

                outline.add_attribute("xmlUrl", &local_url)
            }
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
    update_information: web::Json<UpdateNameSettings>,
    requester: web::ReqData<User>,
) -> Result<HttpResponse, CustomError> {
    if !requester.is_admin() {
        return Err(CustomError::Forbidden);
    }

    let settings = SettingsService::update_name(update_information.into_inner())?;
    Ok(HttpResponse::Ok().json(settings))
}

use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::utils::error::CustomError;
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

#[derive(Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReplacementStrategy {
    ReplaceWithDashAndUnderscore,
    Remove,
    ReplaceWithDash,
}

impl Display for ReplacementStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ReplacementStrategy::ReplaceWithDashAndUnderscore => {
                "replace-with-dash-and-underscore".to_string()
            }
            ReplacementStrategy::Remove => "remove".to_string(),
            ReplacementStrategy::ReplaceWithDash => "replace-with-dash".to_string(),
        };
        write!(f, "{}", str)
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
