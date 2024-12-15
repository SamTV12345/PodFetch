use std::path::PathBuf;
use crate::adapters::filesystem::update_podcast::UpdatePodcast;
use crate::adapters::persistence::repositories::podcast::podcast::PodcastRepositoryImpl;
use crate::application::services::episode::episode_service::EpisodeService;
use crate::application::services::podcast_episode::service::PodcastEpisodeService;
use crate::utils::error::CustomError;

pub struct PodcastService;


impl PodcastService {
    pub fn delete_podcast(podcast_id: i32, delete_files: bool) -> Result<(), CustomError> {
        let found_podcast = PodcastRepositoryImpl::get_podcast(podcast_id)?;
        if delete_files {
            UpdatePodcast::delete_podcast_files(&PathBuf::from(found_podcast.directory_name))?;
        }
        EpisodeService::delete_watchtime(podcast_id)?;
        PodcastEpisodeService::delete_episodes_of_podcast(podcast_id)?;

        Ok(())
    }
}