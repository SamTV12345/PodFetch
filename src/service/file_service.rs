use std::collections::HashMap;
use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::podcast_episode_service::PodcastEpisodeService;
use reqwest::{Client, ClientBuilder};
use std::io::{Error, Write};
use std::path::Path;
use std::str::FromStr;
use regex::Regex;

use crate::controllers::settings_controller::ReplacementStrategy;
use crate::models::settings::Setting;
use crate::service::path_service::PathService;
use crate::service::settings_service::SettingsService;

#[derive(Clone)]
pub struct FileService {
    pub db: DB,
    pub client: Client,
}

impl FileService {
    pub fn new() -> Self {
        FileService {
            db: DB::new().unwrap(),
            client: ClientBuilder::new().build().unwrap(),
        }
    }
    pub fn check_if_podcast_main_image_downloaded(&mut self, podcast_id: &str) -> bool {
        let podcast = self
            .db
            .clone()
            .get_podcast_by_directory_id(podcast_id)
            .unwrap();
        match podcast {
            Some(podcast) => {
                if !podcast.image_url.contains("http") {
                    return Path::new(&podcast.image_url).exists();
                }
            }
            None => {
                return false;
            }
        }
        return false;
    }

    pub fn create_podcast_root_directory_exists() ->Result<(), Error> {
        if !Path::new("podcasts").exists() {
            return std::fs::create_dir("podcasts")
        }

        Ok(())
    }

    pub fn create_podcast_directory_exists(podcast_title: &str, podcast_id: &String) ->Result<String,
        Error> {
        let escaped_title = prepare_podcast_title_to_directory(podcast_title);
        if !Path::new(&format!("podcasts/{}", escaped_title)).exists() {
            std::fs::create_dir(&format!("podcasts/{}", escaped_title))
                .expect(&*("Error creating directory when inserting ".to_owned() + &escaped_title));
            Ok(format!("podcasts/{}", escaped_title))
        }
        else{
            // Check if this is a new podcast with the same name as an old one

            let db = DB::new().unwrap();
            let podcast = db.get_podcast_by_directory_id(podcast_id).unwrap();
            match podcast {
                Some(_)=>{
                    // is the same podcast
                    Ok(format!("podcasts/{}", escaped_title))
                }
                None=>{
                    // has not been inserted into the database yet
                    let mut i = 1;
                    while Path::new(&format!("podcasts/{}-{}", escaped_title, i)).exists() {
                        i += 1;
                    }
                    // This is save to insert because this directory does not exist
                    std::fs::create_dir(&format!("podcasts/{}-{}", escaped_title, i))
                        .expect("Error creating directory");
                    Ok(format!("podcasts/{}-{}", escaped_title, i))
                }
            }
        }
    }

    pub async fn download_podcast_image(&self, podcast_path: &str, image_url: &str, podcast_id: &str) {
        let image_response = self.client.get(image_url).send().await.unwrap();
        let image_suffix = PodcastEpisodeService::get_url_file_suffix(image_url);
        let file_path = format!("{}/image.{}", podcast_path, image_suffix);
        let mut image_out = std::fs::File::create(file_path.clone()).unwrap();
        let bytes = image_response.bytes().await.unwrap();
        image_out.write_all(&bytes).unwrap();
        let db = DB::new().unwrap();
        db.update_podcast_image(podcast_id, &file_path).unwrap();
    }

    pub fn cleanup_old_episode(podcast: Podcast, episode: PodcastEpisode) -> std::io::Result<()> {
        log::info!("Cleaning up old episode: {}", episode.episode_id);
        std::fs::remove_dir_all(&format!(
            "podcasts/{}/{}",
            podcast.directory_id, episode.episode_id
        ))
    }

    pub fn delete_podcast_files(podcast_dir: &str){
        std::fs::remove_dir_all(format!("podcasts/{}", podcast_dir)).expect("Error deleting podcast directory");
    }
}


pub fn prepare_podcast_title_to_directory(title: &str) ->String {
    let mut settings_service = SettingsService::new();
    let retrieved_settings = settings_service.get_settings().unwrap();
    let final_string = perform_replacement(title, retrieved_settings.clone());

    let fixed_string = retrieved_settings.podcast_format.replace("{}","{podcasttitle}");

    let mut vars:HashMap<String,&str> = HashMap::new();
    vars.insert("podcasttitle".to_string(), &final_string);
    strfmt::strfmt(&fixed_string, &vars).unwrap()
}

pub fn prepare_podcast_episode_title_to_directory(title: &str) ->String {
    let mut settings_service = SettingsService::new();
    let retrieved_settings = settings_service.get_settings().unwrap();
    let final_string = perform_replacement(title, retrieved_settings.clone());

    let fixed_string = retrieved_settings.episode_format.replace("{}","{episodetitle}");

    let mut vars:HashMap<String, &str> = HashMap::new();
    vars.insert("episodetitle".to_string(), &final_string);
    strfmt::strfmt(&fixed_string, &vars).unwrap()
}

fn perform_replacement(title: &str, retrieved_settings:Setting) -> String {
    let mut final_string: String = title.to_string();


    // If checked replace all illegal characters
    if retrieved_settings.replace_invalid_characters {
        let illegal_chars_regex = Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
        final_string = illegal_chars_regex.replace_all(title, "").to_string();
    }

    // Colon replacement strategy
    match ReplacementStrategy::from_str(&retrieved_settings.replacement_strategy).unwrap() {
        ReplacementStrategy::ReplaceWithDashAndUnderscore => {
            final_string = final_string.replace(":", "_").replace(" ", "_")
        }
        ReplacementStrategy::Remove => {
            final_string = final_string.replace(":", "").replace(" ", "")
        }
        ReplacementStrategy::ReplaceWithDash => {
            final_string = final_string.replace(":", "-").replace(" ", "-")
        }
    }

    final_string = deunicode::deunicode(&final_string);
    final_string
}


/*
First image, then podcast
*/
pub fn determine_image_and_local_podcast_audio_url(podcast:Podcast, podcast_episode:
PodcastEpisode, image_suffix: &str, suffix: &str)->(String, String){
    let image_save_path;
    let podcast_save_path;

    if podcast_episode.local_image_url.trim().len()==0{
        image_save_path= PathService::get_image_path(
            &podcast.clone().directory_name,
            &podcast_episode.clone().name,
            &image_suffix,
        );
    }
    else{
        image_save_path = podcast_episode.clone().local_image_url
    }

    if podcast_episode.local_url.trim().len()==0{
        podcast_save_path = PathService::get_podcast_episode_path(
            &podcast.directory_name.clone(),
            &podcast_episode.name,
            &suffix);
    }
    else{
        podcast_save_path = podcast_episode.clone().local_url;
    }
    return (image_save_path, podcast_save_path)
}