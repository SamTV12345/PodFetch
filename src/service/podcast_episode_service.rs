use std::sync::{Arc, Mutex};
use crate::constants::constants::{PodcastType, TELEGRAM_API_ENABLED};
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::messages::BroadcastMessage;
use crate::models::web_socket_message::Lobby;
use crate::service::download_service::DownloadService;
use crate::service::file_service::{determine_image_and_local_podcast_audio_url, FileService};
use crate::service::mapping_service::MappingService;

use crate::utils::podcast_builder::PodcastBuilder;
use actix::Addr;
use actix_web::web;
use diesel::{OptionalExtension, RunQueryDsl};
use dotenv::var;
use regex::Regex;
use reqwest::blocking::ClientBuilder;
use reqwest::header::{ACCEPT, HeaderMap};
use reqwest::redirect::Policy;
use rss::{Channel, Item};

use crate::DbConnection;
use crate::models::notification::Notification;

use crate::mutex::LockResultExt;
use crate::service::environment_service::EnvironmentService;
use crate::service::settings_service::SettingsService;
use crate::service::telegram_api::send_new_episode_notification;
use crate::utils::error::CustomError;

#[derive(Clone)]
pub struct PodcastEpisodeService {
    mapping_service: MappingService,
}


impl PodcastEpisodeService {
    pub fn new() -> Self {
        PodcastEpisodeService {
            mapping_service: MappingService::new(),
        }
    }

    pub fn download_podcast_episode_if_not_locally_available(
        &mut self,
        podcast_episode: PodcastEpisode,
        podcast: Podcast,
        lobby: Option<web::Data<Addr<Lobby>>>,
        conn: &mut DbConnection,
    ) ->Result<(), CustomError> {
        let mut settings_service = SettingsService::new();
        let settings = settings_service.get_settings(conn)?.unwrap();
        let podcast_episode_cloned = podcast_episode.clone();
        let podcast_cloned = podcast.clone();
        let suffix = Self::get_url_file_suffix(&podcast_episode_cloned.url);
        let image_suffix = Self::get_url_file_suffix(&podcast_episode_cloned.image_url);


        let (image_save_path, podcast_save_path) = determine_image_and_local_podcast_audio_url
            (podcast.clone(), podcast_episode.clone(), &suffix, &image_suffix, settings,conn)?;


        match PodcastEpisode::check_if_downloaded(&podcast_episode.url, conn) {
            Ok(true) => {

                PodcastEpisode::update_total_podcast_time_and_image(
                        &podcast_episode_cloned.episode_id,
                        &image_save_path,
                        &podcast_save_path.clone(),
                        conn
                    )
                    .expect("Error saving total time of podcast episode.");
            }
            Ok(false) => {
                let podcast_inserted = Self::perform_download(
                    &podcast_episode,
                    podcast_episode_cloned,
                    podcast_cloned,
                    conn
                )?;
                let mapped_dto = self
                    .mapping_service
                    .map_podcastepisode_to_dto(&podcast_inserted);
                match lobby {
                    Some(lobby) => lobby.do_send(BroadcastMessage {
                        message: format!(
                            "Episode {} is now available offline",
                            podcast_episode.name
                        ),
                        podcast: Option::from(podcast.clone()),
                        type_of: PodcastType::AddPodcastEpisode,
                        podcast_episode: Some(mapped_dto),
                        podcast_episodes: None,
                    }),
                    None => {}
                }
                if var(TELEGRAM_API_ENABLED).is_ok(){
                    send_new_episode_notification(podcast_episode, podcast)
                }
            }

            _ => {
                println!("Error checking if podcast episode is downloaded.");
            }
        }
        Ok(())
    }

    pub fn perform_download(
        podcast_episode: &PodcastEpisode,
        podcast_episode_cloned: PodcastEpisode,
        podcast_cloned: Podcast,
        conn: &mut DbConnection,
    ) -> Result<PodcastEpisode,CustomError> {
        log::info!("Downloading podcast episode: {}", podcast_episode.name);
        let mut download_service = DownloadService::new();
        download_service.download_podcast_episode(podcast_episode_cloned, podcast_cloned)?;
        let podcast = PodcastEpisode::update_podcast_episode_status(&podcast_episode.url, "D", conn)
            .unwrap();
        let notification = Notification {
            id: 0,
            message: format!("Episode {} is now available offline", podcast_episode.name),
            created_at: chrono::Utc::now().naive_utc().to_string(),
            type_of_message: "Download".to_string(),
            status: "unread".to_string(),
        };
        Notification::insert_notification(notification,conn).unwrap();
        return Ok(podcast);
    }

