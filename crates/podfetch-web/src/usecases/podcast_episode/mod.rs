use crate::notification::Notification;
use crate::server::ChatServerHandle;
use crate::services::download::service::DownloadService;
use crate::services::episode_triage::service::EpisodeTriageService;
use crate::services::favorite_podcast_episode::service::FavoritePodcastEpisodeService;
use crate::services::file::service::FileService;
use crate::services::notification::service::NotificationService;
use crate::services::playlist::service::PlaylistService;
use crate::services::podcast::metadata::PodcastBuilder;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::settings::service::SettingsService;
use crate::services::transcript::service::{FeedTranscriptTag, TranscriptService};
use chrono::{DateTime, FixedOffset, Utc};
use common_infrastructure::config::FileHandlerType;
use common_infrastructure::config::TELEGRAM_API_ENABLED;
use common_infrastructure::config::is_env_var_present_and_true;
use common_infrastructure::error::ErrorSeverity::{Critical, Warning};
use common_infrastructure::error::{
    CustomError, CustomErrorInner, ErrorSeverity, map_db_error, map_reqwest_error,
};
use common_infrastructure::http::COMMON_USER_AGENT;
use common_infrastructure::http::get_sync_client;
use common_infrastructure::mutex::LockResultExt;
use common_infrastructure::retry::do_retry;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use common_infrastructure::telegram::send_new_episode_notification;
use common_infrastructure::time::opt_or_empty_string;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_domain::podcast_episode::{NewPodcastEpisode, PodcastEpisodeRepository};
use podfetch_domain::user::User;
use podfetch_persistence::db::database;
use podfetch_persistence::db::get_connection;
use podfetch_persistence::podcast::PodcastEntity as Podcast;
use podfetch_persistence::podcast_episode::DieselPodcastEpisodeRepository;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_storage::{FileHandleWrapper, FileRequest};
use reqwest::header::{ACCEPT, HeaderMap};
use reqwest::redirect::Policy;
use uuid::Uuid;
use rss::{Channel, Guid, Item};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::io::Error;
use std::path::Path;
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};
use std::thread;
use url::Url;

pub struct PodcastEpisodeUseCase;
static IN_PROGRESS_DOWNLOADS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

type PodcastEpisodeWithFavorited = Result<
    Vec<(
        PodcastEpisode,
        Option<podfetch_persistence::episode::EpisodeEntity>,
        Option<podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode>,
    )>,
    CustomError,
>;

struct InProgressDownloadGuard {
    episode_id: String,
}

impl Drop for InProgressDownloadGuard {
    fn drop(&mut self) {
        let mut guard = IN_PROGRESS_DOWNLOADS.lock().ignore_poison();
        guard.remove(&self.episode_id);
    }
}

impl PodcastEpisodeUseCase {
    fn repo() -> DieselPodcastEpisodeRepository {
        DieselPodcastEpisodeRepository::new(database())
    }

    /// Parse a stored podfetch id (entity rows carry the canonical UUID as a
    /// `String`) into a `Uuid` for the repository layer.
    fn parse_id(id: &str) -> Result<Uuid, CustomError> {
        Uuid::parse_str(id).map_err(|_| {
            CustomErrorInner::BadRequest(format!("'{id}' is not a valid id"), Warning).into()
        })
    }

    fn try_acquire_download_guard(episode_id: &str) -> Option<InProgressDownloadGuard> {
        let mut downloads = IN_PROGRESS_DOWNLOADS.lock().ignore_poison();
        if downloads.contains(episode_id) {
            return None;
        }
        downloads.insert(episode_id.to_string());
        Some(InProgressDownloadGuard {
            episode_id: episode_id.to_string(),
        })
    }

    pub fn get_podcast_episodes_by_url(path: &str) -> Result<Option<PodcastEpisode>, CustomError> {
        Self::repo()
            .find_by_file_path(path)
            .map(|episode| episode.map(Into::into))
            .map_err(Into::into)
    }

    pub fn get_podcast_episode_by_internal_id(
        podcast_episode_id: Uuid,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        Self::repo()
            .find_by_id(podcast_episode_id)
            .map(|episode| episode.map(Into::into))
            .map_err(Into::into)
    }

    /// Resolve an episode by its pre-migration integer id (backwards-compat).
    pub fn get_podcast_episode_by_legacy_id(
        legacy_id: i64,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        Self::repo()
            .find_by_legacy_id(legacy_id)
            .map(|episode| episode.map(Into::into))
            .map_err(Into::into)
    }

    pub fn get_position_of_episode(timestamp: &str, podcast_id: Uuid) -> Result<usize, CustomError> {
        Self::repo()
            .get_position_of_episode(timestamp, podcast_id)
            .map_err(Into::into)
    }

