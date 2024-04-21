use crate::constants::inner_constants::{
    PodcastType, COMMON_USER_AGENT, DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE, ITUNES,
    TELEGRAM_API_ENABLED,
};
use crate::models::messages::BroadcastMessage;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::web_socket_message::Lobby;
use crate::service::download_service::DownloadService;
use crate::service::file_service::FileService;
use crate::service::mapping_service::MappingService;
use std::io::Error;
use std::sync::{Arc, Mutex};

use crate::utils::podcast_builder::PodcastBuilder;
use actix::Addr;

use actix_web::web;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use log::error;
use regex::Regex;
use reqwest::blocking::ClientBuilder;
use reqwest::header::{HeaderMap, ACCEPT};
use reqwest::redirect::Policy;
use rss::{Channel, Item};

use crate::models::episode::Episode;
use crate::models::notification::Notification;
use crate::models::user::User;
use crate::DBType as DbConnection;

use crate::mutex::LockResultExt;
use crate::service::environment_service::EnvironmentService;
use crate::service::settings_service::SettingsService;
use crate::service::telegram_api::send_new_episode_notification;
use crate::utils::environment_variables::is_env_var_present_and_true;
use crate::utils::error::{map_db_error, CustomError};

pub struct PodcastEpisodeService;

impl PodcastEpisodeService {
    pub fn download_podcast_episode_if_not_locally_available(
        podcast_episode: PodcastEpisode,
        podcast: Podcast,
        lobby: Option<web::Data<Addr<Lobby>>>,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        let podcast_episode_cloned = podcast_episode.clone();
        let podcast_cloned = podcast.clone();

        match PodcastEpisode::check_if_downloaded(&podcast_episode.url, conn) {
            Ok(true) => {}
            Ok(false) => {
                let podcast_inserted =
                    Self::perform_download(&podcast_episode_cloned, podcast_cloned, conn)?;
                let mapped_dto = MappingService::map_podcastepisode_to_dto(&podcast_inserted);
                if let Some(lobby) = lobby {
                    lobby.do_send(BroadcastMessage {
                        message: format!(
                            "Episode {} is now available offline",
                            podcast_episode.name
                        ),
                        podcast: Option::from(podcast.clone()),
                        type_of: PodcastType::AddPodcastEpisode,
                        podcast_episode: Some(mapped_dto),
                        podcast_episodes: None,
                    })
                }

                if is_env_var_present_and_true(TELEGRAM_API_ENABLED) {
                    send_new_episode_notification(podcast_episode, podcast)
                }
            }

            _ => {
                error!("Error checking if podcast episode is downloaded.");
            }
        }
        Ok(())
    }

    pub fn perform_download(
        podcast_episode: &PodcastEpisode,
        podcast_cloned: Podcast,
        conn: &mut DbConnection,
    ) -> Result<PodcastEpisode, CustomError> {
        log::info!("Downloading podcast episode: {}", podcast_episode.name);
        let mut download_service = DownloadService::new();
        download_service.download_podcast_episode(podcast_episode.clone(), podcast_cloned)?;
        let podcast =
            PodcastEpisode::update_podcast_episode_status(&podcast_episode.url, "D", conn).unwrap();
        let notification = Notification {
            id: 0,
            message: podcast_episode.name.to_string(),
            created_at: chrono::Utc::now().naive_utc().to_string(),
            type_of_message: "Download".to_string(),
            status: "unread".to_string(),
        };
        Notification::insert_notification(notification, conn).unwrap();
        Ok(podcast)
    }

    pub fn get_last_n_podcast_episodes(
        conn: &mut DbConnection,
        podcast: Podcast,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        let mut settings_service = SettingsService::new();
        let settings = settings_service.get_settings(conn)?.unwrap();
        Ok(
            PodcastEpisode::get_last_n_podcast_episodes(conn, podcast.id, settings.podcast_prefill)
                .unwrap(),
        )
    }

