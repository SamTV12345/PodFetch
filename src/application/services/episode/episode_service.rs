use crate::adapters::persistence::repositories::podcast::episode::EpisodeRepositoryImpl;
use crate::utils::error::CustomError;

pub struct EpisodeService;


impl EpisodeService {
    pub fn log_watchtime(podcast_episode_id: &str, time: i32, username: &str) -> Result<(),
        CustomError> {
        EpisodeRepositoryImpl::log_watchtime(
            podcast_episode_id,
            time,
            username,
        )
    }

    pub fn delete_watchtime(podcast_episode: i32) -> Result<(), CustomError> {
        EpisodeRepositoryImpl::delete_watchtime(podcast_episode)
    }
}