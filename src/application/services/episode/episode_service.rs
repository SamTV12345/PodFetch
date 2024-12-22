use chrono::NaiveDateTime;
use crate::adapters::persistence::repositories::podcast::episode::EpisodeRepositoryImpl;
use crate::domain::models::episode::episode::Episode;
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

    pub fn delete_by_username(username: &str) -> Result<(), CustomError> {
        EpisodeRepositoryImpl::delete_by_username(username)
    }

    pub fn delete_watchtime(podcast_episode: i32) -> Result<(), CustomError> {
        EpisodeRepositoryImpl::delete_watchtime(podcast_episode)
    }

    pub fn get_actions_by_username(username: &str,
                                   since_date: Option<NaiveDateTime>, opt_device: Option<String>,
                                   _opt_aggregate: Option<String>, opt_podcast: Option<String>) -> Result<Vec<String>, CustomError> {
        EpisodeRepositoryImpl::get_actions_by_username(username, since_date, opt_device, _opt_aggregate, opt_podcast)
    }

    pub fn insert_episode(episode: Episode) -> Result<Episode, CustomError> {
        EpisodeRepositoryImpl::insert_episode(episode)
    }
}