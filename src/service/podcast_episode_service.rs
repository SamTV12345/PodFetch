use actix::Addr;
use actix_web::web;
use feed_rs::parser;
use regex::Regex;
use reqwest::blocking::ClientBuilder;
use crate::constants::constants::{PodcastType};
use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::models::messages::BroadcastMessage;
use crate::models::models::Notification;
use crate::models::web_socket_message::Lobby;
use crate::service::download_service::DownloadService;
use crate::service::mapping_service::MappingService;
use crate::service::path_service::PathService;

#[derive(Clone)]
pub struct PodcastEpisodeService {
    db: DB,
    mapping_service: MappingService
}

impl PodcastEpisodeService{
    pub fn new() -> Self {
        PodcastEpisodeService {
            db: DB::new().unwrap(),
            mapping_service: MappingService::new()
        }
    }
    pub fn download_podcast_episode_if_not_locally_available(&mut self, podcast_episode: PodcastEpisode,
                                                             podcast: Podcast, lobby:
                                                             Option<web::Data<Addr<Lobby>>>){
        let mut db = DB::new().unwrap();
        let podcast_episode_cloned = podcast_episode.clone();
        let podcast_cloned = podcast.clone();
        let suffix = Self::get_url_file_suffix(&podcast_episode_cloned.url);
        let image_suffix = Self::get_url_file_suffix(&podcast_episode_cloned.image_url);

        let image_save_path = PathService::get_image_path(&podcast_cloned.clone().directory,
        &podcast_episode_cloned.clone()
        .episode_id, &image_suffix);


        let podcast_save_path = PathService::get_podcast_episode_path(&podcast.directory.clone(),
        &podcast_episode_cloned.episode_id,
        &suffix);


        match  db.check_if_downloaded(&podcast_episode.url){
        Ok(true) => {
                self.db.update_total_podcast_time_and_image(&podcast_episode_cloned.episode_id,
                                                            &image_save_path,
                                                            &podcast_save_path.clone())
                    .expect("Error saving total time of podcast episode.");
        }
        Ok(false) => {
            let podcast_inserted = Self::perform_download(&podcast_episode, &mut db,
                                                  podcast_episode_cloned,
                                    podcast_cloned);
            let mapped_dto = self.mapping_service.map_podcastepisode_to_dto(&podcast_inserted);
            match lobby {
                Some(lobby) => {
                    lobby.do_send(BroadcastMessage{
                        message: format!("Episode {} is now available offline", podcast_episode.name),
                        podcast: Option::from(podcast.clone()),
                        type_of: PodcastType::AddPodcastEpisode,
                        podcast_episode: Some(mapped_dto),
                        podcast_episodes: None,
                    })
                }
                None => {}
            }

        }

        _ => {
                println!("Error checking if podcast episode is downloaded.");
            }
        }
    }

