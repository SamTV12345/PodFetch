use crate::utils::error::CustomError;
use async_trait::async_trait;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;

#[async_trait]
pub trait FileHandler: Sync + Send {
    fn read_file(&self, path: &str) -> Result<String, CustomError>;
    fn write_file(&self, path: &str, content: &mut [u8]) -> Result<(), CustomError>;
    fn write_file_async<'a>(
        &'a self,
        path: &'a str,
        content: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), CustomError>> + 'a>>;
    fn create_dir(&self, path: &str) -> Result<(), CustomError>;
    fn path_exists(&self, path: &str, req: FileRequest) -> bool;
    fn remove_dir(&self, path: &str) -> Result<(), CustomError>;
    fn remove_file(&self, path: &str) -> Result<(), CustomError>;
    fn get_type(&self) -> FileHandlerType;
}

#[derive(PartialEq)]
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

pub enum FileRequest {
    Directory,
    File,
    NoopS3,
}