    // Used for creating/updating podcasts
    pub fn insert_podcast_episodes(
        conn: &mut DbConnection,
        podcast: Podcast,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        let is_redirected = Arc::new(Mutex::new(false)); // Variable to store the redirection status

        let returned_data_from_podcast_insert = Self::do_request_to_podcast_server(podcast.clone());

        let channel = Channel::read_from(returned_data_from_podcast_insert.content.as_bytes());

        match channel {
            Ok(channel) => {
                if *is_redirected.clone().lock().ignore_poison() {
                    log::info!(
                        "The podcast {} has moved to {}",
                        podcast.name,
                        returned_data_from_podcast_insert.url
                    );
                    Podcast::update_podcast_urls_on_redirect(
                        podcast.id,
                        returned_data_from_podcast_insert.url,
                        conn,
                    );
                    Self::update_episodes_on_redirect(conn, channel.items())?;
                }

                Self::handle_itunes_extension(conn, &podcast, &channel)?;

                Self::update_podcast_fields(channel.clone(), podcast.id, conn)?;

                let mut podcast_inserted = Vec::new();

                Self::handle_podcast_image_insert(conn, &podcast, &channel)?;

                for item in channel.items.iter() {
                    let itunes_ext = item.clone().itunes_ext;

                    match itunes_ext {
                        Some(itunes_ext) => {
                            let enclosure = item.enclosure();
                            match enclosure {
                                Some(enclosure) => {
                                    let result = PodcastEpisode::get_podcast_episode_by_url(
                                        conn,
                                        &enclosure.url.to_string(),
                                        Some(podcast.id),
                                    );
                                    let mut duration_episode = 0;

                                    if result.is_err() {
                                        log::info!(
                                            "Skipping episode {} with error: {}",
                                            item.clone()
                                                .title
                                                .unwrap_or("with no title".to_string()),
                                            result.err().unwrap()
                                        );
                                        continue;
                                    }

                                    let result_unwrapped = result.unwrap();

                                    if let Some(result_unwrapped_non_opt) = result_unwrapped.clone()
                                    {
                                        if result_unwrapped_non_opt.clone().podcast_id != podcast.id
                                        {
                                            let inserted_episode =
                                                PodcastEpisode::insert_podcast_episodes(
                                                    conn,
                                                    podcast.clone(),
                                                    item.clone(),
                                                    Some(result_unwrapped_non_opt.image_url),
                                                    duration_episode as i32,
                                                );
                                            podcast_inserted.push(inserted_episode);
                                        }
                                    }

                                    if result_unwrapped.is_none() {
                                        // Insert new podcast episode
                                        if let Some(duration) = itunes_ext.clone().duration {
                                            duration_episode = Self::parse_duration(&duration);
                                        }

                                        let inserted_episode =
                                            PodcastEpisode::insert_podcast_episodes(
                                                conn,
                                                podcast.clone(),
                                                item.clone(),
                                                itunes_ext.image,
                                                duration_episode as i32,
                                            );
                                        podcast_inserted.push(inserted_episode);
                                    }
                                }
                                None => {
                                    log::info!(
                                        "Skipping episode {} without enclosure.",
                                        item.clone().title.unwrap_or("with no title".to_string())
                                    );
                                    continue;
                                }
                            }
                        }
                        None => {
                            let opt_enclosure = &item.enclosure;
                            let mut image_url = DEFAULT_IMAGE_URL.to_string();

                            // Also check the itunes extension map
                            if let Some(image_url_extracted) =
                                Self::extract_itunes_url_if_present(item)
                            {
                                image_url = image_url_extracted;
                            }

                            if opt_enclosure.is_none() {
                                log::info!(
                                    "Skipping episode {} without enclosure.",
                                    item.clone().title.unwrap_or("with no title".to_string())
                                );
                                continue;
                            }
                            let result = PodcastEpisode::get_podcast_episode_by_url(
                                conn,
                                &opt_enclosure.clone().unwrap().url,
                                None,
                            );
                            // We can't retrieve the duration of the podcast episode, so we set it to 0

                            if result.unwrap().is_none() {
                                let duration_episode = 0;
                                let inserted_episode = PodcastEpisode::insert_podcast_episodes(
                                    conn,
                                    podcast.clone(),
                                    item.clone(),
                                    Some(image_url),
                                    duration_episode,
                                );
                                podcast_inserted.push(inserted_episode);
                            }
                        }
                    }
                }
                Ok(podcast_inserted)
            }
            Err(e) => {
                log::info!(
                    "Error parsing podcast {} {:?} with cause {:?}",
                    podcast.name,
                    returned_data_from_podcast_insert.content,
                    e
                );
                Err(CustomError::BadRequest(format!(
                    "Error parsing podcast {} with cause {:?}",
                    podcast.name,e
                )))
            }
        }
    }

