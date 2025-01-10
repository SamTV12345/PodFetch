use crate::adapters::file::file_handler::FileRequest;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::service::file_service::prepare_podcast_episode_title_to_directory;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::CustomError;
use crate::DBType as DbConnection;

pub struct PathService {}

impl PathService {
    pub fn get_image_path(
        directory: &str,
        episode: Option<PodcastEpisode>,
        _suffix: &str,
        filename: &str,
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
        base_path: &str,
        _podcast: Podcast,
        _conn: &mut DbConnection,
    ) -> Result<String, CustomError> {
        let mut i = 0;
        let dir_exists = ENVIRONMENT_SERVICE
            .default_file_handler
            .path_exists(base_path, FileRequest::Directory);
        if !dir_exists {
            ENVIRONMENT_SERVICE
                .default_file_handler
                .create_dir(base_path)?;
            return Ok(base_path.to_string());
        }

        while ENVIRONMENT_SERVICE
            .default_file_handler
            .path_exists(&format!("{}-{}", base_path, i), FileRequest::NoopS3)
        {
            i += 1;
        }
        let final_path = format!("{}-{}", base_path, i);
        // This is safe to insert because this directory does not exist
        ENVIRONMENT_SERVICE
            .default_file_handler
            .create_dir(&final_path)?;
        Ok(final_path)
    }
}
