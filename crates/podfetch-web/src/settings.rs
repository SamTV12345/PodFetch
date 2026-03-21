use chrono::Local;
use podfetch_domain::settings::{Setting, UpdateNameSettings};
use serde::Deserialize;
use std::fmt::Display;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

pub trait SettingsApplicationService {
    type Error;

    fn get_settings(&self) -> Result<Option<Setting>, Self::Error>;
    fn update_settings(&self, settings: Setting) -> Result<Setting, Self::Error>;
    fn update_name(&self, update: UpdateNameSettings) -> Result<Setting, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsControllerError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Service(E),
}

pub fn get_settings<S>(
    service: &S,
    is_admin: bool,
) -> Result<Setting, SettingsControllerError<S::Error>>
where
    S: SettingsApplicationService,
    S::Error: Display,
{
    if !is_admin {
        return Err(SettingsControllerError::Forbidden);
    }

    service
        .get_settings()
        .map_err(SettingsControllerError::Service)?
        .ok_or(SettingsControllerError::NotFound)
}

pub fn update_settings<S>(
    service: &S,
    is_admin: bool,
    settings: Setting,
) -> Result<Setting, SettingsControllerError<S::Error>>
where
    S: SettingsApplicationService,
    S::Error: Display,
{
    if !is_admin {
        return Err(SettingsControllerError::Forbidden);
    }

    service
        .update_settings(settings)
        .map_err(SettingsControllerError::Service)
}

pub fn update_name<S>(
    service: &S,
    is_admin: bool,
    update: UpdateNameSettings,
) -> Result<Setting, SettingsControllerError<S::Error>>
where
    S: SettingsApplicationService,
    S::Error: Display,
{
    if !is_admin {
        return Err(SettingsControllerError::Forbidden);
    }

    service
        .update_name(update)
        .map_err(SettingsControllerError::Service)
}

pub fn cleanup_settings<S>(
    service: &S,
    is_admin: bool,
) -> Result<Setting, SettingsControllerError<S::Error>>
where
    S: SettingsApplicationService,
    S::Error: Display,
{
    get_settings(service, is_admin)
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Local,
    Online,
}

impl From<String> for Mode {
    fn from(value: String) -> Self {
        match value.as_str() {
            "local" => Mode::Local,
            "online" => Mode::Online,
            _ => Mode::Local,
        }
    }
}

#[derive(Clone)]
pub struct OpmlPodcast {
    pub id: i32,
    pub name: String,
    pub summary: Option<String>,
    pub rssfeed: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OpmlError {
    #[error("api key required")]
    ApiKeyRequired,
    #[error("xml error: {0}")]
    Xml(String),
}

pub fn build_opml(
    podcasts: Vec<OpmlPodcast>,
    type_of: Mode,
    requester_api_key: Option<&str>,
    any_auth_enabled: bool,
    server_url: &str,
) -> Result<String, OpmlError> {
    if any_auth_enabled && requester_api_key.is_none() {
        return Err(OpmlError::ApiKeyRequired);
    }

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".to_string())
        .build();
    let mut opml = XMLElement::new("opml");
    opml.add_attribute("version", "2.0");
    opml.add_child(add_header())
        .map_err(|error| OpmlError::Xml(error.to_string()))?;
    opml.add_child(add_podcasts(
        podcasts,
        type_of,
        requester_api_key,
        server_url,
    ))
    .map_err(|error| OpmlError::Xml(error.to_string()))?;
    xml.set_root_element(opml);

    let mut writer: Vec<u8> = Vec::new();
    xml.generate(&mut writer)
        .map_err(|error| OpmlError::Xml(error.to_string()))?;
    String::from_utf8(writer).map_err(|error| OpmlError::Xml(error.to_string()))
}

fn add_header() -> XMLElement {
    let mut head = XMLElement::new("head");
    let mut title = XMLElement::new("title");
    title
        .add_text("PodFetch Feed Export".to_string())
        .expect("title should be valid xml");
    head.add_child(title).expect("title should be attached");
    let mut date_created = XMLElement::new("dateCreated");
    date_created
        .add_text(Local::now().to_rfc3339())
        .expect("date should be valid xml");
    head.add_child(date_created)
        .expect("date should be attached");
    head
}

fn add_podcasts(
    podcasts: Vec<OpmlPodcast>,
    type_of: Mode,
    requester_api_key: Option<&str>,
    server_url: &str,
) -> XMLElement {
    let mut body = XMLElement::new("body");
    for podcast in podcasts {
        let mut outline = XMLElement::new("outline");
        if let Some(summary) = podcast.summary {
            outline.add_attribute("text", &summary);
        }
        outline.add_attribute("title", &podcast.name);
        outline.add_attribute("type", "rss");
        match type_of {
            Mode::Local => {
                let mut local_url = format!("{}rss/{}", server_url, podcast.id);
                if let Some(api_key) = requester_api_key {
                    local_url = format!("{local_url}?apiKey={api_key}");
                }
                outline.add_attribute("xmlUrl", &local_url);
            }
            Mode::Online => outline.add_attribute("xmlUrl", &podcast.rssfeed),
        }
        body.add_child(outline).expect("outline should be attached");
    }
    body
}
