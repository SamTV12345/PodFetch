use std::collections::HashMap;

use crate::models::podcasts::Podcast;
use reqwest::{Client, ClientBuilder};
use std::io::{Error, Write};

use std::path::Path;
use std::str::FromStr;

use crate::models::podcast_episode::PodcastEpisode;
use regex::Regex;
use tokio::task::spawn_blocking;
use crate::config::dbconfig::establish_connection;
use crate::constants::inner_constants::MAX_FILE_TREE_DEPTH;

use crate::controllers::settings_controller::ReplacementStrategy;
use crate::DbConnection;
use crate::models::settings::Setting;
use crate::service::path_service::PathService;
use crate::service::settings_service::SettingsService;
use crate::utils::error::{CustomError, map_io_error};
use crate::utils::file_extension_determination::{determine_file_extension, FileType};

#[derive(Clone)]
pub struct FileService {
    pub client: Client,
}

impl FileService {
    pub fn new() -> Self {
        FileService {
            client: ClientBuilder::new().build().unwrap(),
        }
    }
    pub fn new_db() -> Self {
        FileService {
            client: ClientBuilder::new().build().unwrap(),
        }
    }
    pub fn check_if_podcast_main_image_downloaded(&mut self, podcast_id: &str, conn: &mut
    DbConnection) -> bool {
        let podcast = Podcast::get_podcast_by_directory_id(podcast_id, conn)
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
        false
    }

    pub fn create_podcast_root_directory_exists() ->Result<(), Error> {
        if !Path::new("podcasts").exists() {
            return std::fs::create_dir("podcasts")
        }

        Ok(())
    }

    pub fn create_podcast_directory_exists(podcast_title: &str, podcast_id: &str, conn:&mut
    DbConnection)
        ->Result<String, CustomError> {
        let escaped_title = prepare_podcast_title_to_directory(podcast_title,conn)?;
        let escaped_path = format!("podcasts/{}", escaped_title);
        if !Path::new(&escaped_path).exists() {
            std::fs::create_dir(escaped_path.clone()).map_err(|err|map_io_error(err,
                                                                                Some(escaped_path
                                                                                    .clone())
            ))?;
            Ok(escaped_path)
        }
        else{
            // Check if this is a new podcast with the same name as an old one

            let conn = &mut establish_connection();
            let podcast = Podcast::get_podcast_by_directory_id(podcast_id,conn)?;
            match podcast {
                Some(_)=>{
                    // is the same podcast
                    Ok(escaped_path)
                }
                None=>{
                    // has not been inserted into the database yet
                    let mut i = 1;
                    while Path::new(&format!("podcasts/{}-{}", escaped_title, i)).exists() {
                        i += 1;
                    }
                    // This is save to insert because this directory does not exist
                    std::fs::create_dir(format!("podcasts/{}-{}", escaped_title, i))
                        .map_err(|err|map_io_error(err,Some(format!("podcasts/{}-{}", escaped_title, i))))?;
                    Ok(format!("podcasts/{}-{}", escaped_title, i))
                }
            }
        }
    }

    pub async fn download_podcast_image(&self, podcast_path: &str, image_url: String,
                                        podcast_id: &str, conn: &mut DbConnection) {
        let cloned_image_url = image_url.clone();
        let image_suffix = spawn_blocking(move ||{
            let client = reqwest::blocking::Client::new();
            determine_file_extension(&cloned_image_url.clone(), &client, FileType::Image)
        }).await.unwrap();

        let image_response = self.client.get(image_url).send().await.unwrap();
        let file_path = PathService::get_image_podcast_path_with_podcast_prefix(podcast_path, &image_suffix);
        let mut image_out = std::fs::File::create(file_path.0.clone()).unwrap();
        let bytes = image_response.bytes().await.unwrap();
        image_out.write_all(&bytes).unwrap();
        PodcastEpisode::update_podcast_image(podcast_id, &file_path.1, conn).unwrap();
    }

    pub fn cleanup_old_episode(episode: PodcastEpisode) -> Result<(), CustomError> {
        log::info!("Cleaning up old episode: {}", episode.episode_id);
        let splitted_url = episode.url.split('/').collect::<Vec<&str>>();
        match splitted_url.len() as i32 == MAX_FILE_TREE_DEPTH {
           true=>{
               let path = move_one_path_up(&episode.file_image_path.unwrap());
               std::fs::remove_dir_all(&path).map_err(|v|map_io_error(v, Some(path)))
           }
            false=>{
                if episode.file_episode_path.is_some() {
                    std::fs::remove_file(episode.file_episode_path.clone().unwrap()).map_err
                    (|e|map_io_error(e, episode.file_episode_path))?;
                }
                if episode.file_image_path.is_some(){
                    std::fs::remove_file(episode.file_image_path.clone().unwrap()).map_err
                    (|e|map_io_error(e, episode.file_image_path))?;
                }
                Ok(())
            }
        }
    }

    pub fn delete_podcast_files(podcast_dir: &str){
        std::fs::remove_dir_all(podcast_dir).expect("Error deleting podcast directory");
    }
}


fn move_one_path_up(path: &str) -> String {
    const SEPARATOR: &str = "/";
    let mut split = path.split(SEPARATOR).collect::<Vec<&str>>();
    split.pop();

    split.join(SEPARATOR)
}

