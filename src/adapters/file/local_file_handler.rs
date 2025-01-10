use crate::adapters::file::file_handler::{FileHandler, FileHandlerType, FileRequest};
use crate::utils::error::{map_io_error, CustomError};
use std::fs::File;
use std::future::Future;
use std::io;
use std::pin::Pin;

#[derive(Clone)]
pub struct LocalFileHandler;

impl LocalFileHandler {
    pub fn new() -> Self {
        LocalFileHandler
    }
}

impl FileHandler for LocalFileHandler {
    fn read_file(&self, path: &str) -> Result<String, CustomError> {
        Ok(path.to_string())
    }

    fn write_file(&self, file_path: &str, content: &mut [u8]) -> Result<(), CustomError> {
        let mut file_to_create =
            File::create(file_path).map_err(|s| map_io_error(s, Some(file_path.to_string())))?;
        io::copy::<&[u8], File>(&mut &*content, &mut file_to_create)
            .map_err(|s| map_io_error(s, Some(file_path.to_string())))?;

        Ok(())
    }

    fn create_dir(&self, path: &str) -> Result<(), CustomError> {
        std::fs::create_dir(path).map_err(|s| map_io_error(s, Some(path.to_string())))?;
        Ok(())
    }

    fn path_exists(&self, path: &str, _: FileRequest) -> bool {
        std::path::Path::new(path).exists()
    }
    fn remove_dir(&self, path: &str) -> Result<(), CustomError> {
        std::fs::remove_dir_all(path).map_err(|e| map_io_error(e, Some(path.to_string())))
    }

    fn remove_file(&self, path: &str) -> Result<(), CustomError> {
        std::fs::remove_file(path).map_err(|e| map_io_error(e, Some(path.to_string())))
    }

    fn write_file_async<'a>(
        &'a self,
        path: &'a str,
        content: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), CustomError>> + 'a>> {
        Box::pin(async { self.write_file(path, content) })
    }

    fn get_type(&self) -> FileHandlerType {
        FileHandlerType::Local
    }
}
