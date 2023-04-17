use crate::service::file_service::prepare_podcast_title_to_directory;

pub struct PathService {}

impl PathService {
    pub fn get_podcast_episode_path(directory: &str, episode_id: &str, suffix: &str) -> String {
        return format!("{}/{}/podcast.{}", directory, prepare_podcast_title_to_directory
            (episode_id), suffix);
    }

    pub fn get_image_path(directory: &str, episode_id: &str, suffix: &str) -> String {
        return format!("{}/{}/image.{}", directory, prepare_podcast_title_to_directory(episode_id), suffix);
    }

    pub fn get_image_podcast_path(directory: &str, suffix: &str) -> String {
        return format!("podcasts/{}/image.{}", prepare_podcast_title_to_directory(directory),
            suffix);
    }
}
