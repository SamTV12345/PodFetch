use crate::services::file::service::FileService;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::settings::service::SettingsService;
use crate::services::transcript::service::{FeedTranscriptTag, TranscriptService};
use common_infrastructure::error::ErrorSeverity::{Critical, Warning};
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity, map_db_error};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_catalog::domain::podcast_episode::PodcastEpisodeRepository;
use podfetch_catalog::persistence::podcast::PodcastEntity as Podcast;
use podfetch_catalog::persistence::podcast_episode::DieselPodcastEpisodeRepository;
use podfetch_catalog::persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_identity::api::Principal;
use podfetch_listening::domain::episode::{Episode, EpisodeRepository};
use podfetch_persistence::db::database;
use podfetch_persistence::db::get_connection;
use rss::Guid;
use std::ffi::OsStr;
use std::io::Error;
use std::path::Path;
use url::Url;
use uuid::Uuid;

pub struct PodcastEpisodeUseCase;
type PodcastEpisodeWithFavorited = Result<
    Vec<(
        PodcastEpisode,
        Option<podfetch_listening::domain::episode::Episode>,
        Option<podfetch_library::domain::favorite_podcast_episode::FavoritePodcastEpisode>,
    )>,
    CustomError,
>;

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

    pub fn get_position_of_episode(
        timestamp: &str,
        podcast_id: Uuid,
    ) -> Result<usize, CustomError> {
        Self::repo()
            .get_position_of_episode(timestamp, podcast_id)
            .map_err(Into::into)
    }

    pub fn get_total_episode_count(podcast_id: Uuid) -> Result<usize, CustomError> {
        Self::repo()
            .get_total_episode_count(podcast_id)
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

    /// One page (75) of a podcast's episodes with the user's latest action
    /// and favorite mark. API composition: episodes from the catalog,
    /// actions from listening, favorites from the library.
    ///
    /// With `only_unlistened`, heard episodes are skipped; the catalog is
    /// read in further pages until the page is full, so no exclusion list
    /// of every heard episode has to be sent to the database.
    pub fn get_podcast_episodes_of_podcast(
        podcast_id: Uuid,
        last_id: Option<String>,
        only_unlistened: Option<bool>,
        user: &Principal,
    ) -> PodcastEpisodeWithFavorited {
        const PAGE: i64 = 75;
        let only_unlistened = only_unlistened.unwrap_or(false);
        let listener = podfetch_listening::domain::episode::Listener::new(
            user.id.as_uuid(),
            user.username.clone(),
        );
        let actions = crate::listening::episode_actions();
        let mut cursor = last_id;
        let mut page: Vec<(
            podfetch_catalog::domain::podcast_episode::PodcastEpisode,
            Option<Episode>,
        )> = Vec::new();
        loop {
            let batch = Self::repo()
                .get_episodes_page(podcast_id, cursor.as_deref(), PAGE)
                .map_err(CustomError::from)?;
            let exhausted = (batch.len() as i64) < PAGE;
            cursor = batch.last().map(|e| e.date_of_recording.clone());
            let guids: Vec<String> = batch.iter().map(|e| e.guid.clone()).collect();
            let mut latest: std::collections::HashMap<String, Episode> = actions
                .find_latest_by_guids(&listener, &guids)
                .map_err(CustomError::from)?
                .into_iter()
                .filter_map(|action| Some((action.guid.clone()?, action)))
                .collect();
            for episode in batch {
                let action = latest.remove(&episode.guid);
                if only_unlistened && action.as_ref().is_some_and(Episode::counts_as_heard) {
                    continue;
                }
                page.push((episode, action));
                if page.len() as i64 == PAGE {
                    break;
                }
            }
            if exhausted || page.len() as i64 == PAGE || !only_unlistened {
                break;
            }
        }

        let episode_ids: Vec<Uuid> = page.iter().map(|(episode, _)| episode.id).collect();
        let mut favorites =
            crate::library::favorite_episodes().favorites_of(user.id.into(), &episode_ids)?;
        Ok(page
            .into_iter()
            .map(|(episode, action)| {
                let favorite = favorites.remove(&episode.id);
                (episode.into(), action, favorite)
            })
            .collect())
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

    /// Records the paths of a downloaded episode (by its catalog id). An
    /// empty image path clears the stored image.
    pub fn update_local_paths(
        id: &str,
        file_image_path: &str,
        file_episode_path: &str,
    ) -> Result<(), CustomError> {
        let image = (!file_image_path.is_empty()).then_some(file_image_path);
        crate::media::episode_files().set_paths(id, image, file_episode_path)
    }

    pub fn update_total_time(episode_id: &str, total_time: i32) -> Result<(), CustomError> {
        Self::repo()
            .update_total_time(episode_id, total_time)
            .map_err(Into::into)
    }

    /// Deletes the podcast's episodes from the catalog. The other contexts
    /// must have removed their data first (`crate::podcast_removal`).
    pub fn delete_episodes_of_podcast(podcast_id: Uuid) -> Result<(), CustomError> {
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

    /// Marks the file of the episode (by its catalog id) as deleted by the
    /// user, or lifts that mark.
    pub fn update_deleted(id: &str, deleted: bool) -> Result<(), CustomError> {
        crate::media::episode_files().set_deleted(id, deleted)
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

    pub fn get_last_n_podcast_episodes(
        podcast: Podcast,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        let podcast_settings =
            PodcastSettingsService::get_settings_for_podcast(Self::parse_id(&podcast.id)?)?;
        let settings = SettingsService::shared()
            .get_settings()?
            .ok_or_else(|| CustomError::from(CustomErrorInner::Unknown(ErrorSeverity::Error)))?;
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

    /// Refreshes the podcast from its feed (the catalog's `FeedSync`) and
    /// hands on what the other contexts need from it: the feed transcript
    /// tags to the transcripts, a newly found cover to the media library.
    /// Returns the new episodes.
    pub fn insert_podcast_episodes(podcast: &Podcast) -> Result<Vec<PodcastEpisode>, CustomError> {
        let report = crate::catalog_ports::feed_sync().sync(&podcast.clone().into())?;
        if let Some(cover) = report.new_cover {
            Self::schedule_podcast_cover_download(podcast, cover);
        }
        let transcripts = TranscriptService::default_service();
        for (episode_id, tags) in &report.transcripts {
            let tags: Vec<FeedTranscriptTag> = tags
                .iter()
                .map(|tag| FeedTranscriptTag {
                    url: tag.url.clone(),
                    mime_type: tag.mime_type.clone(),
                    language: tag.language.clone(),
                })
                .collect();
            // Feed refresh must never fail because of transcripts.
            if let Err(err) = transcripts.upsert_from_feed(*episode_id, &tags) {
                tracing::error!(
                    "Failed to upsert feed transcript tags for episode {episode_id}: {err:?}"
                );
            }
        }
        Ok(report.inserted.into_iter().map(Into::into).collect())
    }

    fn schedule_podcast_cover_download(podcast: &Podcast, image_url: String) {
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let directory_name = podcast.directory_name.clone();
        let directory_id = podcast.directory_id.clone();
        let cover_filename = Uuid::parse_str(&podcast.id).map_or_else(
            |_| crate::settings::default_cover_filename(),
            crate::media::cover_filename,
        );
        let name = podcast.name.clone();
        runtime.spawn(async move {
            tracing::info!("Downloading newly found cover of podcast {name}");
            if let Err(err) = FileService::download_podcast_image(
                &directory_name,
                image_url,
                &directory_id,
                &cover_filename,
            )
            .await
            {
                tracing::warn!("Could not download cover of podcast {name}: {err}");
            }
        });
    }

    pub fn get_podcast_episodes_by_guids(
        guids: &[String],
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::repo()
            .find_by_guids(guids)
            .map(|episodes| episodes.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub fn get_podcast_episode_by_guid(
        guid_to_search: &str,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        Self::repo()
            .find_by_guid(guid_to_search)
            .map(|episode| episode.map(Into::into))
            .map_err(Into::into)
    }

    #[inline]
    pub fn get_url_file_suffix(url: &str) -> Result<String, Error> {
        let mut url = Url::parse(url).map_err(Error::other)?;
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

    /// Percent-encodes the last path segment (the file name) of `url`.
    pub fn map_to_local_url(url: &str) -> String {
        match url.rsplit_once('/') {
            Some((base, file)) => format!("{base}/{}", urlencoding::encode(file)),
            None => urlencoding::encode(url).into_owned(),
        }
    }

    pub fn find_all_downloaded_podcast_episodes_by_podcast_id(
        podcast_id: Uuid,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Self::get_episodes_by_podcast_id(podcast_id)
    }

    /// Removes the files of episodes older than the podcast's cleanup
    /// period (its own setting, `days_from_settings` otherwise). Episodes a
    /// user marked as favorite keep their files.
    pub fn cleanup_old_episodes(
        library: &podfetch_media::library::MediaLibrary,
        days_from_settings: i32,
    ) {
        let Ok(podcasts) =
            crate::services::podcast::service::PodcastService::get_all_podcasts_raw()
        else {
            return;
        };

        for p in podcasts {
            let Ok(p_uuid) = Self::parse_id(&p.id) else {
                continue;
            };
            let Ok(podcast_settings) = PodcastSettingsService::get_settings_for_podcast(p_uuid)
            else {
                continue;
            };
            let days = match podcast_settings {
                Some(podcast_settings) if podcast_settings.auto_cleanup => {
                    podcast_settings.auto_cleanup_days
                }
                _ => days_from_settings,
            };

            let old_podcast_episodes =
                match Self::get_podcast_episodes_older_than_days(days, p_uuid) {
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
                match crate::library::favorite_episodes().is_liked_by_someone(episode_id) {
                    Ok(true) => {
                        continue;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        tracing::error!("Error checking if podcast episode is liked.{e}");
                    }
                }

                if let Err(err) =
                    library.remove_file(&crate::media::media_episode(&old_podcast_episode))
                {
                    tracing::error!(
                        "Error removing the files of episode {}: {err}",
                        old_podcast_episode.id
                    );
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::PodcastEpisodeUseCase;

    #[test]
    fn local_url_encodes_only_the_file_name() {
        assert_eq!(
            PodcastEpisodeUseCase::map_to_local_url("http://host/podcasts/my show/ep 1.mp3"),
            "http://host/podcasts/my show/ep%201.mp3"
        );
        assert_eq!(PodcastEpisodeUseCase::map_to_local_url("a b"), "a%20b");
    }
}
