//! Wiring of the media library: the adapters for its ports (catalog,
//! settings, transcripts, ffprobe), the translation of catalog entities
//! into its view, and accessors for code that has no injected
//! dependencies yet.
//!
//! Transitional: the episode use case is still a set of associated
//! functions and reaches the media library through [`services`]. The
//! catalog moves into its own crate in migration step 5.3; the adapters
//! here then call its API.

use crate::podcast_settings::PodcastSetting;
use crate::services::file::service::{FileService, naming_rules};
use crate::services::podcast::service::PodcastService;
use crate::services::podcast_episode_chapter::service::PodcastEpisodeChapterService;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::settings::service::SettingsService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase;
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::db::Database;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use common_infrastructure::http::get_http_client;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_media::audio::Chapter;
use podfetch_media::catalog::{Catalog, MediaEpisode, MediaPodcast};
use podfetch_media::download::{DownloadListener, Downloader, DurationProbe};
use podfetch_media::files::{EpisodeFile, EpisodeFileService};
use podfetch_media::nfo::{NfoFormat, NfoWriter};
use podfetch_media::persistence::{DieselEpisodeFileRepository, DieselSponsorblockRepository};
use podfetch_media::rescan::EpisodeRescanner;
use podfetch_media::settings::{MediaSettings, MediaSettingsSource};
use podfetch_media::sponsorblock::{SponsorblockClient, SponsorblockService, SponsorblockSwitch};
use podfetch_persistence::db::database;
use podfetch_persistence::podcast::PodcastEntity;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// The media library's services, wired to this application.
#[derive(Clone)]
pub struct MediaServices {
    pub files: Arc<EpisodeFileService>,
    pub sponsorblock: Arc<SponsorblockService>,
    pub nfo: Arc<NfoWriter>,
    pub downloader: Arc<Downloader>,
    pub rescanner: Arc<EpisodeRescanner>,
}

impl MediaServices {
    pub fn build(
        database: Database,
        environment: Arc<EnvironmentService>,
        settings: Arc<SettingsService>,
    ) -> Self {
        let catalog: Arc<dyn Catalog> = Arc::new(CatalogAdapter);
        let media_settings: Arc<dyn MediaSettingsSource> =
            Arc::new(SettingsAdapter(settings.clone()));
        let files = Arc::new(EpisodeFileService::new(Arc::new(
            DieselEpisodeFileRepository::new(database.clone()),
        )));
        let sponsorblock = Arc::new(SponsorblockService::new(
            Arc::new(DieselSponsorblockRepository::new(database)),
            SponsorblockClient::new(get_http_client(&environment)),
            Arc::new(SettingsSponsorblockSwitch(settings)),
        ));
        let nfo = Arc::new(NfoWriter::new(
            catalog.clone(),
            media_settings.clone(),
            environment.default_file_handler.clone(),
        ));
        let downloader = Arc::new(Downloader::new(
            environment,
            catalog,
            media_settings,
            files.clone(),
            sponsorblock.clone(),
            nfo.clone(),
            Arc::new(TranscriptsAfterDownload),
            Arc::new(Ffprobe),
        ));
        let rescanner = Arc::new(EpisodeRescanner::new(downloader.clone()));
        Self {
            files,
            sponsorblock,
            nfo,
            downloader,
            rescanner,
        }
    }
}

/// The media services for code without injected dependencies.
pub fn services() -> MediaServices {
    MediaServices::build(
        database(),
        Arc::new(ENVIRONMENT_SERVICE.clone()),
        Arc::new(SettingsService::shared()),
    )
}

/// The file state of the episodes.
pub fn episode_files() -> EpisodeFileService {
    EpisodeFileService::new(Arc::new(DieselEpisodeFileRepository::new(database())))
}

/// The cover file name in effect for the podcast; "image" if the settings
/// can't be read.
pub fn cover_filename(podcast_id: Uuid) -> String {
    SettingsAdapter(Arc::new(SettingsService::shared()))
        .for_podcast(&podcast_id.to_string())
        .map_or_else(
            |err| {
                tracing::warn!("Could not resolve the cover file name: {err}");
                "image".to_string()
            },
            |settings| settings.cover_filename,
        )
}

// ── Translation ─────────────────────────────────────────────────────────