    pub fn perform_download(podcast_episode: &PodcastEpisode, db: &mut DB,
    podcast_episode_cloned: PodcastEpisode, podcast_cloned: Podcast)->PodcastEpisode {
        log::info!("Downloading podcast episode: {}",podcast_episode.name);
        let mut download_service = DownloadService::new();
        download_service.download_podcast_episode(podcast_episode_cloned,
                                                  podcast_cloned);
        let podcast = db.update_podcast_episode_status(&podcast_episode.url, "D").unwrap();
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

    pub fn get_last_5_podcast_episodes(&mut self, podcast: Podcast) -> Vec<PodcastEpisode>{
        self.db.get_last_5_podcast_episodes(podcast.id).unwrap()
    }

    // Used for creating/updating podcasts
    pub fn insert_podcast_episodes(podcast: Podcast) ->Vec<PodcastEpisode>{
        let client = ClientBuilder::new().build().unwrap();
        let result = client.get(podcast.clone().rssfeed).send().unwrap();
        let bytes = result.bytes().unwrap();
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        let vec = Self::get_media_urls(&text);
        let durations = Self::get_time_in_millis(&text);
        let feed = parser::parse(&*bytes).unwrap();

        let mut podcast_inserted:Vec<PodcastEpisode> = Vec::new();
        for (i,item) in feed.entries.iter().enumerate(){
            let mut db = DB::new().unwrap();
            let result = db.get_podcast_episode_by_url(&vec[i].to_owned());

            if result.unwrap().is_none() {
                // Insert new podcast episode
                let duration_string = durations[i].to_owned();
                let duration = Self::parse_duration(&duration_string);
                let mut duration_episode = 0;

                if duration.is_some()
                {
                    duration_episode = duration.unwrap();
                }
                let inserted_episode = db.insert_podcast_episodes(podcast.clone(), &vec[i]
                    .to_owned(),
                                           item, &feed.logo
                        .clone().unwrap().uri, &item.summary.clone().unwrap().content,
                                           duration_episode as i32);
                podcast_inserted.push(inserted_episode);
            }
        }
        return podcast_inserted;
    }


    fn get_media_urls(text: &str)-> Vec<String> {
        let mut urls = Vec::new();
        let re = Regex::new(r#"<enclosure.*?url="(.*?)".*?/>"#).unwrap();
        for capture in re.captures_iter(text){
            let url = capture.get(1).unwrap().as_str();
            urls.push(url.to_owned())
        }
        return urls;
    }

    fn get_time_in_millis(text: &str)-> Vec<String> {
        let mut urls = Vec::new();
        let re = Regex::new(r#"<itunes:duration>(.*?)</itunes:duration>"#).unwrap();
        for capture in re.captures_iter(text){
            let durations = capture.get(1).unwrap().as_str();
            urls.push(durations.to_owned())
        }
        return urls;
    }

    fn parse_duration(duration_str: &str) -> Option<u32> {
        let parts: Vec<&str> = duration_str.split(":").collect();
        match parts.len() {
            1=> {
                let seconds = parts[0].parse::<u32>().ok()?;
                Some(seconds)
            }
            2 => {
                let minutes = parts[0].parse::<u32>().ok()?;
                let seconds = parts[1].parse::<u32>().ok()?;
                Some(minutes * 60 + seconds)
            }
            3 => {
                let hours = parts[0].parse::<u32>().ok()?;
                let minutes = parts[1].parse::<u32>().ok()?;
                let seconds = parts[2].parse::<u32>().ok()?;
                Some(hours * 3600 + minutes * 60 + seconds)
            }
            4=>{
                let days = parts[0].parse::<u32>().ok()?;
                let hours = parts[1].parse::<u32>().ok()?;
                let minutes = parts[2].parse::<u32>().ok()?;
                let seconds = parts[3].parse::<u32>().ok()?;
                Some(days * 86400 + hours * 3600 + minutes * 60 + seconds)
            }
            _ => None
        }
    }

    pub fn get_url_file_suffix(url: &str) -> String {
        let re = Regex::new(r#"\.(\w+)(?:\?.*)?$"#).unwrap();
        let capture = re.captures(&url).unwrap();
        return capture.get(1).unwrap().as_str().to_owned();
    }

    pub fn query_for_podcast(&mut self, query:&str) ->Vec<PodcastEpisode>{
        let podcasts = self.db.query_for_podcast(query).unwrap();
        let podcast_dto = podcasts.iter().map(|podcast| {
            self.mapping_service.map_podcastepisode_to_dto(podcast)
        }).collect::<Vec<PodcastEpisode>>();
        return podcast_dto
    }

    pub fn find_all_downloaded_podcast_episodes(&mut self) ->Vec<PodcastEpisode>{
        let result = self.db.get_downloaded_episodes();
        result.iter().map(|podcast| {
            return self.mapping_service.map_podcastepisode_to_dto(podcast)})
            .collect::<Vec<PodcastEpisode>>()
    }
}
