use crate::models::podcasts::Podcast;
use crate::models::settings::Setting;
use crate::models::user::User;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::settings_service::SettingsService;
use crate::utils::url_builder::resolve_server_url_from_headers;
use axum::extract::Path;
use axum::http::HeaderMap;
use axum::http::Response;
use axum::{Extension, Json};
use chrono::Local;
use file_format::FileFormat;
use reqwest::StatusCode;
use std::fmt::Display;
use std::str::FromStr;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

#[utoipa::path(
get,
path="/settings",
responses(
(status = 200, description = "Gets the current settings", body=Setting)),
tag="podcast_episodes"
)]
pub async fn get_settings(
    Extension(requester): Extension<User>,
) -> Result<Json<Setting>, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }
    let settings = Setting::get_settings()?;
    match settings {
        Some(settings) => Ok(Json(settings)),
        None => Err(CustomErrorInner::NotFound(Debug).into()),
    }
}

#[utoipa::path(
    post,
    path="/settings/rescan-episodes",
    responses(
(status = 200, description = "Rescans all episodes for metadata")),
    tag="podcast_episodes"
)]
pub async fn rescan_episodes(Extension(requester): Extension<User>) -> Result<(), CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }
    let mut current_episode_id = 0;
    let first_page = PodcastEpisode::get_nth_page_of_podcast_episodes(0)?;
    while !first_page.is_empty() {
        let episodes = PodcastEpisode::get_nth_page_of_podcast_episodes(current_episode_id)?;
        if episodes.is_empty() {
            break;
        }
        for episode in &episodes {
            if let Some(filepath_to_episode) = &episode.file_episode_path {
                let detected_file = FileFormat::from_file(filepath_to_episode).unwrap();
                match detected_file {
                    FileFormat::Mpeg12AudioLayer3
                    | FileFormat::Mpeg12AudioLayer2
                    | FileFormat::AppleItunesAudio
                    | FileFormat::Id3v2
                    | FileFormat::WaveformAudio => {
                        let chapters = DownloadService::read_chapters_from_mp3(filepath_to_episode);
                        if let Err(err) = chapters {
                            log::error!(
                                "Error while reading chapters for episode {}: {}",
                                episode.id,
                                err
                            );
                            continue;
                        }
                        let chapters = chapters.expect("Chapters should be available here");
                        log::info!("Inserting chapters for episode {}", episode.name);
                        for chapter in chapters {
                            let res = PodcastEpisodeChapter::save_chapter(&chapter, episode);
                            if let Err(err) = res {
                                log::error!(
                                    "Error while saving chapter for episode {}: {}",
                                    episode.id,
                                    err
                                );
                            }
                        }
                    }
                    FileFormat::Mpeg4Part14 | FileFormat::Mpeg4Part14Audio => {
                        let chapters = DownloadService::read_chapters_from_mp4(filepath_to_episode);
                        for chapter in chapters {
                            let res = PodcastEpisodeChapter::save_chapter(&chapter, episode);
                            if let Err(err) = res {
                                log::error!(
                                    "Error while saving chapter for episode {}: {}",
                                    episode.id,
                                    err
                                );
                            }
                        }
                    }
                    _ => {
                        log::error!("File format not supported: {detected_file:?}");
                        return Err(CustomErrorInner::Conflict(
                            "File format not supported".to_string(),
                            ErrorSeverity::Error,
                        )
                        .into());
                    }
                }
            }
        }

        if episodes.is_empty() {
            break;
        }

        current_episode_id = episodes[episodes.len() - 1].id;
    }

    Ok(())
}

#[utoipa::path(
put,
path="/settings",
request_body=Setting,
responses(
(status = 200, description = "Updates the current settings", body = Setting)),
tag="settings"
)]
pub async fn update_settings(
    Extension(requester): Extension<User>,
    Json(settings): Json<Setting>,
) -> Result<Json<Setting>, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }
    let settings = SettingsService::update_settings(settings)?;
    Ok(Json(settings))
}

