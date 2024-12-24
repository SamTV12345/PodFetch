use crate::constants::inner_constants::{
    PodcastType, COMMON_USER_AGENT, DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE, ITUNES, MAIN_ROOM,
    TELEGRAM_API_ENABLED,
};
use crate::models::messages::BroadcastMessage;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::service::download_service::DownloadService;
use crate::service::file_service::FileService;
use std::io::Error;
use std::sync::{Arc, Mutex};

use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::controllers::server::ChatServerHandle;
use crate::models::episode::Episode;
use crate::models::notification::Notification;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_settings::PodcastSetting;
use crate::models::user::User;
use crate::mutex::LockResultExt;
use crate::service::settings_service::SettingsService;
use crate::service::telegram_api::send_new_episode_notification;
use crate::utils::environment_variables::is_env_var_present_and_true;
use crate::utils::error::{map_db_error, CustomError};
use crate::utils::podcast_builder::PodcastBuilder;
use crate::utils::reqwest_client::get_sync_client;
use actix_web::web;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use log::error;
use regex::Regex;
use reqwest::header::{HeaderMap, ACCEPT};
use reqwest::redirect::Policy;
use rss::{Channel, Item};

pub struct PodcastEpisodeService;

impl PodcastEpisodeService {
    pub fn download_podcast_episode_if_not_locally_available(
        podcast_episode: PodcastEpisode,
        podcast: Podcast,
        lobby: Option<web::Data<ChatServerHandle>>,
    ) -> Result<(), CustomError> {
        let podcast_episode_cloned = podcast_episode.clone();
        let podcast_cloned = podcast.clone();

        match PodcastEpisode::check_if_downloaded(&podcast_episode.url) {
            Ok(true) => {}
            Ok(false) => {
                let podcast_inserted =
                    Self::perform_download(&podcast_episode_cloned, podcast_cloned)?;
                let mapped_dto: PodcastEpisodeDto = podcast_inserted.into();
                if let Some(lobby) = lobby {
                    let podcast: PodcastDto = podcast.clone().into();
                    lobby.send_broadcast_sync(
                        MAIN_ROOM.parse().unwrap(),
                        serde_json::to_string(&BroadcastMessage {
                            message: format!(
                                "Episode {} is now available offline",
                                podcast_episode.name
                            ),
                            podcast: Option::from(podcast),
                            type_of: PodcastType::AddPodcastEpisode,
                            podcast_episode: Some(mapped_dto),
                            podcast_episodes: None,
                        })
                        .unwrap(),
                    );
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
    ) -> Result<PodcastEpisode, CustomError> {
        log::info!("Downloading podcast episode: {}", podcast_episode.name);
        DownloadService::download_podcast_episode(podcast_episode.clone(), podcast_cloned)?;
        let podcast = PodcastEpisode::update_podcast_episode_status(&podcast_episode.url, "D")?;
        let notification = Notification {
            id: 0,
            message: podcast_episode.name.to_string(),
            created_at: chrono::Utc::now().naive_utc().to_string(),
            type_of_message: "Download".to_string(),
            status: "unread".to_string(),
        };
        Notification::insert_notification(notification)?;
        Ok(podcast)
    }

    pub fn get_last_n_podcast_episodes(
        podcast: Podcast,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        let podcast_settings = PodcastSetting::get_settings(podcast.id)?;
        let settings = SettingsService::get_settings()?.unwrap();
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

        PodcastEpisode::get_last_n_podcast_episodes(podcast.id, n_episodes)
    }

    // Used for creating/updating podcasts
    pub fn insert_podcast_episodes(podcast: Podcast) -> Result<Vec<PodcastEpisode>, CustomError> {
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
                    );
                    Self::update_episodes_on_redirect(channel.items())?;
                }

                Self::handle_itunes_extension(&podcast, &channel)?;

                Self::update_podcast_fields(channel.clone(), podcast.id)?;

                let mut podcast_inserted = Vec::new();

                Self::handle_podcast_image_insert(&podcast, &channel)?;

                for item in channel.items.iter() {
                    let itunes_ext = item.clone().itunes_ext;

                    match itunes_ext {
                        Some(itunes_ext) => {
                            let enclosure = item.enclosure();
                            match enclosure {
                                Some(enclosure) => {
                                    let result = PodcastEpisode::get_podcast_episode_by_url(
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
                                &opt_enclosure.clone().unwrap().url,
                                None,
                            );
                            // We can't retrieve the duration of the podcast episode, so we set it to 0

                            if result?.is_none() {
                                let duration_episode = 0;
                                let inserted_episode = PodcastEpisode::insert_podcast_episodes(
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
                    podcast.name, e
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
        podcast: &Podcast,
        channel: &Channel,
    ) -> Result<(), CustomError> {
        match channel.image() {
            Some(image) => {
                Podcast::update_original_image_url(&image.url.to_string(), podcast.id)?;
            }
            None => {
                let env = ENVIRONMENT_SERVICE.get().unwrap();
                let url = env.server_url.clone().to_owned() + DEFAULT_IMAGE_URL;
                Podcast::update_original_image_url(&url, podcast.id)?;
            }
        }
        Ok(())
    }

    fn handle_itunes_extension(podcast: &Podcast, channel: &Channel) -> Result<(), CustomError> {
        if channel.itunes_ext.is_some() {
            let extension = channel.itunes_ext.clone().unwrap();

            if extension.new_feed_url.is_some() {
                let new_url = extension.new_feed_url.unwrap();
                Podcast::update_podcast_urls_on_redirect(podcast.id, new_url);

                let returned_data_from_server = Self::do_request_to_podcast_server(podcast.clone());

                let channel =
                    Channel::read_from(returned_data_from_server.content.as_bytes()).unwrap();
                let items = channel.items();
                Self::update_episodes_on_redirect(items)?;
            }
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
                        PodcastEpisode::update_podcast_episode(podcast_episode);
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
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(guid.eq(guid_to_search))
            .first::<PodcastEpisode>(&mut get_connection())
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
        Ok(capture.unwrap().get(1).unwrap().as_str().to_string())
    }

    pub fn query_for_podcast(query: &str) -> Result<Vec<PodcastEpisode>, CustomError> {
        Podcast::query_for_podcast(query)
    }

    pub fn find_all_downloaded_podcast_episodes() -> Result<Vec<PodcastEpisode>, CustomError> {
        PodcastEpisode::get_episodes()
    }

    pub fn find_all_downloaded_podcast_episodes_with_top_k(
        top_k: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        PodcastEpisode::get_podcast_episodes_by_podcast_to_k(top_k)
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
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        PodcastEpisode::get_episodes_by_podcast_id(podcast_id)
    }

    fn update_podcast_fields(feed: Channel, podcast_id: i32) -> Result<(), CustomError> {
        let itunes = feed.clone().itunes_ext;

        if let Some(itunes) = itunes {
            let constructed_extra_fields = PodcastBuilder::new(podcast_id)
                .author(itunes.author)
                .last_build_date(feed.last_build_date)
                .description(feed.description)
                .language(feed.language)
                .keywords(itunes.categories)
                .build();

            Podcast::update_podcast_fields(constructed_extra_fields)?;
        }

        Ok(())
    }

    pub fn cleanup_old_episodes(days_from_settings: i32) {
        let podcasts = Podcast::get_all_podcasts();

        if podcasts.is_err() {
            return;
        }

        for p in podcasts.unwrap() {
            let podcast_settings = PodcastSetting::get_settings(p.id);
            if podcast_settings.is_err() {
                continue;
            }
            let days;

            if let Some(podcast_settings) = podcast_settings.unwrap() {
                if podcast_settings.auto_cleanup {
                    days = podcast_settings.auto_cleanup_days;
                } else {
                    days = days_from_settings;
                }
            } else {
                days = days_from_settings;
            }

            let old_podcast_episodes =
                PodcastEpisode::get_podcast_episodes_older_than_days(days, p.id);

            log::info!("Cleaning up {} old episodes", old_podcast_episodes.len());
            for old_podcast_episode in old_podcast_episodes {
                let res = FileService::cleanup_old_episode(&old_podcast_episode);

                match res {
                    Ok(_) => {
                        PodcastEpisode::update_download_status_of_episode(
                            old_podcast_episode.clone().id,
                        );
                    }
                    Err(e) => {
                        println!("Error deleting podcast episode.{}", e);
                    }
                }
            }
        }
    }

    pub fn get_podcast_episodes_of_podcast(
        id_num: i32,
        last_id: Option<String>,
        user: User,
    ) -> Result<Vec<(PodcastEpisode, Option<Episode>)>, CustomError> {
        PodcastEpisode::get_podcast_episodes_of_podcast(id_num, last_id, user)
    }

    pub fn get_podcast_episode_by_id(id_num: &str) -> Result<Option<PodcastEpisode>, CustomError> {
        PodcastEpisode::get_podcast_episode_by_id(id_num)
    }

    fn do_request_to_podcast_server(podcast: Podcast) -> RequestReturnType {
        let is_redirected = Arc::new(Mutex::new(false)); // Variable to store the redirection status
        let client = get_sync_client()
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
    ) -> Result<PodcastEpisode, CustomError> {
        let episode = PodcastEpisode::get_podcast_episode_by_id(episode_id)?;
        if episode.is_none() {
            return Err(CustomError::NotFound);
        }

        match episode {
            Some(episode) => {
                FileService::cleanup_old_episode(&episode)?;
                PodcastEpisode::update_download_status_of_episode(episode.id);
                PodcastEpisode::update_deleted(episode_id, true)?;
                Ok(episode)
            }
            None => Err(CustomError::NotFound),
        }
    }

    pub fn get_track_number_for_episode(
        podcast_id: i32,
        date_of_recording_to_search: &str,
    ) -> Result<i64, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;

        podcast_episodes
            .filter(
                crate::adapters::persistence::dbconfig::schema::podcast_episodes::podcast_id
                    .eq(podcast_id),
            )
            .filter(
                crate::adapters::persistence::dbconfig::schema::podcast_episodes::date_of_recording
                    .le(date_of_recording_to_search),
            )
            .count()
            .get_result::<i64>(&mut get_connection())
            .map_err(map_db_error)
    }
}

struct RequestReturnType {
    pub url: String,
    pub content: String,
}