pub fn media_episode(episode: &PodcastEpisodeEntity) -> MediaEpisode {
    MediaEpisode {
        id: episode.id.clone(),
        episode_id: episode.episode_id.clone(),
        podcast_id: episode.podcast_id.clone(),
        name: episode.name.clone(),
        url: episode.url.clone(),
        image_url: episode.image_url.clone(),
        date_of_recording: episode.date_of_recording.clone(),
        description: episode.description.clone(),
        guid: episode.guid.clone(),
        total_time: episode.total_time,
        episode_numbering_processed: episode.episode_numbering_processed,
        youtube_video_id: episode.youtube_video_id.clone(),
        file: EpisodeFile {
            episode_id: episode.id.clone(),
            download_time: episode.download_time,
            file_episode_path: episode.file_episode_path.clone(),
            file_image_path: episode.file_image_path.clone(),
            download_location: episode.download_location.clone(),
            deleted: episode.deleted,
        },
    }
}

pub fn media_podcast(podcast: &PodcastEntity) -> MediaPodcast {
    MediaPodcast {
        id: podcast.id.clone(),
        directory_id: podcast.directory_id.clone(),
        name: podcast.name.clone(),
        directory_name: podcast.directory_name.clone(),
        image_url: podcast.image_url.clone(),
        original_image_url: podcast.original_image_url.clone(),
        author: podcast.author.clone(),
        keywords: podcast.keywords.clone(),
        language: podcast.language.clone(),
        summary: podcast.summary.clone(),
        guid: podcast.guid.clone(),
        download_location: podcast.download_location.clone(),
    }
}

fn parse_id(id: &str) -> Result<Uuid, CustomError> {
    Uuid::parse_str(id).map_err(|_| CustomErrorInner::NotFound(ErrorSeverity::Warning).into())
}

// ── Ports ───────────────────────────────────────────────────────────────

/// The catalog as the media library reaches it.
pub struct CatalogAdapter;

impl Catalog for CatalogAdapter {
    fn podcast(&self, podcast_id: &str) -> Result<MediaPodcast, CustomError> {
        PodcastService::get_podcast(parse_id(podcast_id)?).map(|p| media_podcast(&p))
    }

    fn episodes_of_podcast(&self, podcast_id: &str) -> Result<Vec<MediaEpisode>, CustomError> {
        Ok(
            PodcastEpisodeUseCase::get_episodes_by_podcast_id(parse_id(podcast_id)?)?
                .iter()
                .map(media_episode)
                .collect(),
        )
    }

    fn position_of_episode(
        &self,
        podcast_id: &str,
        date_of_recording: &str,
    ) -> Result<usize, CustomError> {
        PodcastEpisodeUseCase::get_position_of_episode(date_of_recording, parse_id(podcast_id)?)
    }

    fn episode_count(&self, podcast_id: &str) -> Result<usize, CustomError> {
        PodcastEpisodeUseCase::get_total_episode_count(parse_id(podcast_id)?)
    }

    fn track_number(&self, podcast_id: &str, date_of_recording: &str) -> Result<i64, CustomError> {
        PodcastEpisodeUseCase::get_track_number_for_episode(
            parse_id(podcast_id)?,
            date_of_recording,
        )
    }

    fn podcast_cover_downloaded(&self, directory_id: &str) -> bool {
        FileService::check_if_podcast_main_image_downloaded(directory_id)
    }

    fn set_numbered(&self, episode: &MediaEpisode, numbered: bool) -> Result<(), CustomError> {
        PodcastEpisodeUseCase::update_episode_numbering_processed(numbered, &episode.episode_id)
    }

    fn set_duration(&self, episode: &MediaEpisode, seconds: i32) -> Result<(), CustomError> {
        PodcastEpisodeUseCase::update_total_time(&episode.episode_id, seconds)
    }

    fn save_chapter(&self, episode: &MediaEpisode, chapter: &Chapter) -> Result<(), CustomError> {
        PodcastEpisodeChapterService::default_service().save_chapter(chapter, &episode.id)
    }

    fn set_podcast_image(
        &self,
        podcast: &MediaPodcast,
        image_url: &str,
    ) -> Result<(), CustomError> {
        PodcastEpisodeUseCase::update_podcast_image(&podcast.directory_id, image_url)
    }
}

/// The settings the media library works by, for one podcast.
pub struct SettingsAdapter(pub Arc<SettingsService>);

