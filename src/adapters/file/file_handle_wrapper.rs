use crate::adapters::file::file_handler::{
    FileHandlerType, FileRequest, resolve_file_handler_type,
};
use crate::models::podcasts::Podcast;
use crate::application::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use crate::utils::error::ErrorSeverity::Critical;
use crate::utils::error::{CustomError, CustomErrorInner, map_io_error};
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use podfetch_storage::{LocalStorageBackend, S3StorageBackend, StorageError};
use std::future::Future;
use std::pin::Pin;

pub struct FileHandleWrapper;

impl FileHandleWrapper {
    fn s3_backend() -> S3StorageBackend {
        S3StorageBackend::new(ENVIRONMENT_SERVICE.s3_config.clone())
    }

    fn map_storage_error(error: StorageError) -> CustomError {
        match error {
            StorageError::Io { path, source } => map_io_error(source, Some(path), Critical),
            StorageError::Backend { message } => CustomErrorInner::Conflict(message, Critical).into(),
        }
    }

    pub fn write_file(
        path: &str,
        content: &mut [u8],
        download_location: &FileHandlerType,
    ) -> Result<(), CustomError> {
        match download_location {
            FileHandlerType::Local => {
                LocalStorageBackend::write_file(path, content).map_err(Self::map_storage_error)
            }
            FileHandlerType::S3 => Self::s3_backend()
                .write_file(path, content)
                .map_err(Self::map_storage_error),
        }
    }
    pub fn write_file_async<'a>(
        path: &'a str,
        content: &'a mut [u8],
        download_location: &FileHandlerType,
    ) -> Pin<Box<dyn Future<Output = Result<(), CustomError>> + Send + 'a>> {
        match download_location {
            FileHandlerType::Local => Box::pin(async move {
                LocalStorageBackend::write_file_async(path, content)
                    .await
                    .map_err(Self::map_storage_error)
            }),
            FileHandlerType::S3 => {
                let backend = Self::s3_backend();
                Box::pin(async move {
                    backend
                        .write_file_async(path, content)
                        .await
                        .map_err(Self::map_storage_error)
                })
            }
        }
    }
    pub fn create_dir(path: &str, download_location: &FileHandlerType) -> Result<(), CustomError> {
        match download_location {
            FileHandlerType::Local => {
                LocalStorageBackend::create_dir(path).map_err(Self::map_storage_error)
            }
            FileHandlerType::S3 => Self::s3_backend()
                .create_dir(path)
                .map_err(Self::map_storage_error),
        }
    }
    pub fn path_exists(path: &str, req: FileRequest, download_location: &FileHandlerType) -> bool {
        match download_location {
            FileHandlerType::Local => LocalStorageBackend::path_exists(path),
            FileHandlerType::S3 => Self::s3_backend().path_exists(path, req),
        }
    }
    pub fn remove_dir(podcast: &Podcast) -> Result<(), CustomError> {
        match resolve_file_handler_type(podcast.download_location.clone()) {
            FileHandlerType::Local => {
                // Remove the directory
                LocalStorageBackend::remove_dir(&podcast.directory_name)
                    .map_err(Self::map_storage_error)?;
                PodcastEpisodeService::get_episodes_by_podcast_id(podcast.id)?
                    .iter()
                    .for_each(|episode| {
                        // Remove the episode directory
                        if let Some(download_type) = &episode.download_location {
                            let file_type = FileHandlerType::from(download_type.as_str());
                            if FileHandlerType::S3 == file_type {
                                if let Some(file_path) = &episode.file_image_path
                                    && let Err(e) = Self::s3_backend()
                                        .remove_file(file_path)
                                        .map_err(Self::map_storage_error)
                                {
                                    log::error!("Error removing file: {file_path} with reason {e}");
                                }
                                if let Some(file_path) = &episode.file_episode_path
                                    && let Err(e) = Self::s3_backend()
                                        .remove_file(file_path)
                                        .map_err(Self::map_storage_error)
                                {
                                    log::error!("Error removing file: {file_path} with reason {e}");
                                }
                            }
                        }
                    });
                Ok(())
            }
            FileHandlerType::S3 => {
                let image_url = urlencoding::decode(&podcast.image_url).unwrap().to_string();
                Self::s3_backend()
                    .remove_file(&image_url)
                    .map_err(Self::map_storage_error)?;
                PodcastEpisodeService::get_episodes_by_podcast_id(podcast.id)?
                    .iter()
                    .for_each(|episode| {
                        // Remove the episode directory
                        if let Some(download_type) = &episode.download_location {
                            let file_type = FileHandlerType::from(download_type.as_str());
                            if FileHandlerType::S3 == file_type {
                                if let Some(file_path) = &episode.file_image_path
                                    && let Err(e) = Self::s3_backend()
                                        .remove_file(file_path)
                                        .map_err(Self::map_storage_error)
                                {
                                    log::error!("Error removing file: {file_path} with reason {e}");
                                }
                                if let Some(file_path) = &episode.file_episode_path
                                    && let Err(e) = Self::s3_backend()
                                        .remove_file(file_path)
                                        .map_err(Self::map_storage_error)
                                {
                                    log::error!("Error removing file: {file_path} {e}");
                                }
                            } else {
                                if let Some(file_path) = &episode.file_image_path
                                    && let Err(e) = LocalStorageBackend::remove_file(file_path)
                                        .map_err(Self::map_storage_error)
                                {
                                    log::error!("Error removing file: {file_path} with reason {e}");
                                }
                                if let Some(file_path) = &episode.file_episode_path
                                    && let Err(e) = LocalStorageBackend::remove_file(file_path)
                                        .map_err(Self::map_storage_error)
                                {
                                    log::error!("Error removing file: {file_path} {e}");
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
            FileHandlerType::Local => {
                LocalStorageBackend::remove_file(path).map_err(Self::map_storage_error)
            }
            FileHandlerType::S3 => Self::s3_backend()
                .remove_file(path)
                .map_err(Self::map_storage_error),
        }
    }
}
