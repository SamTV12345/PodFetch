use crate::constants::constants::{PodcastType, TELEGRAM_API_ENABLED};
use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::models::messages::BroadcastMessage;
use crate::models::models::Notification;
use crate::models::web_socket_message::Lobby;
use crate::service::download_service::DownloadService;
use crate::service::file_service::{determine_image_and_local_podcast_audio_url, FileService};
use crate::service::mapping_service::MappingService;

use crate::utils::podcast_builder::PodcastBuilder;
use actix::Addr;
use actix_web::web;
use diesel::SqliteConnection;
use dotenv::var;
use regex::Regex;
use reqwest::blocking::ClientBuilder;
use rss::Channel;
use crate::service::settings_service::SettingsService;
use crate::service::telegram_api::send_new_episode_notification;

#[derive(Clone)]
pub struct PodcastEpisodeService {
    db: DB,
    mapping_service: MappingService,
}

impl PodcastEpisodeService {
    pub fn new() -> Self {
        PodcastEpisodeService {
            db: DB::new().unwrap(),
            mapping_service: MappingService::new(),
        }
    }
    pub fn download_podcast_episode_if_not_locally_available(
        &mut self,
        podcast_episode: PodcastEpisode,
        podcast: Podcast,
        lobby: Option<web::Data<Addr<Lobby>>>,
    ) {
        let mut db = DB::new().unwrap();
        let podcast_episode_cloned = podcast_episode.clone();
        let podcast_cloned = podcast.clone();
        let suffix = Self::get_url_file_suffix(&podcast_episode_cloned.url);
        let image_suffix = Self::get_url_file_suffix(&podcast_episode_cloned.image_url);


        let (image_save_path, podcast_save_path) = determine_image_and_local_podcast_audio_url
            (podcast.clone(), podcast_episode.clone(), &suffix, &image_suffix);


        match db.check_if_downloaded(&podcast_episode.url) {
            Ok(true) => {
                self.db
                    .update_total_podcast_time_and_image(
                        &podcast_episode_cloned.episode_id,
                        &image_save_path,
                        &podcast_save_path.clone(),
                    )
                    .expect("Error saving total time of podcast episode.");
            }
            Ok(false) => {
                let podcast_inserted = Self::perform_download(
                    &podcast_episode,
                    &mut db,
                    podcast_episode_cloned,
                    podcast_cloned,
                );
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
    }

    pub fn perform_download(
        podcast_episode: &PodcastEpisode,
        db: &mut DB,
        podcast_episode_cloned: PodcastEpisode,
        podcast_cloned: Podcast,
    ) -> PodcastEpisode {
        log::info!("Downloading podcast episode: {}", podcast_episode.name);
        let mut download_service = DownloadService::new();
        download_service.download_podcast_episode(podcast_episode_cloned, podcast_cloned);
        let podcast = db
            .update_podcast_episode_status(&podcast_episode.url, "D")
            .unwrap();
        let notification = Notification {
            id: 0,
            message: format!("Episode {} is now available offline", podcast_episode.name),
            created_at: chrono::Utc::now().naive_utc().to_string(),
            type_of_message: "Download".to_string(),
            status: "unread".to_string(),
        };
        db.insert_notification(notification).unwrap();
        return podcast;
    }

    pub fn get_last_n_podcast_episodes(conn: &mut SqliteConnection, podcast: Podcast) ->
                                                                             Vec<PodcastEpisode> {

        let mut settings_service = SettingsService::new();
        let settings = settings_service.get_settings().unwrap();
        DB::get_last_n_podcast_episodes(conn, podcast.id,
                                        settings.podcast_prefill).unwrap()
    }

    // Used for creating/updating podcasts
    pub fn insert_podcast_episodes(&mut self, conn: &mut SqliteConnection, podcast: Podcast) ->
                                                                             Vec<PodcastEpisode> {
        let client = ClientBuilder::new().build().unwrap();
        let result = client.get(podcast.clone().rssfeed).send().unwrap();
        let bytes = result.bytes().unwrap();
        let channel = Channel::read_from(&*bytes).unwrap();
        self.update_podcast_fields(channel.clone(), podcast.id.clone());

        let mut podcast_inserted = Vec::new();

        // insert original podcast image url
        if podcast.original_image_url.is_empty() {
            let mut db = DB::new().unwrap();
            db.update_original_image_url(&channel.image().unwrap().url.to_string(), podcast.id);
        }

        for (_, item) in channel.items.iter().enumerate() {
            let itunes_ext = item.clone().itunes_ext;

            match itunes_ext {
                Some(itunes_ext) => {
                    let enclosure = item.enclosure();
                    match enclosure {
                        Some(enclosure)=>{
                            let result =
                                DB::get_podcast_episode_by_url(conn, &enclosure.url.to_string(),
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

                                let inserted_episode = DB::insert_podcast_episodes(conn,
                                                                                   podcast.clone(),
                                                                                   item.clone(),
                                                                                   Some(result_unwrapped.unwrap().image_url),
                                                                                   duration_episode as i32,
                                );
                                podcast_inserted.push(inserted_episode);
                            }

                            if result.clone().unwrap().is_none() {
                                // Insert new podcast episode
                                match itunes_ext.clone().duration {
                                    Some(duration) => {
                                        duration_episode = Self::parse_duration(&duration);
                                    }
                                    None => {}
                                }

                                let inserted_episode = DB::insert_podcast_episodes(conn,
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
                    let result = DB::get_podcast_episode_by_url(
                        conn, &opt_enclosure.clone().unwrap().url, None);
                    // We can't retrieve the duration of the podcast episode, so we set it to 0

                    if result.unwrap().is_none() {
                        let duration_episode = 0;
                        let inserted_episode = DB::insert_podcast_episodes(
                            conn,
                            podcast.clone(),
                            item.clone(),
                            Some("ui/default.jpg".parse().unwrap()),
                            duration_episode as i32,
                        );
                        podcast_inserted.push(inserted_episode);
                    }
                }
            }
        }
        return podcast_inserted;
    }

    fn parse_duration(duration_str: &str) -> u32 {
        let parts: Vec<&str> = duration_str.split(":").collect();
        match parts.len() {
            1 => {
                let seconds = parts[0].parse::<u32>().unwrap();
                seconds
            }
            2 => {
                let minutes = parts[0].parse::<u32>().unwrap();
                let seconds = parts[1].parse::<u32>().unwrap();
                minutes * 60 + seconds
            }
            3 => {
                let hours = parts[0].parse::<u32>().unwrap();
                let minutes = parts[1].parse::<u32>().unwrap();
                let seconds = parts[2].parse::<u32>().unwrap();
                hours * 3600 + minutes * 60 + seconds
            }
            4 => {
                let days = parts[0].parse::<u32>().unwrap();
                let hours = parts[1].parse::<u32>().unwrap();
                let minutes = parts[2].parse::<u32>().unwrap();
                let seconds = parts[3].parse::<u32>().unwrap();
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

    pub fn query_for_podcast(&mut self, query: &str) -> Vec<PodcastEpisode> {
        let podcasts = self.db.query_for_podcast(query).unwrap();
        let podcast_dto = podcasts
            .iter()
            .map(|podcast| self.mapping_service.map_podcastepisode_to_dto(podcast))
            .collect::<Vec<PodcastEpisode>>();
        return podcast_dto;
    }

    pub fn find_all_downloaded_podcast_episodes(&mut self) -> Vec<PodcastEpisode> {
        let result = self.db.get_downloaded_episodes();
        result
            .iter()
            .map(|podcast| {
                return self.mapping_service.map_podcastepisode_to_dto(podcast);
            })
            .collect::<Vec<PodcastEpisode>>()
    }

    pub fn find_all_downloaded_podcast_episodes_by_podcast_id(
        &mut self,
        podcast_id: i32,
    ) -> Vec<PodcastEpisode> {
        let result = self.db.get_downloaded_episodes_by_podcast_id(podcast_id);
        result
            .iter()
            .map(|podcast| {
                return self.mapping_service.map_podcastepisode_to_dto(podcast);
            })
            .collect::<Vec<PodcastEpisode>>()
    }

    fn update_podcast_fields(&mut self, feed: Channel, podcast_id: i32) {
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

                self.db.update_podcast_fields(constructed_extra_fields);
            }
            None => {}
        }
    }

    pub fn cleanup_old_episodes(&mut self, days: i32, conn: &mut SqliteConnection) {
        let old_podcast_episodes = self.db.get_podcast_episodes_older_than_days(days);

        log::info!("Cleaning up {} old episodes", old_podcast_episodes.len());
        for old_podcast in old_podcast_episodes {
            let podcast = DB::get_podcast(conn,old_podcast.clone().podcast_id).unwrap();
            let res = FileService::cleanup_old_episode(podcast, old_podcast.clone());

            match res {
                Ok(_) => {
                    self.db
                        .update_download_status_of_episode(old_podcast.clone().id);
                }
                Err(e) => {
                    println!("Error deleting podcast episode.{}", e);
                }
            }
        }
    }

    pub fn get_podcast_episodes_of_podcast(conn: &mut SqliteConnection, id_num: i32, last_id:
    Option<String>)
        -> Result<Vec<PodcastEpisode>, String> {
        DB::get_podcast_episodes_of_podcast(conn,id_num, last_id)
    }

    pub fn get_podcast_episode_by_id(conn: &mut SqliteConnection, id_num: &str) ->
                                                                   Result<Option<PodcastEpisode>, String> {
        DB::get_podcast_episode_by_id(conn, id_num)
    }
}