#[utoipa::path(
put,
path="/settings/runcleanup",
responses(
(status = 200, description = "Runs a cleanup of old episodes")),
tag="settings"
)]
pub async fn run_cleanup(requester: Extension<User>) -> Result<StatusCode, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }
    let settings = SettingsService::get_settings()?;
    match settings {
        Some(settings) => {
            PodcastEpisodeService::cleanup_old_episodes(settings.auto_cleanup_days);
            Ok(StatusCode::OK)
        }
        None => {
            log::error!("Error getting settings");
            Err(CustomErrorInner::Unknown(Error).into())
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Local,
    Online,
}

impl From<String> for Mode {
    fn from(val: String) -> Self {
        match val.as_str() {
            "local" => Mode::Local,
            "online" => Mode::Online,
            _ => Mode::Local,
        }
    }
}

#[utoipa::path(
get,
path="/settings/opml/{type_of}",
responses(
(status = 200, description = "Gets the podcasts in opml format", body = String)),
tag="settings"
)]
pub async fn get_opml(
    Extension(requester): Extension<User>,
    Path(type_of): Path<String>,
    headers: HeaderMap,
) -> Result<Response<String>, CustomError> {
    if ENVIRONMENT_SERVICE.any_auth_enabled && requester.api_key.is_none() {
        return Err(CustomErrorInner::UnAuthorized(
            "Please generate an api key".to_string(),
            Critical,
        )
        .into());
    }

    let podcasts_found = Podcast::get_all_podcasts()?;

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".to_string())
        .build();
    let mut opml = XMLElement::new("opml");
    opml.add_attribute("version", "2.0");
    opml.add_child(add_header()).expect("TODO: panic message");
    let server_url = resolve_server_url_from_headers(&headers);
    opml.add_child(add_podcasts(
        podcasts_found,
        Mode::from(type_of),
        &requester,
        &server_url,
    ))
    .map_err(|e| {
        log::error!("Error adding podcasts to opml: {e}");
        Into::<CustomError>::into(CustomErrorInner::Unknown(Error))
    })?;

    xml.set_root_element(opml);

    let mut writer: Vec<u8> = Vec::new();
    xml.generate(&mut writer).unwrap();
    let response = Response::builder()
        .header("Content-Type", "application/xml")
        .body(String::from_utf8(writer).unwrap())
        .unwrap();
    Ok(response)
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
    requester: &User,
    server_url: &str,
) -> XMLElement {
    let mut body = add_body();
    for podcast in podcasts_found {
        let mut outline = XMLElement::new("outline");
        if let Some(summary) = podcast.summary {
            outline.add_attribute("text", &summary);
        }
        outline.add_attribute("title", &podcast.name);
        outline.add_attribute("type", "rss");
        match type_of {
            Mode::Local => {
                let mut local_url = format!("{}rss/{}", server_url, podcast.id);

                if let Some(api_key) = &requester.api_key {
                    local_url = format!("{local_url}?apiKey={api_key}");
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
put,
path="/settings/name",
responses(
(status = 200, description = "Updates the name settings", body = Setting)),
tag="settings",
request_body=UpdateNameSettings
)]
pub async fn update_name(
    Extension(requester): Extension<User>,
    Json(update_information): Json<UpdateNameSettings>,
) -> Result<Json<Setting>, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    let settings = SettingsService::update_name(update_information)?;
    Ok(Json(settings))
}

use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcast_episode_chapter::PodcastEpisodeChapter;
use crate::service::download_service::DownloadService;
use crate::utils::error::ErrorSeverity::{Critical, Debug, Error, Warning};
use crate::utils::error::{CustomError, CustomErrorInner, ErrorSeverity};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

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
        write!(f, "{str}")
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

pub fn get_settings_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_settings))
        .routes(routes!(update_settings))
        .routes(routes!(run_cleanup))
        .routes(routes!(get_opml))
        .routes(routes!(update_name))
        .routes(routes!(rescan_episodes))
}
