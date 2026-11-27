use podfetch_catalog::persistence::podcast::PodcastEntity as Podcast;
use podfetch_catalog::persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_identity::api::Principal;

use crate::controllers::controller_utils::unwrap_string;
use crate::podcast::map_podcast_with_context_to_dto;
use crate::podcast::{ItunesWrapper, PodcastDto, PodcastInsertModel, PodindexResponse};
use crate::server::ChatServerHandle;
use crate::services::file::service::FileService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use common_infrastructure::error::ErrorSeverity::Error;
use common_infrastructure::error::{
    CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error,
};
use common_infrastructure::http::COMMON_USER_AGENT;
use common_infrastructure::http::get_http_client;
use common_infrastructure::runtime::{ENVIRONMENT_SERVICE, ITUNES_URL};
use podfetch_catalog::domain::ordering::{OrderCriteria, OrderOption};
use podfetch_catalog::domain::podcast::{NewPodcast, PodcastQuery, PodcastRepository};
use podfetch_catalog::persistence::podcast::DieselPodcastRepository;
use podfetch_library::domain::favorite::FavoriteRepository;
use podfetch_library::persistence::favorite::DieselFavoriteRepository;
use podfetch_persistence::db::database;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use rss::Channel;
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use tokio::task::spawn_blocking;
use uuid::Uuid;

pub struct PodcastService;

fn podcast_repo() -> DieselPodcastRepository {
    DieselPodcastRepository::new(database())
}

fn favorite_repo() -> DieselFavoriteRepository {
    DieselFavoriteRepository::new(database())
}

impl PodcastService {
    pub async fn find_podcast(podcast: &str) -> ItunesWrapper {
        let query = vec![("term", podcast), ("entity", "podcast")];
        let result = async {
            get_http_client(&ENVIRONMENT_SERVICE)
                .get(ITUNES_URL)
                .query(&query)
                .send()
                .await?
                .json::<ItunesWrapper>()
                .await
        }
        .await;

        result.unwrap_or_else(|e| {
            tracing::error!("Error searching for podcast on iTunes: {e}");
            ItunesWrapper::default()
        })
    }

    pub async fn find_podcast_on_podindex(podcast: &str) -> Result<PodindexResponse, CustomError> {
        let headers = Self::compute_podindex_header();

        let query = vec![("q", podcast)];

        let result = get_http_client(&ENVIRONMENT_SERVICE)
            .get("https://api.podcastindex.org/api/1.0/search/byterm")
            .query(&query)
            .headers(headers)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        tracing::info!("Found podcast: {}", result.url());

        let status = result.status();
        let possible_json = result.text().await.map_err(map_reqwest_error)?;

        if status.is_client_error() || status.is_server_error() {
            tracing::error!("Error searching for podcast: {possible_json}");
            Err(CustomErrorInner::BadRequest(possible_json, Error).into())
        } else {
            serde_json::from_str(&possible_json).map_err(|e| {
                tracing::error!("Unexpected podindex search response: {e}");
                CustomErrorInner::BadRequest("Unexpected response from podindex".to_owned(), Error)
                    .into()
            })
        }
    }

    /// Hit Podcastindex `/podcasts/trending`. Passing empty slices / None for
    /// filters yields globally trending podcasts.
    pub async fn trending_on_podindex(
        categories: &[u32],
        language: Option<&str>,
        max: u32,
    ) -> Result<PodindexResponse, CustomError> {
        let headers = Self::compute_podindex_header();
        let mut url = "https://api.podcastindex.org/api/1.0/podcasts/trending".to_string();
        let mut params = vec![("max", max.to_string())];
        if !categories.is_empty() {
            let joined = categories
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("cat", joined));
        }
        if let Some(lang) = language
            && !lang.is_empty()
        {
            params.push(("lang", lang.to_string()));
        }

        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        url.push('?');
        url.push_str(&query);

        let result = get_http_client(&ENVIRONMENT_SERVICE)
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = result.status();
        let body = result.text().await.map_err(map_reqwest_error)?;

        if status.is_client_error() || status.is_server_error() {
            tracing::error!("Podcastindex trending error {status}: {body}");
            return Err(CustomErrorInner::BadRequest(body, Error).into());
        }

