use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedPostModel {
    pub podcast_episode_id: String,
    pub time: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedEpisodeModelWithPodcastEpisode<P, D, E> {
    pub podcast_episode: P,
    pub podcast: D,
    pub episode: E,
}

pub trait WatchtimeApplicationService {
    type Error;
    type EpisodeDto;
    type LastWatchedItem;

    fn log_watchtime(
        &self,
        username: String,
        request: PodcastWatchedPostModel,
    ) -> Result<(), Self::Error>;
    fn get_last_watched(&self, username: &str) -> Result<Vec<Self::LastWatchedItem>, Self::Error>;
    fn get_watchtime(
        &self,
        episode_id: &str,
        username: &str,
    ) -> Result<Option<Self::EpisodeDto>, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum WatchtimeControllerError<E: Display> {
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Service(E),
}

pub fn log_watchtime<S>(
    service: &S,
    username: String,
    request: PodcastWatchedPostModel,
) -> Result<(), WatchtimeControllerError<S::Error>>
where
    S: WatchtimeApplicationService,
    S::Error: Display,
{
    service
        .log_watchtime(username, request)
        .map_err(WatchtimeControllerError::Service)
}

pub fn get_last_watched<S>(
    service: &S,
    username: &str,
) -> Result<Vec<S::LastWatchedItem>, WatchtimeControllerError<S::Error>>
where
    S: WatchtimeApplicationService,
    S::Error: Display,
{
    service
        .get_last_watched(username)
        .map_err(WatchtimeControllerError::Service)
}

pub fn get_watchtime<S>(
    service: &S,
    episode_id: &str,
    username: &str,
) -> Result<S::EpisodeDto, WatchtimeControllerError<S::Error>>
where
    S: WatchtimeApplicationService,
    S::Error: Display,
{
    service
        .get_watchtime(episode_id, username)
        .map_err(WatchtimeControllerError::Service)?
        .ok_or(WatchtimeControllerError::NotFound)
}
