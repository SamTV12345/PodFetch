use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::podcast_episode_service::PodcastEpisodeService;
use reqwest::{Client, ClientBuilder};
use std::io::{Error, Write};
use std::path::Path;
use regex::Regex;

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
                .expect("Error creating directory");
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
    let re = Regex::new(r"[^a-zA-Z0-9_./]").unwrap();
    let res = re.replace_all(title, "").to_string();
    res.replace("..","")
}