    fn extract_itunes_url_if_present(item: &Item) -> Option<String> {
        if let Some(itunes_data) = item.extensions.get(ITUNES) {
            if let Some(image_url_extracted) = itunes_data.get("image") {
                if let Some(i_val) = image_url_extracted.first() {
                    if let Some(image_attr) = i_val.attrs.get("href") {
                        return Some(image_attr.clone());
                    }
                }
            }
        }
        None
    }

    fn handle_podcast_image_insert(
        conn: &mut DbConnection,
        podcast: &Podcast,
        channel: &Channel,
    ) -> Result<(), CustomError> {
        match channel.image() {
            Some(image) => {
                Podcast::update_original_image_url(&image.url.to_string(), podcast.id, conn)?;
            }
            None => {
                let env = ENVIRONMENT_SERVICE.get().unwrap();
                let url = env.server_url.clone().to_owned() + DEFAULT_IMAGE_URL;
                Podcast::update_original_image_url(&url, podcast.id, conn)?;
            }
        }
        Ok(())
    }

    fn handle_itunes_extension(
        conn: &mut DbConnection,
        podcast: &Podcast,
        channel: &Channel,
    ) -> Result<(), CustomError> {
        if channel.itunes_ext.is_some() {
            let extension = channel.itunes_ext.clone().unwrap();

            if extension.new_feed_url.is_some() {
                let new_url = extension.new_feed_url.unwrap();
                Podcast::update_podcast_urls_on_redirect(podcast.id, new_url, conn);

                let returned_data_from_server = Self::do_request_to_podcast_server(podcast.clone());

                let channel =
                    Channel::read_from(returned_data_from_server.content.as_bytes()).unwrap();
                let items = channel.items();
                Self::update_episodes_on_redirect(conn, items)?;
            }
        }
        Ok(())
    }

