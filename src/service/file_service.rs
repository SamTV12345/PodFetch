use std::io::Write;
use std::path::Path;
use reqwest::{Client, ClientBuilder};
use rss::Error;
use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::podcast_episode_service::PodcastEpisodeService;

#[derive(Clone)]
pub struct FileService {
    pub db: DB,
    pub client: Client
}

impl FileService {
        pub fn new() -> Self {
            FileService {
                db: DB::new().unwrap(),
                client: ClientBuilder::new().build().unwrap()
            }
        }
        pub fn check_if_podcast_main_image_downloaded(&mut self,podcast_id: &str) -> bool {
            let podcast = self.db.clone().get_podcast_by_directory(podcast_id).unwrap();
            match podcast {
                Some(podcast) => {
                    if !podcast.image_url.contains("http") {
                        return Path::new(&podcast.image_url).exists()
                    }
                }
                None => {
                    return false;
                }
            }
            return false;
        }

        pub fn create_podcast_root_directory_exists(){
            if !Path::new("podcasts").exists() {
                std::fs::create_dir("podcasts").expect("Error creating directory");
            }
        }

        pub fn create_podcast_directory_exists(podcast_id: &str){
            if !Path::new(&format!("podcasts/{}",podcast_id)).exists() {
                std::fs::create_dir(&format!("podcasts/{}",podcast_id))
                    .expect("Error creating directory");
            }
        }

        pub async fn download_podcast_image(&self,podcast_id: &str, image_url: &str){
            let image_response = self.client.get(image_url).send().await.unwrap();
            let image_suffix = PodcastEpisodeService::get_url_file_suffix(image_url);
            let file_path = format!("podcasts/{}/image.{}", podcast_id, image_suffix);
            let mut image_out = std::fs::File::create(file_path.clone())
                .unwrap();
            let bytes  =image_response.bytes().await.unwrap();
            image_out.write_all(&bytes).unwrap();
            let db = DB::new().unwrap();
            println!("Before update: {}", file_path);
            db.update_podcast_image(podcast_id, &file_path).unwrap();
        }

    pub fn cleanup_old_episode(podcast: Podcast, episode: PodcastEpisode) -> std::io::Result<()> {
        std::fs::remove_dir(&format!("podcasts/{}/{}", podcast.directory, episode
            .episode_id))
    }
}