    pub fn get_last_n_podcast_episodes(conn: &mut DbConnection, podcast: Podcast) ->
                                                                             Result<Vec<PodcastEpisode>, CustomError> {

        let mut settings_service = SettingsService::new();
        let settings = settings_service.get_settings(conn)?.unwrap();
        Ok(PodcastEpisode::get_last_n_podcast_episodes(conn, podcast.id,
                                        settings.podcast_prefill).unwrap())
    }

    // Used for creating/updating podcasts
    pub fn insert_podcast_episodes(&mut self, conn: &mut DbConnection, podcast: Podcast) ->
                                                                             Result<Vec<PodcastEpisode>, CustomError> {
        let is_redirected = Arc::new(Mutex::new(false)); // Variable to store the redirection status

        let returned_data_from_podcast_insert = Self::do_request_to_podcast_server(podcast.clone());

        let channel = Channel::read_from(returned_data_from_podcast_insert.content.as_bytes())
            .unwrap();

        if *is_redirected.clone().lock().ignore_poison() {
            log::info!("The podcast {} has moved to {}", podcast.name,
                returned_data_from_podcast_insert.url);
            Podcast::update_podcast_urls_on_redirect(podcast.id, returned_data_from_podcast_insert.url, conn);
            Self::update_episodes_on_redirect(conn,channel.items());
        }

        if channel.itunes_ext.is_some(){
            let extension = channel.itunes_ext.clone().unwrap();

            if extension.new_feed_url.is_some(){
                let new_url = extension.new_feed_url.unwrap();
                Podcast::update_podcast_urls_on_redirect(podcast.id, new_url, conn);

                let returned_data_from_server = Self::do_request_to_podcast_server(podcast.clone());

                let channel = Channel::read_from(returned_data_from_server.content.as_bytes())
                    .unwrap();
                let items = channel.items();
                Self::update_episodes_on_redirect(conn, items);
            }
        }


        self.update_podcast_fields(channel.clone(), podcast.id.clone(),conn)?;

        let mut podcast_inserted = Vec::new();


            match channel.image() {
                Some(image) => {
                   Podcast::update_original_image_url(&image.url.to_string(), podcast.id,
                    conn)?;
                }
                None => {
                    let env = EnvironmentService::new();
                    let url = env.server_url.clone().to_owned() + &"ui/default.jpg".to_string();
                    Podcast::update_original_image_url(&url, podcast.id,conn)?;
                }
            }

        for (_, item) in channel.items.iter().enumerate() {
            let itunes_ext = item.clone().itunes_ext;

            match itunes_ext {
                Some(itunes_ext) => {
                    let enclosure = item.enclosure();
                    match enclosure {
                        Some(enclosure)=>{
                            let result =
                                PodcastEpisode::get_podcast_episode_by_url(conn, &enclosure.url
                                    .to_string(),
                                                               Some(podcast.id));
                            let mut duration_episode = 0;

                            if result.is_err(){
                                log::info!("Skipping episode {} with error: {}", item.clone().title
                                    .unwrap_or("with no title".to_string()), result.err().unwrap());
                                continue;
                            }

                            let result_unwrapped = result.clone().unwrap();

                            if result_unwrapped.is_some()  && result_unwrapped.clone().unwrap()
                                .podcast_id != podcast.id {

                                let inserted_episode = PodcastEpisode::insert_podcast_episodes(conn,
                                                                                   podcast.clone(),
                                                                                   item.clone(),
                                                                                   Some(result_unwrapped.unwrap().image_url),
                                                                                   duration_episode as i32,
                                );
                                podcast_inserted.push(inserted_episode);
                            }
                            else if result_unwrapped.is_some(){
                                let already_loaded_episode = result_unwrapped.unwrap();
                                if already_loaded_episode.guid.is_empty() && item.guid().is_some
                                () && !item.guid().unwrap().permalink{
                                    PodcastEpisode::update_guid(conn,item.guid.clone().unwrap(),
                                                    &already_loaded_episode.episode_id);
                                }
                            }

                            if result.clone().unwrap().is_none() {
                                // Insert new podcast episode
                                match itunes_ext.clone().duration {
                                    Some(duration) => {
                                        duration_episode = Self::parse_duration(&duration);
                                    }
                                    None => {}
                                }

                                let inserted_episode = PodcastEpisode::insert_podcast_episodes(conn,
                                                                                   podcast.clone(),
                                                                                   item.clone(),
                                                                                   itunes_ext.image,
                                                                                   duration_episode as i32,
                                );
                                podcast_inserted.push(inserted_episode);
                            }
                        }
                        None => {
                            log::info!("Skipping episode {} without enclosure.", item.clone().title
                                .unwrap_or("with no title".to_string()));
                            continue;
                        }
                    }

                }
                None => {
                    let opt_enclosure = &item.enclosure;
                    if opt_enclosure.is_none() {
                        log::info!("Skipping episode {} without enclosure.", item.clone().title.unwrap_or("with no title".to_string()));
                        continue;
                    }
                    let result = PodcastEpisode::get_podcast_episode_by_url(
                        conn, &opt_enclosure.clone().unwrap().url, None);
                    // We can't retrieve the duration of the podcast episode, so we set it to 0

                    if result.unwrap().is_none() {
                        let duration_episode = 0;
                        let inserted_episode = PodcastEpisode::insert_podcast_episodes(
                            conn,
                            podcast.clone(),
                            item.clone(),
                            Some("ui/default.jpg".parse().unwrap()),
                            duration_episode,
                        );
                        podcast_inserted.push(inserted_episode);
                    }
                }
            }
        }
        return Ok(podcast_inserted);
    }