    pub fn get_nth_page_of_podcast_episodes(
        last_podcast_episode_id: Uuid,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::repo()
            .get_nth_page(last_podcast_episode_id, 100)
            .map(|episodes| episodes.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Inbox listing: non-deleted episodes the user has not triaged yet,
    /// newest first, regardless of download state.
    pub fn get_inbox_episodes(
        exclude_episode_ids: &[Uuid],
        last_date: Option<&str>,
        limit: i64,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::repo()
            .get_inbox_episodes(exclude_episode_ids, last_date, limit)
            .map(|episodes| episodes.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// Archive listing: every downloaded, non-deleted episode, newest first.
    pub fn get_downloaded_episodes_paginated(
        last_date: Option<&str>,
        limit: i64,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::repo()
            .get_downloaded_episodes_paginated(last_date, limit)
            .map(|episodes| episodes.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub fn get_podcast_episode_by_id(id: &str) -> Result<Option<PodcastEpisode>, CustomError> {
        Self::repo()
            .find_by_episode_id(id)
            .map(|episode| episode.map(Into::into))
            .map_err(Into::into)
    }

    pub fn get_podcast_episode_by_url(
        url: &str,
        podcast_id: Option<Uuid>,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        Self::repo()
            .find_by_url(url, podcast_id)
            .map(|episode| episode.map(Into::into))
            .map_err(Into::into)
    }

    pub fn query_podcast_episode_by_url(url: &str) -> Result<Option<PodcastEpisode>, CustomError> {
        Self::repo()
            .query_by_url_like(url)
            .map(|episode| episode.map(Into::into))
            .map_err(Into::into)
    }

    fn parse_recording_date(item: &Item) -> String {
        let mut inserted_date = String::new();

        if let Some(date) = &item.pub_date {
            fn parse_naive(timestring: &str) -> chrono::ParseResult<DateTime<FixedOffset>> {
                let date_without_weekday = &timestring[5..];
                DateTime::parse_from_str(date_without_weekday, "%d %b %Y %H:%M:%S %z")
            }

            let parsed_date = DateTime::parse_from_rfc2822(date).unwrap_or(
                DateTime::parse_from_rfc3339(date).unwrap_or(
                    parse_naive(date)
                        .unwrap_or(DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap()),
                ),
            );
            inserted_date = parsed_date.with_timezone(&Utc).to_rfc3339();
        }

        inserted_date
    }

    fn insert_podcast_episode(
        podcast: &Podcast,
        item: &Item,
        episode_image_url: &str,
        duration: i32,
    ) -> Result<PodcastEpisode, CustomError> {
        let guid_to_insert = Guid {
            value: uuid::Uuid::new_v4().to_string(),
            ..Default::default()
        };

        let youtube_video_id = crate::services::sponsorblock::video_id::extract_youtube_video_id(
            item.link.as_deref(),
            item.guid.as_ref().map(|g| g.value.as_str()),
            item.enclosure.as_ref().map(|e| e.url.as_str()),
        );

        Self::repo()
            .create(NewPodcastEpisode {
                podcast_id: Self::parse_id(&podcast.id)?,
                episode_id: uuid::Uuid::new_v4().to_string(),
                name: item
                    .title
                    .clone()
                    .unwrap_or_else(|| "No title given".to_string()),
                url: item.enclosure.clone().unwrap().url,
                date_of_recording: Self::parse_recording_date(item),
                image_url: episode_image_url.to_string(),
                total_time: duration,
                description: opt_or_empty_string(item.clone().description),
                guid: item.guid.clone().unwrap_or(guid_to_insert).value,
                youtube_video_id,
            })
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Bridges a feed item's `<podcast:transcript>` tags into transcript
    /// bookkeeping (Task 6's `TranscriptService::upsert_from_feed`) right
    /// after the episode row they belong to has been created or updated.
    /// This is pure DB bookkeeping — no HTTP fetch happens here, that is the
    /// download hook (Task 8). Feed refresh must never fail because of
    /// transcripts, so any failure (an unparsable episode id, a DB error) is
    /// only logged and never propagated.
    fn sync_transcript_tags_for_episode(item: &Item, episode_id: &str) {
        let tags = extract_transcript_tags(item);
        if tags.is_empty() {
            return;
        }

        let episode_uuid = match Self::parse_id(episode_id) {
            Ok(id) => id,
            Err(err) => {
                tracing::error!(
                    "Could not parse episode id '{}' while syncing feed transcript tags: {:?}",
                    episode_id,
                    err
                );
                return;
            }
        };

        if let Err(err) =
            TranscriptService::default_service().upsert_from_feed(episode_uuid, &tags)
        {
            tracing::error!(
                "Failed to upsert feed transcript tags for episode {}: {:?}",
                episode_uuid,
                err
            );
        }
    }

    pub fn get_podcast_episodes_of_podcast(
        podcast_id: Uuid,
        last_id: Option<String>,
        only_unlistened: Option<bool>,
        user: &User,
    ) -> PodcastEpisodeWithFavorited {
        Self::repo()
            .get_episodes_with_history(
                podcast_id,
                &user.username,
                last_id.as_deref(),
                only_unlistened.unwrap_or(false),
                75,
            )
            .map(|rows| {
                rows.into_iter()
                    .map(|(episode, history, favorite)| {
                        (episode.into(), history.map(Into::into), favorite)
                    })
                    .collect()
            })
            .map_err(Into::into)
    }

    pub fn get_last_n_podcast_episodes_by_count(
        podcast_id: Uuid,
        n_episodes: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::repo()
            .get_last_n_episodes(podcast_id, i64::from(n_episodes))
            .map(|episodes| episodes.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub fn update_local_paths(
        episode_id: &str,
        file_image_path: &str,
        file_episode_path: &str,
    ) -> Result<(), CustomError> {
        Self::repo()
            .update_local_paths(episode_id, file_image_path, file_episode_path)
            .map_err(Into::into)
    }

    pub fn delete_episodes_of_podcast(podcast_id: Uuid) -> Result<(), CustomError> {
        let triage_service = EpisodeTriageService::default_service();
        Self::get_episodes_by_podcast_id(podcast_id)?
            .iter()
            .try_for_each(|episode| {
                let episode_uuid = Self::parse_id(&episode.id)?;
                PlaylistService::default_service()
                    .delete_playlist_items_by_episode_id(episode_uuid)?;
                triage_service.delete_triage_for_episode(episode_uuid)
            })?;

        Self::repo()
            .delete_by_podcast_id(podcast_id)
            .map_err(Into::into)
    }

    pub fn update_podcast_image(id: &str, image_url: &str) -> Result<(), CustomError> {
        crate::services::podcast::service::PodcastService::update_podcast_image(
            id,
            image_url,
            &ENVIRONMENT_SERVICE.default_file_handler.to_string(),
        )
    }

    pub fn check_if_downloaded(download_episode_url: &str) -> Result<bool, CustomError> {
        Self::repo()
            .check_if_downloaded(download_episode_url)
            .map_err(Into::into)
    }

    pub fn update_podcast_episode_status(
        download_url_of_episode: &str,
        download_location_to_set: Option<FileHandlerType>,
    ) -> Result<PodcastEpisode, CustomError> {
        Self::repo()
            .update_download_status(
                download_url_of_episode,
                download_location_to_set.map(|d| d.to_string()),
                Utc::now().naive_utc(),
            )
            .map(Into::into)
            .map_err(Into::into)
    }

    pub fn get_episodes() -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::repo()
            .get_all()
            .map(|episodes| episodes.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub fn get_podcast_episodes_older_than_days(
        days: i32,
        podcast_id: Uuid,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::repo()
            .get_episodes_older_than_days(i64::from(days), podcast_id)
            .map(|episodes| episodes.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub fn remove_download_status_of_episode(id: Uuid) -> Result<(), CustomError> {
        do_retry(|| Self::repo().remove_download_status(id).map_err(Into::into))
    }

    pub fn get_episodes_by_podcast_id(id: Uuid) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::repo()
            .find_by_podcast_id(id)
            .map(|episodes| episodes.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub fn update_guid(guid: Guid, episode_id: &str) -> Result<(), CustomError> {
        Self::repo()
            .update_guid(episode_id, &guid.value)
            .map_err(Into::into)
    }

    pub fn update_podcast_episode(
        episode_to_update: PodcastEpisode,
    ) -> Result<PodcastEpisode, CustomError> {
        Self::repo()
            .update(&episode_to_update.clone().into())
            .map(|_| episode_to_update)
            .map_err(Into::into)
    }

    pub fn update_deleted(episode_id: &str, deleted: bool) -> Result<usize, CustomError> {
        Self::repo()
            .update_deleted(episode_id, deleted)
            .map_err(Into::into)
    }

    pub fn get_podcast_episodes_by_podcast_to_k(
        top_k: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::repo()
            .get_episodes_by_podcast_to_k(i64::from(top_k))
            .map(|episodes| episodes.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub fn update_episode_numbering_processed(
        processed: bool,
        episode_id: &str,
    ) -> Result<(), CustomError> {
        Self::repo()
            .update_episode_numbering_processed(episode_id, processed)
            .map_err(Into::into)
    }

    pub fn download_podcast_episode_if_not_locally_available(
        podcast_episode: PodcastEpisode,
        podcast: Podcast,
    ) -> Result<(), CustomError> {
        let podcast_episode_cloned = podcast_episode.clone();

        if podcast_episode.is_downloaded() {
            return Ok(());
        }
        let podcast_inserted = Self::perform_download(&podcast_episode_cloned, &podcast)?;
        ChatServerHandle::broadcast_podcast_episode_offline_available(&podcast_inserted, &podcast);

        if is_env_var_present_and_true(TELEGRAM_API_ENABLED) {
            send_new_episode_notification(&podcast_episode.name, &podcast.name)
        }
        Ok(())
    }

    pub fn perform_download(
        podcast_episode: &PodcastEpisode,
        podcast_cloned: &Podcast,
    ) -> Result<PodcastEpisode, CustomError> {
        let _guard = match Self::try_acquire_download_guard(&podcast_episode.episode_id) {
            Some(guard) => guard,
            None => {
                tracing::info!(
                    "Skipping download for episode {} because a download is already running",
                    podcast_episode.episode_id
                );
                return Ok(podcast_episode.clone());
            }
        };
        tracing::info!("Downloading podcast episode: {}", podcast_episode.name);
        if let Err(err) =
            DownloadService::download_podcast_episode(podcast_episode.clone(), podcast_cloned)
        {
            if let Err(notification_err) = NotificationService::create_notification(Notification {
                id: String::new(),
                message: format!("{} ({})", podcast_episode.name, err.inner),
                created_at: chrono::Utc::now().naive_utc().to_string(),
                type_of_message: "DownloadFailed".to_string(),
                status: "unread".to_string(),
            }) {
                tracing::error!(
                    "Failed to insert failed-download notification for episode {}: {}",
                    podcast_episode.episode_id,
                    notification_err
                );
            }
            return Err(err);
        }
        let podcast = Self::update_podcast_episode_status(
            &podcast_episode.url,
            Some(ENVIRONMENT_SERVICE.default_file_handler.clone()),
        )?;
        let notification = Notification {
            id: String::new(),
            message: podcast_episode.name.to_string(),
            created_at: chrono::Utc::now().naive_utc().to_string(),
            type_of_message: "Download".to_string(),
            status: "unread".to_string(),
        };
        NotificationService::create_notification(notification)?;
        Ok(podcast)
    }

    pub fn get_last_n_podcast_episodes(
        podcast: Podcast,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        let podcast_settings =
            PodcastSettingsService::get_settings_for_podcast(Self::parse_id(&podcast.id)?)?;
        let settings = SettingsService::shared().get_settings()?.unwrap();
        let n_episodes;

        if let Some(podcast_settings) = podcast_settings {
            if podcast_settings.activated {
                n_episodes = podcast_settings.podcast_prefill;
            } else {
                n_episodes = settings.podcast_prefill;
            }
        } else {
            n_episodes = settings.podcast_prefill;
        }

        Self::get_last_n_podcast_episodes_by_count(Self::parse_id(&podcast.id)?, n_episodes)
    }

    // Used for creating/updating podcasts
    #[tracing::instrument(skip_all, fields(podcast_id = podcast.id, podcast_name = %podcast.name))]
    pub fn insert_podcast_episodes(podcast: &Podcast) -> Result<Vec<PodcastEpisode>, CustomError> {
        let is_redirected = Arc::new(Mutex::new(false)); // Variable to store the redirection status

        let returned_data_from_podcast_insert =
            Self::do_request_to_podcast_server(podcast.clone())?;

        let channel = Channel::read_from(returned_data_from_podcast_insert.content.as_bytes());

        match channel {
            Ok(channel) => {
                if *is_redirected.clone().lock().ignore_poison() {
                    tracing::info!(
                        "The podcast {} has moved to {}",
                        podcast.name,
                        returned_data_from_podcast_insert.url
                    );
                    crate::services::podcast::service::PodcastService::update_podcast_urls_on_redirect(
                        Self::parse_id(&podcast.id)?,
                        &returned_data_from_podcast_insert.url,
                    );
                    Self::update_episodes_on_redirect(channel.items())?;
                }

                Self::handle_itunes_extension(podcast, &channel)?;

                Self::update_podcast_fields(channel.clone(), Self::parse_id(&podcast.id)?)?;

                let mut podcast_inserted = Vec::new();

                Self::handle_podcast_image_insert(podcast, &channel)?;

                for item in channel.items.iter() {
                    if item.enclosure.is_none() {
                        tracing::info!(
                            "Skipping podcast episode {:?} as it has no enclosure",
                            item.title
                        );
                        continue;
                    }

                    let itunes_ext = &item.itunes_ext;
                    let opt_found_podcast_episode: Option<PodcastEpisode> = match &item.guid {
                        Some(guid) => Self::get_podcast_episode_by_guid(&guid.value)?,
                        None => {
                            if let Some(enclosure) = &item.enclosure {
                                Self::get_podcast_episode_by_url(
                                    &enclosure.url.to_string(),
                                    Some(Self::parse_id(&podcast.id)?),
                                )?
                            } else {
                                None
                            }
                        }
                    };

                    if let Some(podcast_episode) = &opt_found_podcast_episode {
                        let mut updated_podcast_episode = podcast_episode.clone();
                        if let Some(title) = &item.title {
                            updated_podcast_episode.name = title.to_string();
                        }

                        if let Some(enclosure) = &item.enclosure {
                            updated_podcast_episode.url = enclosure.url.to_string();
                        }

                        if let Some(description) = &item.description {
                            updated_podcast_episode.description = description.to_string();
                        }

                        // Backfill missing episode artwork with the parent
                        // podcast's image so older rows that stored the
                        // default fallback get repaired on next feed refresh.
                        let episode_itunes_image =
                            itunes_ext.as_ref().and_then(|ext| ext.image.clone());
                        if let Some(itunes_image) = episode_itunes_image {
                            updated_podcast_episode.image_url = itunes_image;
                        } else if DownloadService::is_default_fallback_image_url(
                            &updated_podcast_episode.image_url,
                        ) && !DownloadService::is_default_fallback_image_url(
                            &podcast.original_image_url,
                        ) {
                            updated_podcast_episode.image_url = podcast.original_image_url.clone();
                        }

                        if updated_podcast_episode.name != podcast_episode.name
                            || updated_podcast_episode.url != podcast_episode.url
                            || updated_podcast_episode.description != podcast_episode.description
                            || updated_podcast_episode.image_url != podcast_episode.image_url
                        {
                            Self::update_podcast_episode(updated_podcast_episode.clone())?;
                        }

                        Self::sync_transcript_tags_for_episode(item, &podcast_episode.id);

                        // Skip already existing episodes with insert
                        continue;
                    };

                    let mut duration_of_podcast_episode = 0;
                    // Fall back to the parent podcast's image (the one that
                    // wraps all episodes) when an episode carries no image of
                    // its own.
                    let mut image_url = if !podcast.original_image_url.is_empty() {
                        podcast.original_image_url.clone()
                    } else {
                        String::new()
                    };

                    // itunes extension checking
                    if let Some(itunes_ext) = &itunes_ext {
                        // duration
                        if let Some(duration_from_itunes) = &itunes_ext.duration {
                            duration_of_podcast_episode =
                                Self::parse_duration(duration_from_itunes);
                        }
                        if let Some(itunes_image) = &itunes_ext.image {
                            image_url = itunes_image.to_string();
                        }
                    }

                    let inserted_episode = Self::insert_podcast_episode(
                        podcast,
                        item,
                        &image_url,
                        duration_of_podcast_episode as i32,
                    )?;
                    Self::sync_transcript_tags_for_episode(item, &inserted_episode.id);
                    podcast_inserted.push(inserted_episode);
                }
                Ok(podcast_inserted)
            }
            Err(e) => {
                tracing::info!(
                    "Error parsing podcast {} {:?} with cause {:?}",
                    podcast.name,
                    returned_data_from_podcast_insert.content,
                    e
                );
                Err(CustomErrorInner::BadRequest(
                    format!("Error parsing podcast {} with cause {:?}", podcast.name, e,),
                    ErrorSeverity::Error,
                )
                .into())
            }
        }
    }

    fn handle_podcast_image_insert(
        podcast: &Podcast,
        channel: &Channel,
    ) -> Result<(), CustomError> {
        match channel.image() {
            Some(image) => {
                crate::services::podcast::service::PodcastService::update_original_image_url(
                    &image.url.to_string(),
                    Self::parse_id(&podcast.id)?,
                )?;
            }
            None => {
                crate::services::podcast::service::PodcastService::update_original_image_url(
                    "",
                    Self::parse_id(&podcast.id)?,
                )?;
            }
        }
        Ok(())
    }

    fn handle_itunes_extension(podcast: &Podcast, channel: &Channel) -> Result<(), CustomError> {
        if let Some(extension) = &channel.itunes_ext
            && let Some(new_feed) = &extension.new_feed_url
        {
            let new_url = new_feed;
            crate::services::podcast::service::PodcastService::update_podcast_urls_on_redirect(
                Self::parse_id(&podcast.id)?,
                new_url,
            );

            let returned_data_from_server = Self::do_request_to_podcast_server(podcast.clone())?;

            let channel = Channel::read_from(returned_data_from_server.content.as_bytes()).unwrap();
            let items = channel.items();
            Self::update_episodes_on_redirect(items)?;
        }
        Ok(())
    }

    fn update_episodes_on_redirect(items: &[Item]) -> Result<(), CustomError> {
        for item in items.iter() {
            match &item.guid {
                Some(guid) => {
                    let opt_found_podcast_episode = Self::get_podcast_episode_by_guid(&guid.value)?;
                    if let Some(found_podcast_episode) = opt_found_podcast_episode {
                        let mut podcast_episode = found_podcast_episode.clone();
                        podcast_episode.url = item.enclosure.as_ref().unwrap().url.to_string();
                        Self::update_podcast_episode(podcast_episode)?;
                    }
                }
                None => {
                    println!("No guid found for episode {:?}", item.title.as_ref());
                }
            }
        }
        Ok(())
    }

    fn get_podcast_episode_by_guid(
        guid_to_search: &str,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        Self::repo()
            .find_by_guid(guid_to_search)
            .map(|episode| episode.map(Into::into))
            .map_err(Into::into)
    }

    fn parse_duration(duration_str: &str) -> u32 {
        let parts: Vec<&str> = duration_str.split(':').collect();
        match parts.len() {
            1 => parts[0].parse::<u32>().unwrap_or(0),
            2 => {
                let minutes = parts[0].parse::<u32>().unwrap_or(0);
                let seconds = parts[1].parse::<u32>().unwrap_or(0);
                minutes * 60 + seconds
            }
            3 => {
                let hours = parts[0].parse::<u32>().unwrap_or(0);
                let minutes = parts[1].parse::<u32>().unwrap_or(0);
                let seconds = parts[2].parse::<u32>().unwrap_or(0);
                hours * 3600 + minutes * 60 + seconds
            }
            4 => {
                let days = parts[0].parse::<u32>().unwrap_or(0);
                let hours = parts[1].parse::<u32>().unwrap_or(0);
                let minutes = parts[2].parse::<u32>().unwrap_or(0);
                let seconds = parts[3].parse::<u32>().unwrap_or(0);
                days * 86400 + hours * 3600 + minutes * 60 + seconds
            }
            _ => 0,
        }
    }

    #[inline]
    pub fn get_url_file_suffix(url: &str) -> Result<String, Error> {
        let mut url = Url::parse(url).unwrap();
        url.set_query(None);
        Ok(Path::new(&String::from(url))
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .into_owned())
    }

    pub fn query_for_podcast(query: &str) -> Result<Vec<PodcastEpisode>, CustomError> {
        crate::services::podcast::service::PodcastService::query_for_podcast(query)
    }

    pub fn find_all_downloaded_podcast_episodes() -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::get_episodes()
    }

    pub fn find_all_downloaded_podcast_episodes_with_top_k(
        top_k: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::get_podcast_episodes_by_podcast_to_k(top_k)
    }

    pub fn map_to_local_url(url: &str) -> String {
        let mut splitted_url = url.split('/').collect::<Vec<&str>>();
        let new_last_part = urlencoding::encode(splitted_url.last().unwrap())
            .clone()
            .to_string();
        splitted_url.pop();
        splitted_url.push(&new_last_part);
        splitted_url.join("/")
    }

    pub fn find_all_downloaded_podcast_episodes_by_podcast_id(
        podcast_id: Uuid,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::get_episodes_by_podcast_id(podcast_id)
    }

    fn update_podcast_fields(feed: Channel, podcast_id: Uuid) -> Result<(), CustomError> {
        let itunes = feed.clone().itunes_ext;

        let ext = feed
            .extensions()
            .get("podcast")
            .and_then(|m| m.get("guid"))
            .and_then(|v| v.first())
            .and_then(|v| v.value.clone());

        if let Some(itunes) = itunes {
            let constructed_extra_fields = PodcastBuilder::new(podcast_id)
                .author(itunes.author)
                .last_build_date(feed.last_build_date.clone())
                .description(feed.description.clone())
                .guid(ext)
                .language(feed.language.clone())
                .keywords(itunes.categories)
                .build();
            crate::services::podcast::service::PodcastService::update_podcast_fields(
                constructed_extra_fields,
            )?;
        }

        Ok(())
    }

    pub fn cleanup_old_episodes(days_from_settings: i32) {
        let podcasts = crate::services::podcast::service::PodcastService::get_all_podcasts_raw();

        if podcasts.is_err() {
            return;
        }

        for p in podcasts.unwrap() {
            let p_uuid = match Self::parse_id(&p.id) {
                Ok(id) => id,
                Err(_) => continue,
            };
            let podcast_settings = PodcastSettingsService::get_settings_for_podcast(p_uuid);
            if podcast_settings.is_err() {
                continue;
            }
            let days = match podcast_settings.unwrap() {
                Some(podcast_settings) if podcast_settings.auto_cleanup => {
                    podcast_settings.auto_cleanup_days
                }
                _ => days_from_settings,
            };

            let old_podcast_episodes =
                match Self::get_podcast_episodes_older_than_days(days, p_uuid)
            {
                Ok(episodes) => episodes,
                Err(err) => {
                    tracing::error!(
                        "Error loading old podcast episodes for podcast {}: {}",
                        p.id,
                        err
                    );
                    continue;
                }
            };

            tracing::info!("Cleaning up {} old episodes", old_podcast_episodes.len());
            for old_podcast_episode in old_podcast_episodes {
                let episode_id = match Self::parse_id(&old_podcast_episode.id) {
                    Ok(id) => id,
                    Err(err) => {
                        tracing::error!("Invalid episode id {}: {}", old_podcast_episode.id, err);
                        continue;
                    }
                };
                match FavoritePodcastEpisodeService::default_service()
                    .is_liked_by_someone(episode_id)
                {
                    Ok(true) => {
                        continue;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        tracing::error!("Error checking if podcast episode is liked.{e}");
                    }
                }

                let res = FileService::cleanup_old_episode(&old_podcast_episode);

                match res {
                    Ok(_) => {
                        if let Err(err) =
                            Self::remove_download_status_of_episode(episode_id)
                        {
                            tracing::error!(
                                "Error clearing download status for episode {}: {}",
                                old_podcast_episode.id,
                                err
                            );
                        }
                    }
                    Err(e) => {
                        println!("Error deleting podcast episode.{e}");
                    }
                }
            }
        }
    }

    fn do_request_to_podcast_server(podcast: Podcast) -> Result<RequestReturnType, CustomError> {
        let is_redirected = Arc::new(Mutex::new(false)); // Variable to store the redirection status
        let client = get_sync_client(&ENVIRONMENT_SERVICE)
            .redirect(Policy::custom({
                let is_redirected = Arc::clone(&is_redirected);

                move |attempt| {
                    if !attempt.previous().is_empty() {
                        *is_redirected.lock().unwrap() = true;
                    }
                    attempt.follow()
                }
            }))
            .build()
            .map_err(map_reqwest_error)?;
        let mut header_map = HeaderMap::new();
        header_map.append(
            ACCEPT,
            // Safe as it is a standard header
            "application/rss+xml,application/xml".parse().unwrap(),
        );
        header_map.append("User-Agent", COMMON_USER_AGENT.parse().unwrap());
        let result = client
            .get(podcast.clone().rssfeed)
            .headers(header_map)
            .send()
            .map_err(map_reqwest_error)?;
        let url = result.url().clone().to_string();
        let content = result.text().map_err(map_reqwest_error)?;

        Ok(RequestReturnType { url, content })
    }

    pub(crate) fn delete_podcast_episode_locally(
        episode_id: &str,
    ) -> Result<PodcastEpisode, CustomError> {
        let episode = Self::get_podcast_episode_by_id(episode_id)?;
        if episode.is_none() {
            return Err(CustomErrorInner::NotFound(Warning).into());
        }

        match episode {
            Some(episode) => {
                FileService::cleanup_old_episode(&episode)?;
                Self::remove_download_status_of_episode(Self::parse_id(&episode.id)?)?;
                Self::update_deleted(episode_id, true)?;
                Ok(episode)
            }
            None => Err(CustomErrorInner::NotFound(Warning).into()),
        }
    }

    /// Downloads every episode of the podcast whose DB row has no
    /// `download_location` (i.e. never downloaded) and is not soft-deleted.
    /// Runs downloads in parallel chunks of 3, matching `schedule_episode_download`.
    /// Returns the number of episodes that were queued.
    pub fn download_missing_episodes_for_podcast(podcast: &Podcast) -> Result<usize, CustomError> {
        let episodes = Self::get_episodes_by_podcast_id(Self::parse_id(&podcast.id)?)?;
        Ok(Self::download_missing_in(podcast, episodes))
    }

    /// Download the episodes whose chronological position (oldest = 1) falls
    /// within the inclusive `[from, to]` range. Positions match
    /// `get_position_of_episode` (ascending by publication date). Only missing
    /// episodes are fetched. Returns how many episodes were queued.
    pub fn download_episode_range_for_podcast(
        podcast: &Podcast,
        from: usize,
        to: usize,
    ) -> Result<usize, CustomError> {
        let mut episodes = Self::get_episodes_by_podcast_id(Self::parse_id(&podcast.id)?)?;
        episodes.sort_by(|a, b| a.date_of_recording.cmp(&b.date_of_recording));
        let start = from.saturating_sub(1).min(episodes.len());
        let end = to.min(episodes.len());
        let slice = if start >= end {
            Vec::new()
        } else {
            episodes[start..end].to_vec()
        };
        Ok(Self::download_missing_in(podcast, slice))
    }

    /// Download every not-yet-downloaded, non-deleted episode in `episodes`
    /// (in parallel, capped at `MAX_PARALLEL_DOWNLOADS`). Returns how many
    /// episodes were queued.
    fn download_missing_in(podcast: &Podcast, episodes: Vec<PodcastEpisode>) -> usize {
        const MAX_PARALLEL_DOWNLOADS: usize = 3;
        let missing: Vec<PodcastEpisode> = episodes
            .into_iter()
            .filter(|e| !e.deleted && !e.is_downloaded())
            .collect();
        let count = missing.len();

        for chunk in missing.chunks(MAX_PARALLEL_DOWNLOADS) {
            let mut handles = Vec::with_capacity(chunk.len());
            for episode in chunk.iter().cloned() {
                let podcast_for_thread = podcast.clone();
                handles.push(thread::spawn(move || {
                    if let Err(err) = Self::download_podcast_episode_if_not_locally_available(
                        episode,
                        podcast_for_thread,
                    ) {
                        tracing::error!("Error downloading podcast episode: {err}");
                    }
                }));
            }
            for handle in handles {
                if let Err(err) = handle.join() {
                    tracing::error!(
                        "Error joining download worker for podcast {}: {:?}",
                        podcast.id,
                        err
                    );
                }
            }
        }
        count
    }

    /// Re-downloads episodes whose DB row says they are downloaded but whose
    /// file is missing on disk / in the configured backend. Uses
    /// `perform_download` directly (bypasses the `is_downloaded` guard in
    /// `download_podcast_episode_if_not_locally_available`). Returns the
    /// number of episodes that were queued.
    pub fn redownload_missing_files_for_podcast(podcast: &Podcast) -> Result<usize, CustomError> {
        const MAX_PARALLEL_DOWNLOADS: usize = 3;
        let episodes = Self::get_episodes_by_podcast_id(Self::parse_id(&podcast.id)?)?;
        let to_redownload: Vec<PodcastEpisode> = episodes
            .into_iter()
            .filter(|e| !e.deleted && e.is_downloaded() && Self::episode_file_missing(e))
            .collect();
        let count = to_redownload.len();

        for chunk in to_redownload.chunks(MAX_PARALLEL_DOWNLOADS) {
            let mut handles = Vec::with_capacity(chunk.len());
            for episode in chunk.iter().cloned() {
                let podcast_for_thread = podcast.clone();
                handles.push(thread::spawn(move || {
                    match Self::perform_download(&episode, &podcast_for_thread) {
                        Ok(updated) => {
                            ChatServerHandle::broadcast_podcast_episode_offline_available(
                                &updated,
                                &podcast_for_thread,
                            );
                        }
                        Err(err) => {
                            tracing::error!(
                                "Error re-downloading episode {}: {}",
                                episode.episode_id,
                                err
                            );
                        }
                    }
                }));
            }
            for handle in handles {
                if let Err(err) = handle.join() {
                    tracing::error!(
                        "Error joining re-download worker for podcast {}: {:?}",
                        podcast.id,
                        err
                    );
                }
            }
        }
        Ok(count)
    }

    /// Clears DB download flags for episodes whose file is missing on disk.
    /// Filesystem is source of truth; no downloads happen. Returns the number
    /// of episodes whose flags were cleared.
    pub fn resync_db_for_podcast(podcast_id: Uuid) -> Result<usize, CustomError> {
        let episodes = Self::get_episodes_by_podcast_id(podcast_id)?;
        let mut affected = 0usize;
        for episode in episodes {
            if !episode.is_downloaded() {
                continue;
            }
            if !Self::episode_file_missing(&episode) {
                continue;
            }
            Self::remove_download_status_of_episode(Self::parse_id(&episode.id)?)?;
            ChatServerHandle::broadcast_podcast_episode_deleted_locally(&episode);
            affected += 1;
        }
        Ok(affected)
    }

    /// Removes every downloaded file for this podcast and clears the matching
    /// DB flags, but keeps the episode rows intact (unlike single-episode
    /// delete, this does not set `deleted=true`). Episodes that any user has
    /// marked as favorite are skipped — same convention as the auto-cleanup
    /// path (see `cleanup_old_episodes`). Returns the number of episodes
    /// whose files were removed.
    pub fn delete_all_downloaded_files_for_podcast(podcast_id: Uuid) -> Result<usize, CustomError> {
        let favorite_service = FavoritePodcastEpisodeService::default_service();
        let episodes = Self::get_episodes_by_podcast_id(podcast_id)?;
        let mut affected = 0usize;
        for episode in episodes {
            if !episode.is_downloaded() {
                continue;
            }
            let episode_id = Self::parse_id(&episode.id)?;
            if favorite_service
                .is_liked_by_someone(episode_id)
                .unwrap_or(false)
            {
                continue;
            }
            FileService::cleanup_old_episode(&episode)?;
            Self::remove_download_status_of_episode(episode_id)?;
            ChatServerHandle::broadcast_podcast_episode_deleted_locally(&episode);
            affected += 1;
        }
        Ok(affected)
    }

    fn episode_file_missing(episode: &PodcastEpisode) -> bool {
        let (Some(path), Some(location)) = (
            episode.file_episode_path.as_deref(),
            episode.download_location.as_deref(),
        ) else {
            return true;
        };
        let handler = FileHandlerType::from(location);
        !FileHandleWrapper::path_exists(path, FileRequest::File, &handler)
    }

    pub fn get_track_number_for_episode(
        podcast_id: Uuid,
        date_of_recording_to_search: &str,
    ) -> Result<i64, CustomError> {
        use podfetch_persistence::schema::podcast_episodes::dsl::podcast_episodes;

        podcast_episodes
            .filter(
                podfetch_persistence::schema::podcast_episodes::podcast_id
                    .eq(podcast_id.to_string()),
            )
            .filter(
                podfetch_persistence::schema::podcast_episodes::date_of_recording
                    .le(date_of_recording_to_search),
            )
            .count()
            .get_result::<i64>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))
    }
}

struct RequestReturnType {
    pub url: String,
    pub content: String,
}

/// Reads `<podcast:transcript>` tags out of a feed item's extensions.
///
/// The `rss` crate keys parsed extensions by their local name, i.e. without
/// the namespace prefix, so a `<podcast:transcript>` element normally lands
/// under `extensions()["podcast"]["transcript"]`. Some feeds/tooling can
/// still surface the qualified name as the map key instead, so both
/// `"transcript"` and `"podcast:transcript"` are checked defensively. A tag
/// without a `url` attribute is skipped — there is nothing to store or later
/// download (Task 8) for it.
fn extract_transcript_tags(item: &Item) -> Vec<FeedTranscriptTag> {
    let Some(podcast_ns) = item.extensions().get("podcast") else {
        return Vec::new();
    };

    ["transcript", "podcast:transcript"]
        .into_iter()
        .filter_map(|key| podcast_ns.get(key))
        .flatten()
        .filter_map(|ext| {
            let url = ext.attrs().get("url")?.clone();
            let mime_type = ext
                .attrs()
                .get("type")
                .cloned()
                .unwrap_or_else(|| "text/plain".to_string());
            let language = ext.attrs().get("language").cloned();
            Some(FeedTranscriptTag {
                url,
                mime_type,
                language,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rss::extension::{Extension, ExtensionBuilder};
    use std::collections::BTreeMap;

    fn transcript_extension(url: Option<&str>, mime_type: Option<&str>, language: Option<&str>) -> Extension {
        let mut builder = ExtensionBuilder::default();
        builder.name("podcast:transcript");

        let mut attrs = BTreeMap::new();
        if let Some(url) = url {
            attrs.insert("url".to_string(), url.to_string());
        }
        if let Some(mime_type) = mime_type {
            attrs.insert("type".to_string(), mime_type.to_string());
        }
        if let Some(language) = language {
            attrs.insert("language".to_string(), language.to_string());
        }
        builder.attrs(attrs);

        builder.build()
    }

    fn item_with_transcripts(transcripts: Vec<Extension>) -> Item {
        let mut podcast_ns = BTreeMap::new();
        podcast_ns.insert("transcript".to_string(), transcripts);

        let mut extensions = rss::extension::ExtensionMap::new();
        extensions.insert("podcast".to_string(), podcast_ns);

        Item {
            extensions,
            ..Default::default()
        }
    }

    #[test]
    fn extract_transcript_tags_reads_url_type_and_language() {
        let item = item_with_transcripts(vec![
            transcript_extension(Some("https://example.com/ep1.vtt"), Some("text/vtt"), Some("en")),
            transcript_extension(Some("https://example.com/ep1.json"), Some("application/json"), Some("en")),
        ]);

        let tags = extract_transcript_tags(&item);

        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].url, "https://example.com/ep1.vtt");
        assert_eq!(tags[0].mime_type, "text/vtt");
        assert_eq!(tags[0].language.as_deref(), Some("en"));
        assert_eq!(tags[1].url, "https://example.com/ep1.json");
        assert_eq!(tags[1].mime_type, "application/json");
        assert_eq!(tags[1].language.as_deref(), Some("en"));
    }

    #[test]
    fn extract_transcript_tags_defaults_missing_type_to_text_plain() {
        let item = item_with_transcripts(vec![transcript_extension(
            Some("https://example.com/ep1.txt"),
            None,
            None,
        )]);

        let tags = extract_transcript_tags(&item);

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].mime_type, "text/plain");
        assert_eq!(tags[0].language, None);
    }

    #[test]
    fn extract_transcript_tags_skips_tag_without_url() {
        let item = item_with_transcripts(vec![
            transcript_extension(None, Some("text/vtt"), Some("en")),
            transcript_extension(Some("https://example.com/ep1.vtt"), Some("text/vtt"), Some("en")),
        ]);

        let tags = extract_transcript_tags(&item);

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].url, "https://example.com/ep1.vtt");
    }

    #[test]
    fn extract_transcript_tags_checks_qualified_name_key_too() {
        let mut podcast_ns = BTreeMap::new();
        podcast_ns.insert(
            "podcast:transcript".to_string(),
            vec![transcript_extension(
                Some("https://example.com/ep1.vtt"),
                Some("text/vtt"),
                None,
            )],
        );
        let mut extensions = rss::extension::ExtensionMap::new();
        extensions.insert("podcast".to_string(), podcast_ns);
        let item = Item {
            extensions,
            ..Default::default()
        };

        let tags = extract_transcript_tags(&item);

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].url, "https://example.com/ep1.vtt");
    }

    #[test]
    fn extract_transcript_tags_returns_empty_when_no_podcast_namespace() {
        let item = Item::default();

        let tags = extract_transcript_tags(&item);

        assert!(tags.is_empty());
    }
}
