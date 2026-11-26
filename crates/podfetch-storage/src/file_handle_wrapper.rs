use crate::file_handler::resolve_file_handler_type;
use crate::{FileRequest, LocalStorageBackend, S3StorageBackend, StorageError};
use common_infrastructure::config::FileHandlerType;
use common_infrastructure::error::ErrorSeverity::Critical;
use common_infrastructure::error::{CustomError, CustomErrorInner, map_io_error};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use std::borrow::Cow;
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
                match LocalStorageBackend::remove_dir(&podcast.directory_name) {
                    Ok(()) => {}
                    // Already gone (removed by hand or by an interrupted
                    // delete): that is the state we want. Still clean up the
                    // episodes stored in S3 below.
                    Err(StorageError::Io { source, .. })
                        if source.kind() == std::io::ErrorKind::NotFound =>
                    {
                        tracing::debug!(
                            "Podcast directory {} is already gone",
                            podcast.directory_name
                        );
                    }
                    Err(err) => return Err(Self::map_storage_error(err)),
                }
                // Local episode files went away with the directory.
                Self::remove_episode_files(episodes, false);
            }
            FileHandlerType::S3 => {
                // Not percent-encoded UTF-8? Then the stored value is the key.
                let image_url = urlencoding::decode(&podcast.image_url)
                    .map_or_else(|_| podcast.image_url.clone(), Cow::into_owned);
                Self::s3_backend()
                    .remove_file(&image_url)
                    .map_err(Self::map_storage_error)?;
                Self::remove_episode_files(episodes, true);
            }
        }
        Ok(())
    }

    /// Best-effort removal of episode files and images; failures are logged.
    /// S3 files are always removed, local ones only if `include_local`.
    fn remove_episode_files(episodes: &[EpisodeFileInfo], include_local: bool) {
        for episode in episodes {
            let Some(location) = episode.download_location.as_deref() else {
                continue;
            };
            let is_s3 = FileHandlerType::from_stored(location) == FileHandlerType::S3;
            if !is_s3 && !include_local {
                continue;
            }
            for path in [&episode.file_image_path, &episode.file_episode_path]
                .into_iter()
                .flatten()
            {
                let result = if is_s3 {
                    Self::s3_backend().remove_file(path)
                } else {
                    LocalStorageBackend::remove_file(path)
                };
                if let Err(e) = result {
                    tracing::error!("Error removing file {path}: {e}");
                }
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

    pub fn rename_file(
        src: &str,
        dst: &str,
        download_location: &FileHandlerType,
    ) -> Result<(), CustomError> {
        match download_location {
            FileHandlerType::Local => {
                LocalStorageBackend::rename_file(src, dst).map_err(Self::map_storage_error)
            }
            FileHandlerType::S3 => Err(CustomErrorInner::Conflict(
                "rename_file is not supported for the S3 backend".to_string(),
                Critical,
            )
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FileHandleWrapper, PodcastFileInfo};

    fn local_podcast(directory: &str) -> PodcastFileInfo {
        PodcastFileInfo {
            id: 0,
            image_url: String::new(),
            directory_name: directory.to_string(),
            download_location: Some("Local".to_string()),
        }
    }

    #[test]
    fn removing_a_missing_local_directory_succeeds() {
        let missing = std::env::temp_dir().join(format!("podfetch-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&missing);
        assert!(
            FileHandleWrapper::remove_dir(&local_podcast(missing.to_str().unwrap()), &[]).is_ok()
        );
    }

    #[test]
    fn removing_an_existing_local_directory_deletes_it() {
        let dir = std::env::temp_dir().join(format!("podfetch-existing-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub").join("file.mp3"), b"x").unwrap();
        FileHandleWrapper::remove_dir(&local_podcast(dir.to_str().unwrap()), &[]).unwrap();
        assert!(!dir.exists());
    }
}