    fn update_episodes_on_redirect(conn: &mut DbConnection, items: &[Item]) {
        for (_, item) in items.iter().enumerate(){
            let opt_found_podcast_episode = Self::get_podcast_episode_by_guid(conn, item.title.as_ref().unwrap());
            if opt_found_podcast_episode.is_some(){
                let found_podcast_episode = opt_found_podcast_episode.unwrap();
                let mut podcast_episode = found_podcast_episode.clone();
                if item.itunes_ext.is_some(){
                    podcast_episode.image_url = item.itunes_ext.as_ref().unwrap().image.clone()
                        .unwrap();
                }
                podcast_episode.url = item.enclosure.as_ref().unwrap().url.to_string();
                PodcastEpisode::update_podcast_episode(conn, podcast_episode);
            }
        }
    }

    fn get_podcast_episode_by_guid(conn: &mut DbConnection, guid_to_search: &str) ->
                                                                    Option<PodcastEpisode>{
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(guid.eq(guid_to_search))
            .first::<PodcastEpisode>(conn)
            .optional().expect("Error loading podcast episode")
    }

    fn parse_duration(duration_str: &str) -> u32 {
        let parts: Vec<&str> = duration_str.split(":").collect();
        match parts.len() {
            1 => {
                let seconds = parts[0].parse::<u32>().unwrap_or(0);
                seconds
            }
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

    pub fn get_url_file_suffix(url: &str) -> String {
        let re = Regex::new(r#"\.(\w+)(?:\?.*)?$"#).unwrap();
        let capture = re.captures(&url).unwrap();
        return capture.get(1).unwrap().as_str().to_owned();
    }

    pub fn query_for_podcast(&mut self, query: &str, conn:&mut DbConnection) -> Result<Vec<PodcastEpisode>, CustomError> {

        let podcasts = Podcast::query_for_podcast(query,conn)?;
        let podcast_dto = podcasts
            .iter()
            .map(|podcast| self.mapping_service.map_podcastepisode_to_dto(podcast))
            .collect::<Vec<PodcastEpisode>>();
        return Ok(podcast_dto);
    }

    pub fn find_all_downloaded_podcast_episodes(&mut self, conn:&mut DbConnection, env: EnvironmentService) ->
                                                                                   Result<Vec<PodcastEpisode>, CustomError> {
        let result = PodcastEpisode::get_episodes(conn);
        Ok(self.map_rss_podcast_episodes(env, result)?)
    }

    fn map_rss_podcast_episodes(&mut self, env: EnvironmentService, result: Vec<PodcastEpisode>)
        -> Result<Vec<PodcastEpisode>, CustomError> {
        Ok(result
            .iter()
            .map(|podcast| {
                let mut podcast_episode_dto = self.mapping_service.map_podcastepisode_to_dto(podcast);
                return if podcast_episode_dto.is_downloaded() {
                    let local_url = self.map_to_local_url(&podcast_episode_dto.clone().local_url);
                    let local_image_url = self.map_to_local_url(&podcast_episode_dto.clone()
                        .local_image_url);

                    podcast_episode_dto.local_image_url = env.server_url.clone() + &local_image_url;
                    podcast_episode_dto.local_url = env.server_url.clone() + &local_url;

                    return podcast_episode_dto
                } else {
                    podcast_episode_dto.local_image_url = podcast_episode_dto.clone().image_url;
                    podcast_episode_dto.local_url = podcast_episode_dto.clone().url;

                    podcast_episode_dto
                }
            })
            .collect::<Vec<PodcastEpisode>>())
    }


    fn map_to_local_url(&mut self, url: &str) -> String {
        let splitted_url = url.split("/").collect::<Vec<&str>>();
        splitted_url.iter()
            .map(|s| return if s.starts_with("podcasts.")|| s.starts_with("image.") {
                s.to_string()
            } else {
                urlencoding::encode(s).clone().to_string()
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    pub fn find_all_downloaded_podcast_episodes_by_podcast_id(
        &mut self,
        podcast_id: i32,
        conn:&mut DbConnection
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        let env = EnvironmentService::new();
        let result = PodcastEpisode::get_episodes_by_podcast_id(podcast_id, conn);
        Ok(self.map_rss_podcast_episodes(env, result)?)
    }

    fn update_podcast_fields(&mut self, feed: Channel, podcast_id: i32, conn:&mut DbConnection)
        ->Result<(), CustomError> {
        let itunes = feed.clone().itunes_ext;

        match itunes {
            Some(itunes) => {
                let constructed_extra_fields = PodcastBuilder::new(podcast_id)
                    .author(itunes.author)
                    .last_build_date(feed.last_build_date)
                    .description(feed.description)
                    .language(feed.language)
                    .keywords(itunes.categories)
                    .build();

                Podcast::update_podcast_fields(constructed_extra_fields,conn)?;
            }
            None => {
            }
        }
        Ok(())

    }

    pub fn cleanup_old_episodes(&mut self, days: i32, conn: &mut DbConnection) {

        let old_podcast_episodes = PodcastEpisode::get_podcast_episodes_older_than_days(days,conn);

        log::info!("Cleaning up {} old episodes", old_podcast_episodes.len());
        for old_podcast in old_podcast_episodes {
            let podcast = Podcast::get_podcast(conn,old_podcast.clone().podcast_id).unwrap();
            let res = FileService::cleanup_old_episode(podcast, old_podcast.clone());

            match res {
                Ok(_) => {
                    PodcastEpisode::update_download_status_of_episode(old_podcast.clone().id,conn);
                }
                Err(e) => {
                    println!("Error deleting podcast episode.{}", e);
                }
            }
        }
    }

    pub fn get_podcast_episodes_of_podcast(conn: &mut DbConnection, id_num: i32, last_id:
    Option<String>)
        -> Result<Vec<PodcastEpisode>, CustomError> {
        Ok(PodcastEpisode::get_podcast_episodes_of_podcast(conn,id_num, last_id)?)
    }

    pub fn get_podcast_episode_by_id(conn: &mut DbConnection, id_num: &str) ->
                                                                   Result<Option<PodcastEpisode>,
                                                                       CustomError> {
        Ok(PodcastEpisode::get_podcast_episode_by_id(conn, id_num)?)
    }

    fn do_request_to_podcast_server(podcast:Podcast) ->RequestReturnType{
        let is_redirected = Arc::new(Mutex::new(false)); // Variable to store the redirection status
        let client = ClientBuilder::new().redirect(Policy::custom({
            let is_redirected = Arc::clone(&is_redirected);

            move |attempt|{

                if attempt.previous().len() > 0 {
                    *is_redirected.lock().unwrap() = true;
                }
                attempt.follow()
            }
        })).build().unwrap();
        let mut header_map = HeaderMap::new();
        header_map.append(ACCEPT, "application/rss+xml,application/xml".parse().unwrap());
        header_map.append("User-Agent", "PostmanRuntime/7.32.2".parse().unwrap());
        let result = client
            .get(podcast.clone().rssfeed)
            .headers(header_map)
            .send()
            .unwrap();
        let url = result.url().clone().to_string();
        let content = result.text().unwrap().clone();

        return RequestReturnType {
            url,
            content
        }
    }
}

struct RequestReturnType {
    pub url:String,
    pub content:String
}
