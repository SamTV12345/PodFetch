use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_persistence::podcast::PodcastEntity as Podcast;

use crate::server::ChatServerHandle;
use crate::services::file::service::FileService;
use crate::services::podcast::metadata::PodcastExtra;
use podfetch_persistence::db::database;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::tag::service::TagService;
use crate::controllers::controller_utils::unwrap_string;
use common_infrastructure::error::ErrorSeverity::{Critical, Error};
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error};
use common_infrastructure::http::COMMON_USER_AGENT;
use common_infrastructure::http::get_http_client;
use common_infrastructure::runtime::{ENVIRONMENT_SERVICE, ITUNES_URL};
use podfetch_domain::favorite::FavoriteRepository;
use podfetch_domain::ordering::{OrderCriteria, OrderOption};
use podfetch_domain::podcast::{NewPodcast, PodcastMetadataUpdate, PodcastRepository};
use podfetch_domain::user::User;
use podfetch_persistence::db::PersistenceError;
use podfetch_persistence::favorite::DieselFavoriteRepository;
use podfetch_persistence::podcast::DieselPodcastRepository;
use crate::podcast::{ItunesWrapper, PodcastDto, PodcastInsertModel, PodindexResponse};
use crate::podcast::map_podcast_with_context_to_dto;
use reqwest::header::{HeaderMap, HeaderValue};
use rss::Channel;
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::thread;
use std::time::SystemTime;
use tokio::task::spawn_blocking;

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
        let result = get_http_client(&ENVIRONMENT_SERVICE)
            .get(ITUNES_URL)
            .query(&query)
            .send()
            .await
            .unwrap();
        log::info!("Found podcast: {}", result.url());
        let res_of_search = result.json().await;

        match res_of_search {
            Ok(res) => res,
            _ => {
                log::error!(
                    "Error searching for podcast: {}",
                    res_of_search.err().unwrap()
                );
                ItunesWrapper::default()
            }
        }
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

        log::info!("Found podcast: {}", result.url());

        let status = result.status();
        let possible_json = result.text().await.map_err(map_reqwest_error)?;

        if status.is_client_error() || status.is_server_error() {
            log::error!("Error searching for podcast: {possible_json}");
            Err(CustomErrorInner::BadRequest(possible_json, Error).into())
        } else {
            let res_of_search = serde_json::from_str(&possible_json);

            if let Ok(res) = res_of_search {
                Ok(res)
            } else {
                log::error!(
                    "Error searching for podcast: {}",
                    res_of_search.err().unwrap()
                );
                Ok(serde_json::from_str("{}").unwrap())
            }
        }
    }

    pub async fn insert_podcast_from_podindex(id: i32) -> Result<Podcast, CustomError> {
        let resp = get_http_client(&ENVIRONMENT_SERVICE)
            .get(format!(
                "https://api.podcastindex.org/api/1.0/podcasts/byfeedid?id={}",
                &id.to_string()
            ))
            .headers(Self::compute_podindex_header())
            .send()
            .await
            .unwrap();

        println!("Result: {resp:?}");

        let podcast = resp.json::<Value>().await.unwrap();

        Self::handle_insert_of_podcast(
            PodcastInsertModel {
                title: unwrap_string(&podcast["feed"]["title"]),
                id,
                feed_url: unwrap_string(&podcast["feed"]["url"]),
                image_url: unwrap_string(&podcast["feed"]["image"]),
            },
            None,
        )
        .await
    }

    pub async fn handle_insert_of_podcast(
        podcast_insert: PodcastInsertModel,
        channel: Option<Channel>,
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
            })
            .map(Into::into)
            .map_err(CustomError::from)?;

        FileService::download_podcast_image(
            &inserted_podcast.directory_name.clone().to_string(),
            podcast_insert.image_url.clone().to_string(),
            &podcast_insert.id.clone().to_string(),
        )
        .await?;
        let podcast: Option<Podcast> = podcast_repo()
            .find_by_track_id(podcast_insert.id)
            .map(|opt| opt.map(Into::into))
            .map_err(CustomError::from)?;
        match podcast {
            Some(podcast) => {
                ChatServerHandle::broadcast_podcast_downloaded(podcast.clone());
                spawn_blocking(move || {
                    log::debug!("Inserting podcast episodes of {}", podcast.name);
                    let inserted_podcasts =
                        PodcastEpisodeService::insert_podcast_episodes(&podcast).unwrap();

                    ChatServerHandle::broadcast_added_podcast_episodes(
                        &podcast,
                        inserted_podcasts.clone(),
                    );
                    if let Err(e) = Self::schedule_episode_download(&podcast) {
                        log::error!("Error scheduling episode download: {e}");
                    }
                })
                .await
                .unwrap();
                Ok(inserted_podcast)
            }
            None => {
                panic!("No podcast found")
            }
        }
    }

    pub fn schedule_episode_download(podcast: &Podcast) -> Result<(), CustomError> {
        const MAX_PARALLEL_DOWNLOADS: usize = 3;
        let settings =
            crate::services::settings::service::SettingsService::shared().get_settings()?;
        let podcast_settings = PodcastSettingsService::get_settings_for_podcast(podcast.id)?;
        match settings {
            Some(settings) => {
                if (podcast_settings.is_some() && podcast_settings.unwrap().auto_download)
                    || settings.auto_download
                {
                    let result =
                        PodcastEpisodeService::get_last_n_podcast_episodes(podcast.clone())?;
                    for chunk in result.chunks(MAX_PARALLEL_DOWNLOADS) {
                        let mut handles = Vec::with_capacity(chunk.len());
                        for podcast_episode in chunk.iter().cloned() {
                            if podcast_episode.deleted {
                                continue;
                            }
                            let podcast_for_thread = podcast.clone();
                            handles.push(thread::spawn(move || {
                                if let Err(err) = PodcastEpisodeService::download_podcast_episode_if_not_locally_available(
                                    podcast_episode,
                                    podcast_for_thread,
                                ) {
                                    log::error!("Error downloading podcast episode: {err}");
                                }
                            }));
                        }

                        for handle in handles {
                            if let Err(err) = handle.join() {
                                log::error!(
                                    "Error joining download worker for podcast {}: {:?}",
                                    podcast.id,
                                    err
                                );
                            }
                        }
                    }
                }
                Ok(())
            }
            None => {
                log::error!("Error getting settings");
                Err(CustomErrorInner::Unknown(Critical).into())
            }
        }
    }

    pub fn refresh_podcast(podcast: &Podcast) -> Result<(), CustomError> {
        log::info!("Refreshing podcast: {}", podcast.name);
        PodcastEpisodeService::insert_podcast_episodes(podcast)?;
        Self::schedule_episode_download(podcast)
    }

    pub fn get_podcast_by_episode_id(id: i32) -> Result<Podcast, CustomError> {
        podcast_repo()
            .find_by_episode_id(id)
            .map_err(CustomError::from)?
            .map(Into::into)
            .ok_or_else(|| CustomErrorInner::NotFound(ErrorSeverity::Warning).into())
    }

    pub fn get_favorite_state(username: &str, podcast_id: i32) -> Result<Option<bool>, CustomError> {
        favorite_repo()
            .find_by_username_and_podcast_id(username, podcast_id)
            .map(|opt| opt.map(|f| f.favored))
            .map_err(CustomError::from)
    }

    pub fn get_all_podcasts_raw() -> Result<Vec<Podcast>, CustomError> {
        podcast_repo()
            .find_all()
            .map_err(CustomError::from)
            .map(|rows| rows.into_iter().map(Into::into).collect())
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

    pub fn update_podcast_name(id: i32, new_name: &str) -> Result<(), CustomError> {
        podcast_repo()
            .update_name(id, new_name)
            .map_err(CustomError::from)
    }

    pub fn delete_podcast(id: i32) -> Result<(), CustomError> {
        podcast_repo().delete(id).map_err(CustomError::from)
    }

    pub fn delete_favorites_by_username(username: &str) -> Result<(), CustomError> {
        favorite_repo()
            .delete_by_username(username)
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
            })
            .map(Into::into)
            .map_err(CustomError::from)
    }

    pub fn update_podcast_fields(podcast_extra: PodcastExtra) -> Result<usize, CustomError> {
        podcast_repo()
            .update_metadata(PodcastMetadataUpdate {
                id: podcast_extra.id,
                author: if podcast_extra.author.is_empty() {
                    None
                } else {
                    Some(podcast_extra.author)
                },
                keywords: if podcast_extra.keywords.is_empty() {
                    None
                } else {
                    Some(podcast_extra.keywords)
                },
                explicit: Some(podcast_extra.explicit.to_string()),
                language: if podcast_extra.language.is_empty() {
                    None
                } else {
                    Some(podcast_extra.language)
                },
                description: if podcast_extra.description.is_empty() {
                    None
                } else {
                    Some(podcast_extra.description)
                },
                last_build_date: if podcast_extra.last_build_date.is_empty() {
                    None
                } else {
                    Some(podcast_extra.last_build_date)
                },
                guid: podcast_extra.guid,
            })
            .map(|_| 1)
            .map_err(|e: PersistenceError| e.into())
    }

    pub fn update_original_image_url(
        original_image_url_to_set: &str,
        podcast_id_to_find: i32,
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

    pub fn update_podcast_urls_on_redirect(podcast_id_to_update: i32, new_url: &str) {
        podcast_repo()
            .update_rss_feed(podcast_id_to_update, new_url)
            .expect("Error updating podcast episode");
    }

    pub fn query_for_podcast(query: &str) -> Result<Vec<PodcastEpisode>, CustomError> {
        use podfetch_persistence::db::get_connection;
        use podfetch_persistence::schema::podcast_episodes::dsl::*;
        use diesel::BoolExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use diesel::TextExpressionMethods;

        podcast_episodes
            .filter(
                name.like(format!("%{query}%"))
                    .or(description.like(format!("%{query}%"))),
            )
            .load::<PodcastEpisode>(&mut get_connection())
            .map_err(|e| common_infrastructure::error::map_db_error(e, ErrorSeverity::Critical))
    }

    pub fn update_favor_podcast(id: i32, x: bool, username: &str) -> Result<(), CustomError> {
        favorite_repo()
            .update_podcast_favor(id, x, username)
            .map_err(CustomError::from)
    }

    pub fn get_podcast_by_id(id: i32) -> Podcast {
        podcast_repo()
            .find_by_id(id)
            .ok()
            .flatten()
            .map(Into::into)
            .unwrap()
    }

    pub fn get_favored_podcasts(requester: User) -> Result<Vec<PodcastDto>, CustomError> {
        let result = favorite_repo()
            .get_favored_podcasts(&requester.username)
            .map_err(CustomError::from)?;
        let mapped_result = result
            .iter()
            .map(|podcast| {
                let tags = TagService::default_service()
                    .get_tags_of_podcast(podcast.podcast.id, &requester.username)
                    .unwrap();
                map_podcast_with_context_to_dto(
                    podcast.podcast.clone(),
                    Some(podcast.favorite.favored),
                    tags,
                    &requester,
                )
            })
            .collect::<Vec<PodcastDto>>();
        Ok(mapped_result)
    }

    pub fn update_active_podcast(id: i32) -> Result<(), CustomError> {
        let found = podcast_repo()
            .find_by_id(id)
            .map_err(CustomError::from)?
            .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(ErrorSeverity::Warning)))?;
        podcast_repo()
            .update_active(id, !found.active)
            .map_err(CustomError::from)
    }

    fn compute_podindex_header() -> HeaderMap {
        let seconds = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut headers = HeaderMap::new();
        let non_hashed_string = format!(
            "{}{}{}",
            ENVIRONMENT_SERVICE.podindex_api_key.clone(),
            &*ENVIRONMENT_SERVICE.podindex_api_secret.clone(),
            &seconds.to_string()
        );
        let mut hasher = Sha1::new();

        hasher.update(non_hashed_string);

        let hashed_auth_key = format!("{:x}", hasher.finalize());

        headers.insert(
            "User-Agent",
            HeaderValue::from_str(COMMON_USER_AGENT).unwrap(),
        );
        headers.insert(
            "X-Auth-Key",
            HeaderValue::from_str(&ENVIRONMENT_SERVICE.podindex_api_key).unwrap(),
        );
        headers.insert(
            "X-Auth-Date",
            HeaderValue::from_str(&seconds.to_string()).unwrap(),
        );
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&hashed_auth_key).unwrap(),
        );

        headers
    }

    pub fn get_podcast(podcast_id_to_be_searched: i32) -> Result<Podcast, CustomError> {
        podcast_repo()
            .find_by_id(podcast_id_to_be_searched)
            .map_err(CustomError::from)?
            .map(Into::into)
            .ok_or_else(|| CustomErrorInner::NotFound(ErrorSeverity::Warning).into())
    }

    pub fn get_podcasts(u: &User) -> Result<Vec<PodcastDto>, CustomError> {
        let result = podcast_repo()
            .find_all_with_favorites(&u.username)
            .map_err(CustomError::from)?;
        let mapped_result = result
            .iter()
            .map(|podcast| {
                let tags = TagService::default_service()
                    .get_tags_of_podcast(podcast.podcast.id, &u.username)
                    .unwrap();
                map_podcast_with_context_to_dto(
                    podcast.podcast.clone(),
                    podcast.favorite.clone().map(|f| f.favored),
                    tags,
                    u,
                )
            })
            .collect::<Vec<PodcastDto>>();
        Ok(mapped_result)
    }

    pub fn search_podcasts_favored(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        designated_username: String,
        tag: Option<String>,
        requester: &User,
    ) -> Result<Vec<PodcastDto>, CustomError> {
        let podcasts = favorite_repo()
            .search_podcasts_favored(order, title, latest_pub, &designated_username)
            .map_err(CustomError::from)?
            .iter()
            .filter(|podcast| {
                if let Some(tag) = &tag {
                    return podcast.tags.iter().filter(|p| p.name == *tag).count() > 0;
                }
                true
            })
            .map(|p| {
                map_podcast_with_context_to_dto(
                    p.podcast.clone(),
                    Some(p.favorite.favored),
                    p.tags.iter().cloned().map(Into::into).collect(),
                    requester,
                )
            })
            .collect::<Vec<PodcastDto>>();

        Ok(podcasts)
    }

    pub fn search_podcasts(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        tag: Option<String>,
        requester: &User,
    ) -> Result<Vec<PodcastDto>, CustomError> {
        let podcasts = favorite_repo()
            .search_podcasts(order, title, latest_pub, &requester.username)
            .map_err(CustomError::from)?
            .iter()
            .filter(|podcast| {
                if let Some(tag) = &tag {
                    return podcast.tags.iter().filter(|p| p.id == *tag).count() > 0;
                }
                true
            })
            .map(|p| {
                map_podcast_with_context_to_dto(
                    p.podcast.clone(),
                    p.favorite.as_ref().map(|f| f.favored),
                    p.tags.iter().cloned().map(Into::into).collect(),
                    requester,
                )
            })
            .collect::<Vec<PodcastDto>>();
        Ok(podcasts)
    }
}


