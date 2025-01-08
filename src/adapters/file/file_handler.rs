use std::future::Future;
use std::pin::Pin;
use async_trait::async_trait;
use crate::utils::error::CustomError;

#[async_trait]
pub trait FileHandler: Sync + Send {
    fn read_file(&self, path: &str) -> Result<String, CustomError>;
    fn write_file(&self, path: &str, content: &mut [u8]) -> Result<(), CustomError>;
    fn write_file_async<'a>(&'a self, path: &'a str, content: &'a mut [u8]) -> Pin<Box<dyn
    Future<Output =
    Result<(),
        CustomError>> + 'a>>;
    fn create_dir(&self, path: &str) -> Result<(), CustomError>;
    fn path_exists(&self, path: &str, req: FileRequest) -> bool;
    fn remove_dir(&self, path: &str) -> Result<(), CustomError>;
    fn remove_file(&self, path: &str) -> Result<(), CustomError>;
    fn get_type(&self) -> FileHandlerType;
}

#[derive(PartialEq)]
pub enum FileHandlerType {
    Local,
    S3
}


pub enum FileRequest {
    Directory,
    File,
    NoopS3
}