impl MediaSettingsSource for SettingsAdapter {
    fn for_podcast(&self, podcast_id: &str) -> Result<MediaSettings, CustomError> {
        let global = self.0.get_settings()?.ok_or_else(|| {
            tracing::error!("Global settings are missing");
            CustomError::from(CustomErrorInner::Unknown(ErrorSeverity::Error))
        })?;
        let own = PodcastSettingsService::get_settings_for_podcast(parse_id(podcast_id)?)?;
        Ok(resolve_media_settings(&global, own.as_ref()))
    }
}

/// How the instance-wide and the podcast's own settings combine. Naming,
/// cover, NFO and "one cover" follow the podcast's settings only when they
/// are activated; numbering is on when either layer turns it on; the
/// existing-file-name, direct-path and transcoding options are
/// instance-wide; auto-transcription is per podcast.
pub fn resolve_media_settings(
    global: &crate::settings::Setting,
    own: Option<&PodcastSetting>,
) -> MediaSettings {
    let activated = own.filter(|s| s.activated);
    let cover_filename = activated.map_or(&global.cover_filename, |s| &s.cover_filename);
    let cover_filename = match cover_filename.trim() {
        "" => "image".to_string(),
        name => name.to_string(),
    };
    let nfo_format = activated.map_or(&global.nfo_format, |s| &s.nfo_format);
    MediaSettings {
        naming: naming_rules(global, own),
        use_existing_filename: global.use_existing_filename,
        direct_paths: global.direct_paths,
        auto_transcode_opus: global.auto_transcode_opus,
        use_one_cover: podfetch_media::rules::use_one_cover_from(
            own.map(|s| (s.activated, s.use_one_cover_for_all_episodes)),
            global.use_one_cover_for_all_episodes,
        ),
        numbering: podfetch_media::rules::numbering_from(
            own.map(|s| s.episode_numbering),
            Some(global.episode_numbering),
        ),
        cover_filename,
        nfo_format: NfoFormat::from_str(nfo_format).unwrap_or_default(),
        auto_transcribe: own.is_some_and(|s| s.auto_transcribe),
    }
}

/// SponsorBlock is on unless the admin turned it off in the settings.
pub struct SettingsSponsorblockSwitch(pub Arc<SettingsService>);

impl SponsorblockSwitch for SettingsSponsorblockSwitch {
    fn enabled(&self) -> Result<bool, CustomError> {
        Ok(self
            .0
            .get_settings()?
            .is_none_or(|settings| settings.sponsorblock_enabled))
    }
}

/// Feed transcripts and auto-transcription after a download. Never fails
/// the download.
pub struct TranscriptsAfterDownload;

impl DownloadListener for TranscriptsAfterDownload {
    fn downloaded(&self, episode: &MediaEpisode, file_path: &str, settings: &MediaSettings) {
        let Ok(episode_uuid) = parse_id(&episode.id) else {
            return;
        };
        let entity = match PodcastEpisodeUseCase::get_podcast_episode_by_internal_id(episode_uuid) {
            Ok(Some(entity)) => entity,
            Ok(None) => return,
            Err(err) => {
                tracing::error!(
                    "Could not load episode {} for transcripts: {err}",
                    episode.id
                );
                return;
            }
        };
        let transcripts =
            crate::services::transcript::service::TranscriptService::default_service();
        if let Err(err) = transcripts.process_pending_after_download(&entity, file_path) {
            tracing::error!(
                "Error processing transcripts for episode {}: {err}",
                episode.id
            );
        }

        // Enqueue a generation job when this podcast opted in and the episode
        // doesn't already have a usable (feed) transcript in flight.
        if ENVIRONMENT_SERVICE.transcription_config.is_some()
            && settings.auto_transcribe
            && transcripts
                .needs_generated_transcript(episode_uuid)
                .unwrap_or(false)
            && let Err(err) = transcripts.enqueue_job(episode_uuid)
        {
            tracing::error!("Error enqueuing transcription job: {err}");
        }
    }
}

/// Measures durations with ffprobe.
pub struct Ffprobe;

impl DurationProbe for Ffprobe {
    fn duration(&self, file_path: &str) -> Result<f64, CustomError> {
        podfetch_audiobooks::services::audio_probe::probe_audio_file(std::path::Path::new(
            file_path,
        ))
        .map(|probed| probed.duration)
        .map_err(|err| CustomErrorInner::Conflict(err.to_string(), ErrorSeverity::Warning).into())
    }
}
