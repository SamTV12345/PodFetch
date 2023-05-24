use std::io;
use std::path::Path;
use crate::DbConnection;
use crate::models::itunes_models::{Podcast, PodcastEpisode};

use crate::service::file_service::{FileService, prepare_podcast_episode_title_to_directory};


pub struct PathService {}

impl PathService {
    pub fn get_podcast_episode_path(directory: &str, episode: Option<PodcastEpisode>, suffix:
    &str, filename: &str, conn: &mut DbConnection)
        -> String {
        return match episode {
            Some(episode) => {
                format!("{}/{}/podcast.{}", directory,
                        prepare_podcast_episode_title_to_directory(episode, conn), suffix)
            },
            None => {
                format!("{}/{}/podcast.{}", directory, filename, suffix)
            }
        }
    }

    pub fn get_image_path(directory: &str, episode: Option<PodcastEpisode>, _suffix: &str,
                          filename: &str,
                          conn: &mut DbConnection) -> String {
        return match episode {
            Some(episode) => {
                format!("{}/{}", directory, prepare_podcast_episode_title_to_directory(episode,
                                                                                       conn))
            },
            None => {
                format!("{}/{}", directory, filename)
            }
        }
    }

    pub fn get_image_podcast_path_with_podcast_prefix(directory: &str, suffix: &str) -> String {
        return format!("{}/image.{}", directory, suffix);
    }

    pub fn check_if_podcast_episode_directory_available(base_path:&str, podcast: Podcast,conn: &mut DbConnection) ->
                                                                                          String {
        let mut i = 0;
        if !Path::new(&base_path).exists() {
            std::fs::create_dir(&base_path).map_err(|e| {
                Self::handle_error_when_creating_directory(podcast, conn);
                return e
            }).expect("Error creating directory");
            return base_path.to_string();
        }

        while Path::new(&format!("{}-{}",base_path, i)).exists() {
            i += 1;
        }
        let final_path = format!("{}-{}",base_path, i);
        // This is save to insert because this directory does not exist
        std::fs::create_dir(&final_path)
            .map_err(|e| {
                Self::handle_error_when_creating_directory(podcast, conn);
                return e
            }).expect("Error creating directory");
        return final_path;
    }

    fn handle_error_when_creating_directory(podcast:Podcast,conn: &mut DbConnection){
                match FileService::create_podcast_root_directory_exists(){
                    Ok(_) => {}
                    Err(e) => {
                        if e.kind()==io::ErrorKind::AlreadyExists {
                            log::info!("Podcast root directory already exists")
                        }
                        else {
                            log::error!("Error creating podcast root directory");
                        }

                match FileService::create_podcast_directory_exists(&podcast.name, &podcast
                    .directory_id, conn) {
                    Ok(_) => {}
                    Err(e) => {
                        if e.kind()==io::ErrorKind::AlreadyExists {
                            log::info!("Podcast directory already exists")
                        }
                        else {
                            log::error!("Error creating podcast directory {}",e);
                        }
                    }
                }
            }
        }
    }
}
