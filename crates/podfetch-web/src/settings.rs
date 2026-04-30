use chrono::Local;
use serde::Deserialize;
use std::fmt::Display;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, utoipa::ToSchema, Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub id: i32,
    pub auto_download: bool,
    pub auto_update: bool,
    pub auto_cleanup: bool,
    pub auto_cleanup_days: i32,
    pub podcast_prefill: i32,
    pub replace_invalid_characters: bool,
    pub use_existing_filename: bool,
    pub replacement_strategy: String,
    pub episode_format: String,
    pub podcast_format: String,
    pub direct_paths: bool,
    pub auto_transcode_opus: bool,
}

impl From<podfetch_domain::settings::Setting> for Setting {
    fn from(value: podfetch_domain::settings::Setting) -> Self {
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
        }
    }
}

impl From<Setting> for podfetch_domain::settings::Setting {
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
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReplacementStrategy {
    ReplaceWithDashAndUnderscore,
    Remove,
    ReplaceWithDash,
}

impl std::fmt::Display for ReplacementStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            ReplacementStrategy::ReplaceWithDashAndUnderscore => "replace-with-dash-and-underscore",
            ReplacementStrategy::Remove => "remove",
            ReplacementStrategy::ReplaceWithDash => "replace-with-dash",
        };
        write!(f, "{value}")
    }
}

