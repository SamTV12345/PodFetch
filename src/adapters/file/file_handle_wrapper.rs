use crate::adapters::file::file_handler::{FileHandler, FileHandlerType, FileRequest};
use crate::adapters::file::local_file_handler::LocalFileHandler;
use crate::adapters::file::s3_file_handler::S3Handler;
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
    ) -> Pin<Box<dyn Future<Output = Result<(), CustomError>> + 'a>> {
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
    pub fn remove_dir(path: &str, download_location: &FileHandlerType) -> Result<(), CustomError> {
        match download_location {
            FileHandlerType::Local => LocalFileHandler::remove_dir(path),
            FileHandlerType::S3 => S3Handler::remove_dir(path),
        }
    }
    pub fn remove_file(path: &str, download_location: &FileHandlerType) -> Result<(), CustomError> {
        match download_location {
            FileHandlerType::Local => LocalFileHandler::remove_file(path),
            FileHandlerType::S3 => S3Handler::remove_file(path),
        }
    }
}
