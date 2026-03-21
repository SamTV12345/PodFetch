use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::utils::error::CustomError;
use async_trait::async_trait;
pub use common_infrastructure::config::FileHandlerType;
use std::future::Future;
use std::pin::Pin;

#[async_trait]
pub trait FileHandler: Sync + Send {
    fn read_file(path: &str) -> Result<String, CustomError>;
    fn write_file(path: &str, content: &mut [u8]) -> Result<(), CustomError>;
    fn write_file_async<'a>(
        path: &'a str,
        content: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), CustomError>> + Send + 'a>>;
    fn create_dir(path: &str) -> Result<(), CustomError>;
    fn path_exists(path: &str, req: FileRequest) -> bool;
    fn remove_dir(path: &str) -> Result<(), CustomError>;
    fn remove_file(path: &str) -> Result<(), CustomError>;
}

pub fn resolve_file_handler_type(value: Option<String>) -> FileHandlerType {
    match value {
        Some(val) => FileHandlerType::from(val.as_str()),
        None => ENVIRONMENT_SERVICE.default_file_handler.clone(),
    }
}

pub enum FileRequest {
    Directory,
    File,
    NoopS3,
}
