use std::io::Error;
use regex::Regex;
use crate::application::services::podcast_episode::service::PodcastEpisodeService;

pub enum FileType {
    Audio,
    Image,
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
) -> String {
    get_suffix_by_url(url).unwrap_or_else(|_| {
        let response = client.head(url).send().unwrap();
        let mime_type = response.headers().get("content-type").unwrap();
        let complete_extension = mime_type.to_str().unwrap();
        if let Some(extension) = complete_extension.split('/').last() {
            let file_extension = extension;
            if complete_extension.contains("audio") || complete_extension.contains("image") {
                if file_extension.contains(';') {
                    let f_ext = file_extension.split(';').next().unwrap().to_string();
                    return f_ext;
                }
                file_extension.to_string()
            } else {
                match file_type {
                    FileType::Audio => ".mp3".to_string(),
                    FileType::Image => ".jpg".to_string(),
                }
            }
        } else {
            match file_type {
                FileType::Audio => ".mp3".to_string(),
                FileType::Image => ".jpg".to_string(),
            }
        }
    })
}

fn get_suffix_by_url(url: &str) -> Result<String, Error> {
    get_url_file_suffix(url)
}
pub fn get_url_file_suffix(url: &str) -> Result<String, Error> {
    let re = Regex::new(r"\.(\w+)(?:\?.*)?$").unwrap();
    let capture = re.captures(url);
    if capture.is_none() {
        return Err(Error::new(std::io::ErrorKind::Other, "No"));
    }
    Ok(capture.unwrap().get(1).unwrap().as_str().to_string())
}
