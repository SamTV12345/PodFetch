use crate::adapters::file::file_handler::{FileHandler, FileHandlerType, FileRequest};
use crate::adapters::file::local_file_handler::LocalFileHandler;
use crate::adapters::file::s3_file_handler::S3Handler;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::utils::error::CustomError;
use std::future::Future;
use std::pin::Pin;

pub struct FileHandleWrapper;

impl FileHandleWrapper {
    pub fn write_file(
        path: &str,
        content: &mut [u8],
        download_location: &FileHandlerType,
    ) -> Result<(), CustomError> {
        match download_location {
            FileHandlerType::Local => LocalFileHandler::write_file(path, content),
            FileHandlerType::S3 => S3Handler::write_file(path, content),
        }
    }
    pub fn write_file_async<'a>(
        path: &'a str,
        content: &'a mut [u8],
        download_location: &FileHandlerType,
    ) -> Pin<Box<dyn Future<Output = Result<(), CustomError>> + Send + 'a>> {
        match download_location {
            FileHandlerType::Local => LocalFileHandler::write_file_async(path, content),
            FileHandlerType::S3 => S3Handler::write_file_async(path, content),
        }
    }
    pub fn create_dir(path: &str, download_location: &FileHandlerType) -> Result<(), CustomError> {
        match download_location {
            FileHandlerType::Local => LocalFileHandler::create_dir(path),
            FileHandlerType::S3 => S3Handler::create_dir(path),
        }
    }
    pub fn path_exists(path: &str, req: FileRequest, download_location: &FileHandlerType) -> bool {
        match download_location {
            FileHandlerType::Local => LocalFileHandler::path_exists(path, req),
            FileHandlerType::S3 => S3Handler::path_exists(path, req),
        }
    }
    pub fn remove_dir(podcast: &Podcast) -> Result<(), CustomError> {
        match FileHandlerType::from(podcast.download_location.clone()) {
            FileHandlerType::Local => {
                // Remove the directory
                LocalFileHandler::remove_dir(&podcast.directory_name)?;
                PodcastEpisode::get_episodes_by_podcast_id(podcast.id)?
                    .iter()
                    .for_each(|episode| {
                        // Remove the episode directory
                        if let Some(download_type) = &episode.download_location {
                            let file_type = FileHandlerType::from(download_type.as_str());
                            if FileHandlerType::S3 == file_type {
                                if let Some(file_path) = &episode.file_image_path {
                                    if let Err(e) = S3Handler::remove_file(file_path) {
                                        log::error!(
                                            "Error removing file: {file_path} with reason {e}"
                                        );
                                    }
                                }
                                if let Some(file_path) = &episode.file_episode_path {
                                    if let Err(e) = S3Handler::remove_file(file_path) {
                                        log::error!(
                                            "Error removing file: {file_path} with reason {e}"
                                        );
                                    }
                                }
                            }
                        }
                    });
                Ok(())
            }
            FileHandlerType::S3 => {
                let image_url = urlencoding::decode(&podcast.image_url).unwrap().to_string();
                S3Handler::remove_file(&image_url)?;
                PodcastEpisode::get_episodes_by_podcast_id(podcast.id)?
                    .iter()
                    .for_each(|episode| {
                        // Remove the episode directory
                        if let Some(download_type) = &episode.download_location {
                            let file_type = FileHandlerType::from(download_type.as_str());
                            if FileHandlerType::S3 == file_type {
                                if let Some(file_path) = &episode.file_image_path {
                                    if let Err(e) = S3Handler::remove_file(file_path) {
                                        log::error!(
                                            "Error removing file: {file_path} with reason {e}"
                                        );
                                    }
                                }
                                if let Some(file_path) = &episode.file_episode_path {
                                    if let Err(e) = S3Handler::remove_file(file_path) {
                                        log::error!("Error removing file: {file_path} {e}");
                                    }
                                }
                            } else {
                                if let Some(file_path) = &episode.file_image_path {
                                    if let Err(e) = LocalFileHandler::remove_file(file_path) {
                                        log::error!(
                                            "Error removing file: {file_path} with reason {e}"
                                        );
                                    }
                                }
                                if let Some(file_path) = &episode.file_episode_path {
                                    if let Err(e) = LocalFileHandler::remove_file(file_path) {
                                        log::error!("Error removing file: {file_path} {e}");
                                    }
                                }
                            }
                        }
                    });
                Ok(())
            }
        }
    }
    pub fn remove_file(path: &str, download_location: &FileHandlerType) -> Result<(), CustomError> {
        match download_location {
            FileHandlerType::Local => LocalFileHandler::remove_file(path),
            FileHandlerType::S3 => S3Handler::remove_file(path),
        }
    }
}
