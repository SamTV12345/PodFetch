use std::path::Path;
use crate::models::itunes_models::PodcastEpisode;

use crate::service::file_service::{prepare_podcast_episode_title_to_directory, prepare_podcast_title_to_directory};


pub struct PathService {}

impl PathService {
    pub fn get_podcast_episode_path(directory: &str, episode: Option<PodcastEpisode>, suffix: &str, filename: &str)
        -> String {
        return match episode {
            Some(episode) => {
                format!("{}/{}/podcast.{}", directory,
                        prepare_podcast_episode_title_to_directory(episode), suffix)
            },
            None => {
                format!("{}/{}/podcast.{}", directory, filename, suffix)
            }
        }
    }

    pub fn get_image_path(directory: &str, episode: Option<PodcastEpisode>, _suffix: &str, filename: &str) -> String {
        return match episode {
            Some(episode) => {
                format!("{}/{}", directory, prepare_podcast_episode_title_to_directory(episode))
            },
            None => {
                format!("{}/{}", directory, filename)
            }
        }
    }

    pub fn get_image_podcast_path(directory: &str, suffix: &str) -> String {
        return format!("podcasts/{}/image.{}", prepare_podcast_title_to_directory(directory),
            suffix);
    }

    pub fn get_image_podcast_path_with_podcast_prefix(directory: &str, suffix: &str) -> String {
        return format!("{}/image.{}", directory, suffix);
    }

    pub fn check_if_podcast_episode_directory_available(directory:&str, escaped_title: &str) -> String {
        let mut i = 0;
        let base_path = format!("{}/{}",directory, escaped_title);
        if !Path::new(&base_path).exists() {
            return base_path;
        }

        while Path::new(&format!("{}/{}-{}",directory, escaped_title, i)).exists() {
            i += 1;
        }
        let final_path = format!("{}/{}-{}",directory, escaped_title, i);
        // This is save to insert because this directory does not exist
        std::fs::create_dir(&final_path)
            .expect("Error creating directory");
        return final_path;

    }
}
