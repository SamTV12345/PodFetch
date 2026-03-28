use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Storage IO error for path '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Storage backend error: {message}")]
    Backend { message: String },
}