impl From<ReplacementStrategy> for podfetch_domain::settings::ReplacementStrategy {
    fn from(value: ReplacementStrategy) -> Self {
        match value {
            ReplacementStrategy::ReplaceWithDashAndUnderscore => {
                podfetch_domain::settings::ReplacementStrategy::ReplaceWithDashAndUnderscore
            }
            ReplacementStrategy::Remove => podfetch_domain::settings::ReplacementStrategy::Remove,
            ReplacementStrategy::ReplaceWithDash => {
                podfetch_domain::settings::ReplacementStrategy::ReplaceWithDash
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNameSettings {
    pub use_existing_filename: bool,
    pub replace_invalid_characters: bool,
    pub replacement_strategy: ReplacementStrategy,
    pub episode_format: String,
    pub podcast_format: String,
    pub direct_paths: bool,
}

impl UpdateNameSettings {
    pub fn to_domain(self) -> podfetch_domain::settings::UpdateNameSettings {
        podfetch_domain::settings::UpdateNameSettings {
            use_existing_filename: self.use_existing_filename,
            replace_invalid_characters: self.replace_invalid_characters,
            replacement_strategy: self.replacement_strategy.into(),
            episode_format: self.episode_format,
            podcast_format: self.podcast_format,
            direct_paths: self.direct_paths,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertPodcastEpisodeChapter {
    pub episode_id: i32,
    pub title: String,
    pub start_time: i32,
    pub end_time: i32,
    pub href: Option<String>,
    pub image: Option<String>,
}

impl From<UpsertPodcastEpisodeChapter>
    for podfetch_domain::podcast_episode_chapter::UpsertPodcastEpisodeChapter
{
    fn from(value: UpsertPodcastEpisodeChapter) -> Self {
        Self {
            episode_id: value.episode_id,
            title: value.title,
            start_time: value.start_time,
            end_time: value.end_time,
            href: value.href,
            image: value.image,
        }
    }
}

pub trait SettingsApplicationService {
    type Error;

    fn get_settings(&self) -> Result<Option<Setting>, Self::Error>;
    fn update_settings(&self, settings: Setting) -> Result<Setting, Self::Error>;
    fn update_name(&self, update: UpdateNameSettings) -> Result<Setting, Self::Error>;
}

/// Represents a podcast episode with file path for chapter scanning
pub struct EpisodeWithPath {
    pub id: i32,
    pub name: String,
    pub file_path: String,
}

/// Parsed chapter from media file
#[derive(Debug, Clone)]
pub struct ParsedChapter {
    pub title: String,
    pub start_time_seconds: i32,
    pub end_time_seconds: i32,
    pub href: Option<String>,
    pub image: Option<String>,
}

impl ParsedChapter {
    pub fn to_upsert(&self, episode_id: i32) -> UpsertPodcastEpisodeChapter {
        UpsertPodcastEpisodeChapter {
            episode_id,
            title: self.title.clone(),
            start_time: self.start_time_seconds,
            end_time: self.end_time_seconds,
            href: self.href.clone(),
            image: self.image.clone(),
        }
    }
}

/// File format for media files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaFileFormat {
    Mp3,
    Mp4,
    Unsupported,
}

/// Service trait for episode scanning operations
pub trait EpisodeScanService {
    type Error: Display;

    /// Get paginated episodes with file paths, starting after the given ID
    fn get_episodes_with_paths_after(
        &self,
        last_id: i32,
        limit: usize,
    ) -> Result<Vec<EpisodeWithPath>, Self::Error>;

    /// Detect the file format of a media file
    fn detect_file_format(&self, path: &str) -> Result<MediaFileFormat, Self::Error>;

    /// Read chapters from an MP3 file
    fn read_chapters_mp3(&self, path: &str) -> Result<Vec<ParsedChapter>, Self::Error>;

    /// Read chapters from an MP4 file
    fn read_chapters_mp4(&self, path: &str) -> Vec<ParsedChapter>;

    /// Save a chapter for an episode
    fn save_chapter(&self, chapter: UpsertPodcastEpisodeChapter) -> Result<(), Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum RescanError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("unsupported file format")]
    UnsupportedFormat,
    #[error("{0}")]
    Service(E),
}

/// Rescan all episodes for chapters and metadata
pub fn rescan_episodes<S>(service: &S, is_admin: bool) -> Result<RescanStats, RescanError<S::Error>>
where
    S: EpisodeScanService,
{
    if !is_admin {
        return Err(RescanError::Forbidden);
    }

    let mut stats = RescanStats::default();
    let mut last_id = 0;
    const PAGE_SIZE: usize = 100;

    loop {
        let episodes = service
            .get_episodes_with_paths_after(last_id, PAGE_SIZE)
            .map_err(RescanError::Service)?;

        if episodes.is_empty() {
            break;
        }

        for episode in &episodes {
            stats.episodes_scanned += 1;

            let format = match service.detect_file_format(&episode.file_path) {
                Ok(f) => f,
                Err(e) => {
                    tracing::error!(
                        "Error detecting file format for episode {}: {}",
                        episode.id,
                        e
                    );
                    stats.errors += 1;
                    continue;
                }
            };

            let chapters = match format {
                MediaFileFormat::Mp3 => match service.read_chapters_mp3(&episode.file_path) {
                    Ok(chapters) => chapters,
                    Err(e) => {
                        tracing::error!("Error reading chapters for episode {}: {}", episode.id, e);
                        stats.errors += 1;
                        continue;
                    }
                },
                MediaFileFormat::Mp4 => service.read_chapters_mp4(&episode.file_path),
                MediaFileFormat::Unsupported => {
                    tracing::debug!("Unsupported format for episode {}", episode.id);
                    stats.skipped += 1;
                    continue;
                }
            };

            tracing::info!(
                "Found {} chapters for episode {}",
                chapters.len(),
                episode.name
            );

            for chapter in chapters {
                let upsert = chapter.to_upsert(episode.id);
                if let Err(e) = service.save_chapter(upsert) {
                    tracing::error!("Error saving chapter for episode {}: {}", episode.id, e);
                    stats.errors += 1;
                } else {
                    stats.chapters_saved += 1;
                }
            }
        }

        last_id = episodes.last().map(|e| e.id).unwrap_or(last_id);
    }

    Ok(stats)
}

/// Statistics from episode rescan operation
#[derive(Debug, Default, Clone)]
pub struct RescanStats {
    pub episodes_scanned: usize,
    pub chapters_saved: usize,
    pub skipped: usize,
    pub errors: usize,
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
