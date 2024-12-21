use crate::DBType as DbConnection;
use std::path::Path;
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::service::file_service::prepare_podcast_episode_title_to_directory;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::{map_io_error, CustomError};

pub struct PathService {}

impl PathService {
    pub fn get_image_path(
        directory: &str,
        episode: Option<PodcastEpisode>,
        _suffix: &str,
        filename: &str
    ) -> Result<String, CustomError> {
        match episode {
            Some(episode) => Ok(format!(
                "{}/{}",
                directory,
                prepare_podcast_episode_title_to_directory(episode)?
            )),
            None => Ok(format!("{}/{}", directory, filename)),
        }
    }

    pub fn get_image_podcast_path_with_podcast_prefix(
        directory: &str,
        suffix: &str,
    ) -> (String, String) {
        let file_path = format!("{}/image.{}", directory, suffix);
        let url_path = format!(
            "{}/image.{}",
            PodcastEpisodeService::map_to_local_url(directory),
            suffix
        );
        (file_path, url_path)
    }

    pub fn check_if_podcast_episode_directory_available(
        base_path: &str
    ) -> Result<String, CustomError> {
        let mut i = 0;
        if !Path::new(&base_path).exists() {
            std::fs::create_dir(base_path)
                .map_err(|v| map_io_error(v, Some(base_path.to_string())))?;
            return Ok(base_path.to_string());
        }

        while Path::new(&format!("{}-{}", base_path, i)).exists() {
            i += 1;
        }
        let final_path = format!("{}-{}", base_path, i);
        // This is save to insert because this directory does not exist
        std::fs::create_dir(&final_path)
            .map_err(|v| map_io_error(v, Some(base_path.to_string())))?;
        Ok(final_path)
    }
}
