use crate::service::podcast_episode_service::PodcastEpisodeService;
use file_format::FileFormat;
use std::fmt::Display;
use std::io::Error;

pub enum FileType {
    Audio,
    Image,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Audio => write!(f, "mp3"),
            FileType::Image => write!(f, "jpg"),
        }
    }
}

pub enum DetermineFileExtensionReturn {
    FileExtension(String, Vec<u8>),
    String(String),
}

//
// File extension determination
// It determines the current file extension
// 1. By the file name
// 2. By the mime type
// 2. By the file name
// 3. By a set of extensions
pub fn determine_file_extension(
    url: &str,
    client: &reqwest::blocking::Client,
    file_type: FileType,
) -> DetermineFileExtensionReturn {
    get_suffix_by_url(url)
        .map(DetermineFileExtensionReturn::String)
        .unwrap_or_else(|_| {
            let response = match client.get(url).send() {
                Ok(response) => response,
                Err(_) => {
                    return DetermineFileExtensionReturn::String(file_type.to_string());
                }
            };

            let bytes = match response.bytes() {
                Ok(bytes) => bytes,
                Err(_) => {
                    return DetermineFileExtensionReturn::String(file_type.to_string());
                }
            };

            let file_extension = FileFormat::from(bytes.as_ref()).to_string();
            DetermineFileExtensionReturn::FileExtension(
                file_extension.to_string(),
                bytes.as_ref().to_vec(),
            )
        })
}

fn get_suffix_by_url(url: &str) -> Result<String, Error> {
    PodcastEpisodeService::get_url_file_suffix(url)
}
