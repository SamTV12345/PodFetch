use crate::utils::error::CustomError;

pub trait FileHandler: Sync + Send {
    fn read_file(&self, path: &str) -> Result<String, CustomError>;
    fn write_file(&self, path: &str, content: &mut [u8]) -> Result<(), CustomError>;
    fn create_dir(&self, path: &str) -> Result<(), CustomError>;
    fn path_exists(&self, path: &str, req: FileRequest) -> bool;
    fn remove_dir(&self, path: &str) -> Result<(), CustomError>;
    fn remove_file(&self, path: &str) -> Result<(), CustomError>;
}


pub enum FileRequest {
    Directory,
    File,
    NoopS3
}