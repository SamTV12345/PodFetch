use crate::file_handler::resolve_file_handler_type;
use crate::{FileRequest, LocalStorageBackend, S3StorageBackend, StorageError};
use common_infrastructure::config::FileHandlerType;
use common_infrastructure::error::ErrorSeverity::Critical;
use common_infrastructure::error::{CustomError, CustomErrorInner, map_io_error};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use std::future::Future;
use std::pin::Pin;

/// Represents a podcast episode's file information needed for cleanup.
pub struct EpisodeFileInfo {
    pub download_location: Option<String>,
    pub file_image_path: Option<String>,
    pub file_episode_path: Option<String>,
}

/// Represents a podcast's information needed for directory removal.
pub struct PodcastFileInfo {
    pub id: i32,
    pub image_url: String,
    pub directory_name: String,
    pub download_location: Option<String>,
}

pub struct FileHandleWrapper;

impl FileHandleWrapper {
    fn s3_backend() -> S3StorageBackend {
        S3StorageBackend::new(ENVIRONMENT_SERVICE.s3_config.clone())
    }

    fn map_storage_error(error: StorageError) -> CustomError {
        match error {
            StorageError::Io { path, source } => map_io_error(source, Some(path), Critical),
            StorageError::Backend { message } => {
                CustomErrorInner::Conflict(message, Critical).into()
            }
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

    pub fn remove_dir(
        podcast: &PodcastFileInfo,
        episodes: &[EpisodeFileInfo],
    ) -> Result<(), CustomError> {
        match resolve_file_handler_type(podcast.download_location.clone()) {
            FileHandlerType::Local => {
                LocalStorageBackend::remove_dir(&podcast.directory_name)
                    .map_err(Self::map_storage_error)?;
                episodes.iter().for_each(|episode| {
                    if let Some(download_type) = &episode.download_location {
                        let file_type = FileHandlerType::from(download_type.as_str());
                        if FileHandlerType::S3 == file_type {
                            if let Some(file_path) = &episode.file_image_path
                                && let Err(e) = Self::s3_backend()
                                    .remove_file(file_path)
                                    .map_err(Self::map_storage_error)
                            {
                                tracing::error!("Error removing file: {file_path} with reason {e}");
                            }
                            if let Some(file_path) = &episode.file_episode_path
                                && let Err(e) = Self::s3_backend()
                                    .remove_file(file_path)
                                    .map_err(Self::map_storage_error)
                            {
                                tracing::error!("Error removing file: {file_path} with reason {e}");
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
                episodes.iter().for_each(|episode| {
                    if let Some(download_type) = &episode.download_location {
                        let file_type = FileHandlerType::from(download_type.as_str());
                        if FileHandlerType::S3 == file_type {
                            if let Some(file_path) = &episode.file_image_path
                                && let Err(e) = Self::s3_backend()
                                    .remove_file(file_path)
                                    .map_err(Self::map_storage_error)
                            {
                                tracing::error!("Error removing file: {file_path} with reason {e}");
                            }
                            if let Some(file_path) = &episode.file_episode_path
                                && let Err(e) = Self::s3_backend()
                                    .remove_file(file_path)
                                    .map_err(Self::map_storage_error)
                            {
                                tracing::error!("Error removing file: {file_path} {e}");
                            }
                        } else {
                            if let Some(file_path) = &episode.file_image_path
                                && let Err(e) = LocalStorageBackend::remove_file(file_path)
                                    .map_err(Self::map_storage_error)
                            {
                                tracing::error!("Error removing file: {file_path} with reason {e}");
                            }
                            if let Some(file_path) = &episode.file_episode_path
                                && let Err(e) = LocalStorageBackend::remove_file(file_path)
                                    .map_err(Self::map_storage_error)
                            {
                                tracing::error!("Error removing file: {file_path} {e}");
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
