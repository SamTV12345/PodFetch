use crate::adapters::file::file_handle_wrapper::FileHandleWrapper;
use crate::adapters::file::file_handler::{FileHandlerType, FileRequest};
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
        let dir_exists = FileHandleWrapper::path_exists(
            base_path,
            FileRequest::Directory,
            &FileHandlerType::from(_podcast.download_location.clone()),
        );
        if !dir_exists {
            FileHandleWrapper::create_dir(
                base_path,
                &FileHandlerType::from(_podcast.download_location.clone()),
            )?;
            return Ok(base_path.to_string());
        }

        while FileHandleWrapper::path_exists(
            &format!("{}-{}", base_path, i),
            FileRequest::NoopS3,
            &FileHandlerType::from(_podcast.download_location.clone()),
        ) {
            i += 1;
        }
        let final_path = format!("{}-{}", base_path, i);
        // This is safe to insert because this directory does not exist
        FileHandleWrapper::create_dir(
            &final_path,
            &FileHandlerType::from(_podcast.download_location.clone()),
        )?;
        Ok(final_path)
    }
}
