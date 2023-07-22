
use std::path::Path;
use crate::DbConnection;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;

use crate::service::file_service::{prepare_podcast_episode_title_to_directory};
use crate::utils::error::{CustomError, map_io_error};


pub struct PathService {}

impl PathService {
    pub fn get_podcast_episode_path(directory: &str, episode: Option<PodcastEpisode>, suffix:
    &str, filename: &str, conn: &mut DbConnection)
        -> Result<String, CustomError> {
        match episode {
            Some(episode) => {
                Ok(format!("{}/{}/podcast.{}", directory,
                        prepare_podcast_episode_title_to_directory(episode, conn)?, suffix))
            },
            None => {
                Ok(format!("{}/{}/podcast.{}", directory, filename, suffix))
            }
        }
    }

    pub fn get_image_path(directory: &str, episode: Option<PodcastEpisode>, _suffix: &str,
                          filename: &str,
                          conn: &mut DbConnection) -> Result<String, CustomError> {
        match episode {
            Some(episode) => {
                Ok(format!("{}/{}", directory, prepare_podcast_episode_title_to_directory(episode,
                                                                                       conn)?))
            },
            None => {
                Ok(format!("{}/{}", directory, filename))
            }
        }
    }

    pub fn get_image_podcast_path_with_podcast_prefix(directory: &str, suffix: &str) -> String {
        format!("{}/image.{}", directory, suffix)
    }

    pub fn check_if_podcast_episode_directory_available(base_path:&str, _podcast: Podcast,_conn: &mut DbConnection) ->
                                                                                          Result<String, CustomError> {
        let mut i = 0;
        if !Path::new(&base_path).exists() {
            std::fs::create_dir(base_path).map_err(map_io_error)?;
            return Ok(base_path.to_string());
        }

        while Path::new(&format!("{}-{}",base_path, i)).exists() {
            i += 1;
        }
        let final_path = format!("{}-{}",base_path, i);
        // This is save to insert because this directory does not exist
        std::fs::create_dir(&final_path)
            .map_err(map_io_error)?;
        Ok(final_path)
    }
}