        serde_json::from_str::<PodindexResponse>(&body).map_err(|e| {
            tracing::error!("Could not parse Podcastindex trending response: {e}");
            CustomErrorInner::BadRequest(e.to_string(), Error).into()
        })
    }

    pub async fn insert_podcast_from_podindex(
        id: i32,
        added_by: Option<Uuid>,
    ) -> Result<Podcast, CustomError> {
        let resp = get_http_client(&ENVIRONMENT_SERVICE)
            .get(format!(
                "https://api.podcastindex.org/api/1.0/podcasts/byfeedid?id={id}"
            ))
            .headers(Self::compute_podindex_header())
            .send()
            .await
            .map_err(map_reqwest_error)?;
        let podcast = resp.json::<Value>().await.map_err(map_reqwest_error)?;

        Self::handle_insert_of_podcast(
            PodcastInsertModel {
                title: unwrap_string(&podcast["feed"]["title"]),
                id,
                feed_url: unwrap_string(&podcast["feed"]["url"]),
                image_url: unwrap_string(&podcast["feed"]["image"]),
            },
            None,
            added_by,
        )
        .await
    }

    pub async fn handle_insert_of_podcast(
        podcast_insert: PodcastInsertModel,
        channel: Option<Channel>,
        added_by: Option<Uuid>,
    ) -> Result<Podcast, CustomError> {
        let opt_podcast = podcast_repo()
            .find_by_rss_feed(&podcast_insert.feed_url)
            .map_err(CustomError::from)?;
        if opt_podcast.is_some() {
            return Err(CustomErrorInner::Conflict(
                format!(
                    "Podcast with feed url {} already exists",
                    podcast_insert.feed_url
                ),
                ErrorSeverity::Warning,
            )
            .into());
        }

        let podcast_directory_created =
            FileService::create_podcast_directory_exists(&podcast_insert, channel).await?;

        let inserted_podcast: Podcast = podcast_repo()
            .create(NewPodcast {
                name: podcast_insert.title.clone(),
                directory_id: podcast_insert.id.to_string(),
                rssfeed: podcast_insert.feed_url.clone(),
                image_url: podcast_insert.image_url.clone(),
                directory_name: podcast_directory_created,
                added_by,
            })
            .map(Into::into)
            .map_err(CustomError::from)?;

        // Remember which image the cover was downloaded from, so the first
        // feed refresh doesn't treat the podcast as having no image yet.
        if let Ok(podcast_uuid) = Uuid::parse_str(&inserted_podcast.id)
            && !common_infrastructure::runtime::is_default_image_url(&podcast_insert.image_url)
        {
            Self::update_original_image_url(&podcast_insert.image_url, podcast_uuid)?;
        }

        // Resolve the cover name via the podcast's UUID; `podcast_insert.id`
        // is the feed/track id and can't be used to look up settings.
        let cover_filename = Uuid::parse_str(&inserted_podcast.id).map_or_else(
            |_| crate::settings::default_cover_filename(),
            crate::media::cover_filename,
        );
        FileService::download_podcast_image(
            &inserted_podcast.directory_name.clone().clone(),
            podcast_insert.image_url.clone().clone(),
            &podcast_insert.id.clone().to_string(),
            &cover_filename,
        )
        .await?;
        let podcast: Option<Podcast> = podcast_repo()
            .find_by_track_id(podcast_insert.id)
            .map(|opt| opt.map(Into::into))
            .map_err(CustomError::from)?;
        match podcast {
            Some(podcast) => {
                ChatServerHandle::broadcast_podcast_downloaded(podcast.clone());
                let join_result = spawn_blocking(move || {
                    tracing::debug!("Inserting podcast episodes of {}", podcast.name);
                    match PodcastEpisodeService::insert_podcast_episodes(&podcast) {
                        Ok(inserted_podcasts) => {
                            ChatServerHandle::broadcast_added_podcast_episodes(
                                &podcast,
                                inserted_podcasts.clone(),
                            );
                            Self::announce_refreshed(&podcast);
                        }
                        Err(e) => {
                            tracing::error!(
                                "Error inserting podcast episodes for {}: {e}",
                                podcast.name
                            );
                        }
                    }
                })
                .await;
                if let Err(e) = join_result {
                    tracing::error!("Blocking task failed: {e}");
                }
                Ok(inserted_podcast)
            }
            None => {
                tracing::error!("No podcast found after insert");
                Err(CustomErrorInner::NotFound(Error).into())
            }
        }
    }

    pub fn refresh_podcast(podcast: &Podcast) -> Result<(), CustomError> {
        tracing::info!("Refreshing podcast: {}", podcast.name);
        PodcastEpisodeService::insert_podcast_episodes(podcast)?;
        Self::announce_refreshed(podcast);
        Ok(())
    }

    /// Tells the other contexts that the podcast's episodes were refreshed
    /// from its feed; the media library then auto-downloads the latest ones.
    pub fn announce_refreshed(podcast: &Podcast) {
        crate::podcast_events::publish(&podfetch_catalog::events::PodcastRefreshed {
            podcast_id: podcast.id.clone(),
            podcast_name: podcast.name.clone(),
        });
    }

    pub fn get_podcast_by_episode_id(id: Uuid) -> Result<Podcast, CustomError> {
        podcast_repo()
            .find_by_episode_id(id)
            .map_err(CustomError::from)?
            .map(Into::into)
            .ok_or_else(|| CustomErrorInner::NotFound(ErrorSeverity::Warning).into())
    }

    pub fn get_favorite_state(
        user_id: Uuid,
        podcast_id: Uuid,
    ) -> Result<Option<bool>, CustomError> {
        favorite_repo()
            .find_by_user_id_and_podcast_id(user_id, podcast_id)
            .map(|opt| opt.map(|f| f.favored))
            .map_err(CustomError::from)
    }

    #[tracing::instrument]
    pub fn get_all_podcasts_raw() -> Result<Vec<Podcast>, CustomError> {
        podcast_repo()
            .find_all()
            .map_err(CustomError::from)
            .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    pub fn find_by_rss_feed(
        rss_feed: &str,
    ) -> Result<Option<podfetch_catalog::domain::podcast::Podcast>, CustomError> {
        podcast_repo()
            .find_by_rss_feed(rss_feed)
            .map_err(CustomError::from)
    }

    pub fn find_by_id(
        id: Uuid,
    ) -> Result<Option<podfetch_catalog::domain::podcast::Podcast>, CustomError> {
        podcast_repo().find_by_id(id).map_err(CustomError::from)
    }

    pub fn get_podcast_by_rss_feed(rss_feed: &str) -> Result<Podcast, CustomError> {
        podcast_repo()
            .find_by_rss_feed(rss_feed)
            .map_err(CustomError::from)?
            .map(Into::into)
            .ok_or_else(|| CustomErrorInner::NotFound(ErrorSeverity::Warning).into())
    }

    pub fn get_podcast_by_directory_id(podcast_id: &str) -> Result<Option<Podcast>, CustomError> {
        podcast_repo()
            .find_by_directory_id(podcast_id)
            .map_err(CustomError::from)
            .map(|opt| opt.map(Into::into))
    }

    pub fn find_podcast_by_image_path(path: &str) -> Result<Option<Podcast>, CustomError> {
        podcast_repo()
            .find_by_image_path(path)
            .map_err(CustomError::from)
            .map(|opt| opt.map(Into::into))
    }

    pub fn update_podcast_name(id: Uuid, new_name: &str) -> Result<(), CustomError> {
        podcast_repo()
            .update_name(id, new_name)
            .map_err(CustomError::from)
    }

    pub fn delete_podcast(id: Uuid) -> Result<(), CustomError> {
        podcast_repo().delete(id).map_err(CustomError::from)
    }

    pub fn delete_favorites_by_user_id(user_id: Uuid) -> Result<(), CustomError> {
        favorite_repo()
            .delete_by_user_id(user_id)
            .map_err(CustomError::from)
    }

    pub fn add_podcast_to_database(
        collection_name: &str,
        collection_id: &str,
        feed_url: &str,
        image_url_1: &str,
        directory_name_to_insert: &str,
    ) -> Result<Podcast, CustomError> {
        podcast_repo()
            .create(NewPodcast {
                name: collection_name.to_string(),
                directory_id: collection_id.to_string(),
                rssfeed: feed_url.to_string(),
                image_url: image_url_1.to_string(),
                directory_name: directory_name_to_insert.to_string(),
                added_by: None,
            })
            .map(Into::into)
            .map_err(CustomError::from)
    }

    pub fn update_original_image_url(
        original_image_url_to_set: &str,
        podcast_id_to_find: Uuid,
    ) -> Result<(), CustomError> {
        podcast_repo()
            .update_original_image_url(podcast_id_to_find, original_image_url_to_set)
            .map_err(CustomError::from)
    }

    pub fn update_podcast_image(
        directory_id: &str,
        image_url: &str,
        download_location: &str,
    ) -> Result<(), CustomError> {
        podcast_repo()
            .update_image_url_and_download_location(directory_id, image_url, download_location)
            .map_err(CustomError::from)
    }

    pub fn update_podcast_urls_on_redirect(
        podcast_id_to_update: Uuid,
        new_url: &str,
    ) -> Result<(), CustomError> {
        podcast_repo()
            .update_rss_feed(podcast_id_to_update, new_url)
            .map_err(CustomError::from)
    }

    pub fn query_for_podcast(query: &str) -> Result<Vec<PodcastEpisode>, CustomError> {
        use podfetch_catalog::domain::podcast_episode::PodcastEpisodeRepository;
        podfetch_catalog::persistence::podcast_episode::DieselPodcastEpisodeRepository::new(
            podfetch_persistence::db::database(),
        )
        .search_by_text(query)
        .map(|episodes| episodes.into_iter().map(Into::into).collect())
        .map_err(CustomError::from)
    }

    /// Whether `user` may add `adding` more podcasts to the server: admins
    /// and uploaders always, other users only within USER_PODCAST_LIMIT
    /// (0 = not at all). Every endpoint that subscribes the server to a feed -
    /// native API and compatibility layers alike - must call this.
    pub fn ensure_can_add_podcasts(user: &Principal, adding: u32) -> Result<(), CustomError> {
        let current_count = podcast_repo()
            .count_by_added_by(user.id.into())
            .map_err(CustomError::from)?;
        crate::podcast::check_podcast_add_permission::<CustomError>(
            user.is_privileged_user(),
            ENVIRONMENT_SERVICE.user_podcast_limit,
            current_count,
            adding,
        )
        .map_err(crate::podcast::map_podcast_error)
    }

    /// Mirror subscription changes made on a GPodder device that shares its
    /// subscriptions with the web UI into the user's favorites.
    ///
    /// Only feeds that already exist in PodFetch can become favorites; the
    /// subscription rows themselves have already been written by the caller.
    /// Sets the user's favorite flag. gPodder sync mirrors the change into
    /// the subscriptions when it handles `library.podcast_favor_changed`.
    pub fn update_favor_podcast(id: Uuid, favored: bool, user_id: Uuid) -> Result<(), CustomError> {
        favorite_repo()
            .update_podcast_favor(id, favored, user_id)
            .map_err(CustomError::from)?;
        crate::podcast_events::publish(&podfetch_library::events::PodcastFavorChanged {
            user_id: user_id.into(),
            podcast_id: id,
            favored,
        });
        Ok(())
    }

    /// Resolve a podcast by its pre-migration integer id (backwards-compat).
    pub fn get_podcast_by_legacy_id(legacy_id: i64) -> Result<Podcast, CustomError> {
        podcast_repo()
            .find_by_legacy_id(legacy_id)
            .map_err(CustomError::from)?
            .map(Into::into)
            .ok_or_else(|| CustomErrorInner::NotFound(ErrorSeverity::Warning).into())
    }

    /// Resolve a podcast by its pre-migration integer id, returning the raw row
    /// (panics on db error, mirroring [`get_podcast_by_id`]).
    pub fn get_podcast_by_legacy_id_raw(legacy_id: i64) -> Option<Podcast> {
        podcast_repo()
            .find_by_legacy_id(legacy_id)
            .ok()
            .flatten()
            .map(Into::into)
    }

    pub fn get_favored_podcasts(
        requester: Principal,
        api_key: Option<&str>,
        server_url: &str,
    ) -> Result<Vec<PodcastDto>, CustomError> {
        let podcasts = podcast_repo()
            .search(PodcastQuery {
                order: OrderCriteria::Asc,
                order_option: OrderOption::Title,
                title: None,
                only_ids: Some(Self::favored_podcast_ids(requester.id.into())?),
            })
            .map_err(CustomError::from)?;
        Self::with_library_context(podcasts, requester.id.into(), api_key, server_url)
    }

    /// The user's favorite flags for the given podcasts.
    pub fn favorites_of_podcasts(
        user_id: Uuid,
        podcast_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, bool>, CustomError> {
        Ok(favorite_repo()
            .find_by_user_and_podcasts(user_id, podcast_ids)
            .map_err(CustomError::from)?
            .into_iter()
            .map(|favorite| (favorite.podcast_id, favorite.favored))
            .collect())
    }

    pub fn favored_podcast_ids(user_id: Uuid) -> Result<Vec<Uuid>, CustomError> {
        Ok(favorite_repo()
            .find_favored_by_user_id(user_id)
            .map_err(CustomError::from)?
            .into_iter()
            .map(|favorite| favorite.podcast_id)
            .collect())
    }

    /// API composition: podcasts come from the catalog, favorite flag and
    /// tags from the library, fetched in bulk for the whole list.
    fn with_library_context(
        podcasts: Vec<podfetch_catalog::domain::podcast::Podcast>,
        user_id: Uuid,
        api_key: Option<&str>,
        server_url: &str,
    ) -> Result<Vec<PodcastDto>, CustomError> {
        let ids: Vec<Uuid> = podcasts.iter().map(|podcast| podcast.id).collect();
        let favored: HashMap<Uuid, bool> = favorite_repo()
            .find_by_user_and_podcasts(user_id, &ids)
            .map_err(CustomError::from)?
            .into_iter()
            .map(|favorite| (favorite.podcast_id, favorite.favored))
            .collect();
        let mut tags = crate::library::tags().get_tags_of_podcasts(&ids, user_id)?;
        Ok(podcasts
            .into_iter()
            .map(|podcast| {
                let favorite = favored.get(&podcast.id).copied();
                let podcast_tags = tags.remove(&podcast.id).unwrap_or_default();
                map_podcast_with_context_to_dto(
                    podcast,
                    favorite,
                    podcast_tags,
                    api_key,
                    server_url,
                )
            })
            .collect())
    }

    pub fn update_active_podcast(id: Uuid) -> Result<(), CustomError> {
        let found = podcast_repo()
            .find_by_id(id)
            .map_err(CustomError::from)?
            .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(ErrorSeverity::Warning)))?;
        podcast_repo()
            .update_active(id, !found.active)
            .map_err(CustomError::from)
    }

    fn compute_podindex_header() -> HeaderMap {
        let seconds = chrono::Utc::now().timestamp();
        let api_key = &ENVIRONMENT_SERVICE.podindex_api_key;
        let api_secret = &ENVIRONMENT_SERVICE.podindex_api_secret;
        let digest = Sha1::digest(format!("{api_key}{api_secret}{seconds}"));
        let hashed_auth_key: String = digest.iter().map(|b| format!("{b:02x}")).collect();

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(COMMON_USER_AGENT));
        headers.insert("X-Auth-Date", HeaderValue::from(seconds));
        // Hex digest and digits are always valid header values; only a
        // misconfigured API key can be rejected.
        if let Ok(hashed) = HeaderValue::from_str(&hashed_auth_key) {
            headers.insert(AUTHORIZATION, hashed);
        }
        match HeaderValue::from_str(api_key) {
            Ok(key) => {
                headers.insert("X-Auth-Key", key);
            }
            Err(_) => tracing::error!("PODINDEX_API_KEY contains characters invalid in headers"),
        }
        headers
    }

    pub fn get_podcasts_by_ids(ids: &[Uuid]) -> Result<Vec<Podcast>, CustomError> {
        podcast_repo()
            .search(PodcastQuery {
                order: OrderCriteria::Asc,
                order_option: OrderOption::Title,
                title: None,
                only_ids: Some(ids.to_vec()),
            })
            .map(|podcasts| podcasts.into_iter().map(Into::into).collect())
            .map_err(CustomError::from)
    }

    pub fn get_podcast(podcast_id_to_be_searched: Uuid) -> Result<Podcast, CustomError> {
        podcast_repo()
            .find_by_id(podcast_id_to_be_searched)
            .map_err(CustomError::from)?
            .map(Into::into)
            .ok_or_else(|| CustomErrorInner::NotFound(ErrorSeverity::Warning).into())
    }

    pub fn get_podcasts(
        u: &Principal,
        api_key: Option<&str>,
        server_url: &str,
    ) -> Result<Vec<PodcastDto>, CustomError> {
        let podcasts = podcast_repo().find_all().map_err(CustomError::from)?;
        Self::with_library_context(podcasts, u.id.into(), api_key, server_url)
    }

    pub fn search_podcasts_favored(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        designated_user_id: Uuid,
        tag: Option<String>,
        api_key: Option<&str>,
        server_url: &str,
    ) -> Result<Vec<PodcastDto>, CustomError> {
        let podcasts = podcast_repo()
            .search(PodcastQuery {
                order,
                order_option: latest_pub,
                title,
                only_ids: Some(Self::favored_podcast_ids(designated_user_id)?),
            })
            .map_err(CustomError::from)?;
        let mut dtos =
            Self::with_library_context(podcasts, designated_user_id, api_key, server_url)?;
        if let Some(tag) = tag {
            dtos.retain(|podcast| podcast.tags.iter().any(|t| t.name == tag));
        }
        Ok(dtos)
    }

    pub fn search_podcasts(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        tag: Option<String>,
        requester: &Principal,
        api_key: Option<&str>,
        server_url: &str,
    ) -> Result<Vec<PodcastDto>, CustomError> {
        let podcasts = podcast_repo()
            .search(PodcastQuery {
                order,
                order_option: latest_pub,
                title,
                only_ids: None,
            })
            .map_err(CustomError::from)?;
        let mut dtos =
            Self::with_library_context(podcasts, requester.id.into(), api_key, server_url)?;
        if let Some(tag) = tag {
            dtos.retain(|podcast| podcast.tags.iter().any(|t| t.id == tag));
        }
        Ok(dtos)
    }
}