pub fn prepare_podcast_title_to_directory(title: &str, conn:&mut DbConnection) ->Result<String,
    CustomError> {
    let mut settings_service = SettingsService::new();
    let retrieved_settings = settings_service.get_settings(conn)?.unwrap();
    let final_string = perform_replacement(title, retrieved_settings.clone());

    let fixed_string = retrieved_settings.podcast_format.replace("{}","{podcasttitle}");

    let mut vars:HashMap<String,&str> = HashMap::new();
    vars.insert("podcasttitle".to_string(), &final_string);
    Ok(strfmt::strfmt(&fixed_string, &vars).unwrap())
}

pub fn prepare_podcast_episode_title_to_directory(podcast_episode: PodcastEpisode, conn: &mut
DbConnection)
    ->Result<String, CustomError> {
    let mut settings_service = SettingsService::new();
    let retrieved_settings = settings_service.get_settings(conn)?.unwrap();
    if retrieved_settings.use_existing_filename{
        let res_of_filename = get_filename_of_url(&podcast_episode.url);
        if let Ok(res_unwrapped) =res_of_filename {
                return Ok(res_unwrapped);
        }
    }
    let final_string = perform_replacement(&podcast_episode.name,
                                           retrieved_settings.clone())
        .replace(|c: char| !c.is_ascii(), "");

    let fixed_string = retrieved_settings.episode_format.replace("{}", "{episodetitle}")
        .chars()
        .filter(|&c| c as u32!= 44)
        .collect::<String>();
    let mut vars:HashMap<String, &str> = HashMap::new();
    vars.insert("episodetitle".to_string(), &final_string);

    Ok(format!("'{}'",strfmt::strfmt(&fixed_string.trim(), &vars).unwrap()))
}

fn perform_replacement(title: &str, retrieved_settings:Setting) -> String {
    let mut final_string: String = title.to_string();


    // If checked replace all illegal characters
    if retrieved_settings.replace_invalid_characters {
        let illegal_chars_regex = Regex::new(r#"[<>"/\\|?*”“„]"#).unwrap();
        final_string = illegal_chars_regex.replace_all(&final_string.clone(), "").to_string();
    }

    // Colon replacement strategy
    match ReplacementStrategy::from_str(&retrieved_settings.replacement_strategy).unwrap() {
        ReplacementStrategy::ReplaceWithDashAndUnderscore => {
            final_string = final_string.replace(':', " - ")
        }
        ReplacementStrategy::Remove => {
            final_string = final_string.replace(':', "")
        }
        ReplacementStrategy::ReplaceWithDash => {
            final_string = final_string.replace(':', "-")
        }
    }
    deunicode::deunicode(&final_string).trim().to_string()
}


fn get_filename_of_url(url: &str) -> Result<String,String> {
    let re = Regex::new(r"/([^/?]+)\.\w+(?:\?.*)?$").unwrap();

    if let Some(captures) = re.captures(url) {
        let dir_name = remove_extension(captures.get(1).unwrap().as_str()).to_string();

        return Ok(dir_name)
    }
    Err("Could not get filename".to_string())
}

fn remove_extension(filename: &str) -> &str {
    if let Some(dot_idx) = filename.rfind('.') {
        &filename[..dot_idx]
    } else {
        filename
    }
}


#[cfg(test)]
mod tests{
    use crate::service::file_service::{perform_replacement};

    #[test]
    fn test_remove_file_suffix(){
        let filename = "test.mp3";
        let filename_without_suffix = super::remove_extension(filename);
        assert_eq!(filename_without_suffix,"test");
    }

    #[test]
    fn test_remove_file_suffix_long_name(){
        let filename = "testz398459345z!?234.mp3";
        let filename_without_suffix = super::remove_extension(filename);
        assert_eq!(filename_without_suffix,"testz398459345z!?234");
    }

    #[test]
    fn get_filename_of_url_test(){
        let url = "https://www.example.com/test.mp3";
        let filename = super::get_filename_of_url(url);
        assert_eq!(filename.unwrap(),"test");
    }

    #[test]
    fn get_filename_of_url_test_with_numbers(){
        let url = "https://www.example823459274892347.com234/mypodcast.mp3";
        let filename = super::get_filename_of_url(url);
        assert_eq!(filename.unwrap(),"mypodcast");
    }

    #[test]
    fn test_perform_replacement_dash_and_underscore(){
        let title = "test: test";
        let settings = crate::models::settings::Setting{
            id: 1,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "test{podcasttitle}".to_string(),
            episode_format: "test123{episodetitle}".to_string(),
            replacement_strategy: "replace-with-dash-and-underscore".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,

            podcast_prefill: 0,
            direct_paths: false,
        };

        let result = perform_replacement(title, settings);

        assert_eq!(result,"test -  test");
    }

    #[test]
    fn test_perform_replacement_remove(){
        let title = "test: test";
        let settings = crate::models::settings::Setting{
            id: 1,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "test{podcasttitle}".to_string(),
            episode_format: "test123{episodetitle}".to_string(),
            replacement_strategy: "remove".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,

            podcast_prefill: 0,
            direct_paths: false,
        };

        let result = perform_replacement(title, settings);

        assert_eq!(result,"test test");
    }

    #[test]
    fn test_perform_replacement_replace_with_dash(){
        let title = "test: test";
        let settings = crate::models::settings::Setting{
            id: 1,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "test{podcasttitle}".to_string(),
            episode_format: "test123{episodetitle}".to_string(),
            replacement_strategy: "replace-with-dash".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,

            podcast_prefill: 0,
            direct_paths: false,
        };

        let result = perform_replacement(title, settings);

        assert_eq!(result,"test- test");
    }
}