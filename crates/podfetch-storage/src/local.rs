use crate::StorageError;
use std::fs::File;
use std::future::Future;
use std::io;
use std::pin::Pin;

#[derive(Clone, Default)]
pub struct LocalStorageBackend;

impl LocalStorageBackend {
    pub fn read_file(path: &str) -> Result<String, StorageError> {
        Ok(path.to_string())
    }

    pub fn write_file(file_path: &str, content: &mut [u8]) -> Result<(), StorageError> {
        let mut file_to_create = File::create(file_path).map_err(|source| StorageError::Io {
            path: file_path.to_string(),
            source,
        })?;
        io::copy::<&[u8], File>(&mut &*content, &mut file_to_create).map_err(|source| {
            StorageError::Io {
                path: file_path.to_string(),
                source,
            }
        })?;

        Ok(())
    }

    pub fn create_dir(path: &str) -> Result<(), StorageError> {
        std::fs::create_dir_all(path).map_err(|source| StorageError::Io {
            path: path.to_string(),
            source,
        })?;
        Ok(())
    }

    pub fn path_exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    pub fn remove_dir(path: &str) -> Result<(), StorageError> {
        std::fs::remove_dir_all(path).map_err(|source| StorageError::Io {
            path: path.to_string(),
            source,
        })
    }

    pub fn remove_file(path: &str) -> Result<(), StorageError> {
        std::fs::remove_file(path).map_err(|source| StorageError::Io {
            path: path.to_string(),
            source,
        })
    }

    pub fn rename_file(src: &str, dst: &str) -> Result<(), StorageError> {
        if let Some(parent) = std::path::Path::new(dst).parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|source| StorageError::Io {
                path: parent.to_string_lossy().into_owned(),
                source,
            })?;
        }

        if let Err(rename_err) = std::fs::rename(src, dst) {
            // Same-filesystem rename failed (often EXDEV on Unix or
            // ERROR_NOT_SAME_DEVICE on Windows when crossing volumes); fall
            // back to copy + delete so cross-device moves still work.
            if std::fs::copy(src, dst).is_err() {
                return Err(StorageError::Io {
                    path: src.to_string(),
                    source: rename_err,
                });
            }
            std::fs::remove_file(src).map_err(|source| StorageError::Io {
                path: src.to_string(),
                source,
            })?;
        }
        Ok(())
    }

    pub fn write_file_async<'a>(
        path: &'a str,
        content: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send + 'a>> {
        Box::pin(async move { Self::write_file(path, content) })
    }
}
