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

    pub fn get_image_path(directory: &str, episode: Option<PodcastEpisode>, suffix: &str, filename: &str) -> String {
        return match episode {
            Some(episode) => {
                format!("{}/{}/image.{}", directory, prepare_podcast_episode_title_to_directory
                    (episode), suffix)
            },
            None => {
                format!("{}/{}/image.{}", directory, filename, suffix)
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
}
