use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::utils::error::CustomError;
use async_trait::async_trait;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;

#[async_trait]
pub trait FileHandler: Sync + Send {
    fn read_file(path: &str) -> Result<String, CustomError>;
    fn write_file(path: &str, content: &mut [u8]) -> Result<(), CustomError>;
    fn write_file_async<'a>(
        path: &'a str,
        content: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), CustomError>> + 'a>>;
    fn create_dir(path: &str) -> Result<(), CustomError>;
    fn path_exists(path: &str, req: FileRequest) -> bool;
    fn remove_dir(path: &str) -> Result<(), CustomError>;
    fn remove_file(path: &str) -> Result<(), CustomError>;
}

#[derive(PartialEq, Clone)]
pub enum FileHandlerType {
    Local,
    S3,
}

impl Display for FileHandlerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileHandlerType::Local => write!(f, "Local"),
            FileHandlerType::S3 => write!(f, "S3"),
        }
    }
}

impl From<&str> for FileHandlerType {
    fn from(value: &str) -> Self {
        match value {
            "Local" => FileHandlerType::Local,
            "S3" => FileHandlerType::S3,
            _ => panic!("Invalid FileHandlerType"),
        }
    }
}

impl From<Option<String>> for FileHandlerType {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(val) => FileHandlerType::from(val.as_str()),
            None => ENVIRONMENT_SERVICE.default_file_handler.clone(),
        }
    }
}

pub enum FileRequest {
    Directory,
    File,
    NoopS3,
}
