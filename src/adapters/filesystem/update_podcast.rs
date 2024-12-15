use std::fs;
use std::path::PathBuf;
use crate::utils::error::{map_io_error, CustomError};

pub struct UpdatePodcast;


impl UpdatePodcast {
    pub fn delete_podcast_files(podcast_dir: &PathBuf) -> Result<(), CustomError> {
        fs::remove_dir_all(podcast_dir).map_err(|e|map_io_error(e, Some(podcast_dir.into())))?;
        Ok(())
    }
}