    fn update_episodes_on_redirect(
        conn: &mut DbConnection,
        items: &[Item],
    ) -> Result<(), CustomError> {
        for item in items.iter() {
            match &item.guid {
                Some(guid) => {
                    let opt_found_podcast_episode =
                        Self::get_podcast_episode_by_guid(conn, &guid.value)?;
                    if let Some(found_podcast_episode) = opt_found_podcast_episode {
                        let mut podcast_episode = found_podcast_episode.clone();
                        podcast_episode.url = item.enclosure.as_ref().unwrap().url.to_string();
                        PodcastEpisode::update_podcast_episode(conn, podcast_episode);
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
        conn: &mut DbConnection,
        guid_to_search: &str,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(guid.eq(guid_to_search))
            .first::<PodcastEpisode>(conn)
            .optional()
            .map_err(map_db_error)
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

    pub fn get_url_file_suffix(url: &str) -> Result<String, Error> {
        let re = Regex::new(r"\.(\w+)(?:\?.*)?$").unwrap();
        let capture = re.captures(url);
        if capture.is_none() {
            return Err(Error::new(std::io::ErrorKind::Other, "No"));
        }
        return Ok(capture.unwrap().get(1).unwrap().as_str().to_string());
    }

    pub fn query_for_podcast(
        query: &str,
        conn: &mut DbConnection,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        let podcasts = Podcast::query_for_podcast(query, conn)?;
        let podcast_dto = podcasts
            .iter()
            .map(MappingService::map_podcastepisode_to_dto)
            .collect::<Vec<PodcastEpisode>>();
        Ok(podcast_dto)
    }

    pub fn find_all_downloaded_podcast_episodes(
        conn: &mut DbConnection,
        env: &EnvironmentService,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        let result = PodcastEpisode::get_episodes(conn);
        Self::map_rss_podcast_episodes(env, result)
    }

    pub fn find_all_downloaded_podcast_episodes_with_top_k(
        conn: &mut DbConnection,
        top_k: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        PodcastEpisode::get_podcast_episodes_by_podcast_to_k(conn, top_k)
    }

    fn map_rss_podcast_episodes(
        env: &EnvironmentService,
        result: Vec<PodcastEpisode>,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Ok(result
            .iter()
            .map(|podcast| {
                let mut podcast_episode_dto = MappingService::map_podcastepisode_to_dto(podcast);
                if podcast_episode_dto.is_downloaded() {
                    let local_url = Self::map_to_local_url(&podcast_episode_dto.clone().local_url);
                    let local_image_url =
                        Self::map_to_local_url(&podcast_episode_dto.clone().local_image_url);

                    podcast_episode_dto.local_image_url = env.server_url.clone() + &local_image_url;
                    podcast_episode_dto.local_url = env.server_url.clone() + &local_url;

                    podcast_episode_dto
                } else {
                    podcast_episode_dto.local_image_url = podcast_episode_dto.clone().image_url;
                    podcast_episode_dto.local_url = podcast_episode_dto.clone().url;

                    podcast_episode_dto
                }
            })
            .collect::<Vec<PodcastEpisode>>())
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
        podcast_id: i32,
        conn: &mut DbConnection,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        let result = PodcastEpisode::get_episodes_by_podcast_id(podcast_id, conn);
        Self::map_rss_podcast_episodes(ENVIRONMENT_SERVICE.get().unwrap(), result)
    }

    fn update_podcast_fields(
        feed: Channel,
        podcast_id: i32,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        let itunes = feed.clone().itunes_ext;

        if let Some(itunes) = itunes {
            let constructed_extra_fields = PodcastBuilder::new(podcast_id)
                .author(itunes.author)
                .last_build_date(feed.last_build_date)
                .description(feed.description)
                .language(feed.language)
                .keywords(itunes.categories)
                .build();

            Podcast::update_podcast_fields(constructed_extra_fields, conn)?;
        }

        Ok(())
    }

    pub fn cleanup_old_episodes(days: i32, conn: &mut DbConnection) {
        let old_podcast_episodes = PodcastEpisode::get_podcast_episodes_older_than_days(days, conn);

        log::info!("Cleaning up {} old episodes", old_podcast_episodes.len());
        for old_podcast_episode in old_podcast_episodes {
            let res = FileService::cleanup_old_episode(old_podcast_episode.clone());

            match res {
                Ok(_) => {
                    PodcastEpisode::update_download_status_of_episode(
                        old_podcast_episode.clone().id,
                        conn,
                    );
                }
                Err(e) => {
                    println!("Error deleting podcast episode.{}", e);
                }
            }
        }
    }

    pub fn get_podcast_episodes_of_podcast(
        conn: &mut DbConnection,
        id_num: i32,
        last_id: Option<String>,
        user: User,
    ) -> Result<Vec<(PodcastEpisode, Option<Episode>)>, CustomError> {
        PodcastEpisode::get_podcast_episodes_of_podcast(conn, id_num, last_id, user)
    }

    pub fn get_podcast_episode_by_id(
        conn: &mut DbConnection,
        id_num: &str,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        PodcastEpisode::get_podcast_episode_by_id(conn, id_num)
    }

    fn do_request_to_podcast_server(podcast: Podcast) -> RequestReturnType {
        let is_redirected = Arc::new(Mutex::new(false)); // Variable to store the redirection status
        let client = ClientBuilder::new()
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
            .unwrap();
        let mut header_map = HeaderMap::new();
        header_map.append(
            ACCEPT,
            "application/rss+xml,application/xml".parse().unwrap(),
        );
        header_map.append("User-Agent", COMMON_USER_AGENT.parse().unwrap());
        let result = client
            .get(podcast.clone().rssfeed)
            .headers(header_map)
            .send()
            .unwrap();
        let url = result.url().clone().to_string();
        let content = result.text().unwrap().clone();

        RequestReturnType { url, content }
    }

    pub(crate) fn delete_podcast_episode_locally(
        episode_id: &str,
        conn: &mut DbConnection,
    ) -> Result<PodcastEpisode, CustomError> {
        let episode = PodcastEpisode::get_podcast_episode_by_id(conn, episode_id)?;
        if episode.is_none() {
            return Err(CustomError::NotFound);
        }
        FileService::cleanup_old_episode(episode.clone().unwrap())?;
        PodcastEpisode::update_download_status_of_episode(episode.clone().unwrap().id, conn);
        PodcastEpisode::update_deleted(conn, episode_id, true)?;
        Ok(episode.unwrap())
    }

    pub fn get_track_number_for_episode(
        conn: &mut DbConnection,
        podcast_id: i32,
        date_of_recording_to_search: &str,
    ) -> Result<i64, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;

        podcast_episodes
            .filter(crate::dbconfig::schema::podcast_episodes::podcast_id.eq(podcast_id))
            .filter(
                crate::dbconfig::schema::podcast_episodes::date_of_recording
                    .le(date_of_recording_to_search),
            )
            .count()
            .get_result::<i64>(conn)
            .map_err(map_db_error)
    }
}

struct RequestReturnType {
    pub url: String,
    pub content: String,
}
