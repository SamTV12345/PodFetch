use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, Clone, IntoParams)]
pub struct OptionalId {
    pub last_podcast_episode: Option<String>,
    pub only_unlistened: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastChapterDto {
    pub id: String,
    pub start_time: i32,
    pub title: String,
    pub end_time: i32,
    pub chapter_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TimelineQueryParams {
    pub favored_only: Option<bool>,
    pub last_timestamp: Option<String>,
    pub not_listened: bool,
    pub favored_episodes: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastEpisodeWithHistory<E, H> {
    pub podcast_episode: E,
    pub podcast_history_item: Option<H>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TimeLinePodcastEpisode {
    pub podcast_episode: crate::podcast_episode_dto::PodcastEpisodeDto,
    pub podcast: crate::podcast::PodcastDto,
    pub history: Option<crate::history::EpisodeDto>,
    pub favorite: Option<TimelineFavorite>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimeLinePodcastItem {
    pub data: Vec<TimeLinePodcastEpisode>,
    pub total_elements: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FavoritePut {
    pub favored: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TimelineFavorite {
    pub favored: bool,
}

impl From<podfetch_domain::favorite::Favorite> for TimelineFavorite {
    fn from(value: podfetch_domain::favorite::Favorite) -> Self {
        Self {
            favored: value.favored,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EpisodeFormatDto {
    pub content: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PodcastEpisodeControllerError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("{0}")]
    Service(E),
}

pub fn get_episode_with_history<E, H, Err, GetEpisode, GetHistory>(
    episode_id: &str,
    username: &str,
    get_episode: GetEpisode,
    get_history: GetHistory,
) -> Result<PodcastEpisodeWithHistory<E, H>, PodcastEpisodeControllerError<Err>>
where
    Err: Display,
    GetEpisode: FnOnce(&str) -> Result<Option<E>, Err>,
    GetHistory: FnOnce(&str, &str) -> Result<Option<H>, Err>,
{
    let podcast_episode = get_episode(episode_id)
        .map_err(PodcastEpisodeControllerError::Service)?
        .ok_or(PodcastEpisodeControllerError::NotFound)?;
    let podcast_history_item =
        get_history(episode_id, username).map_err(PodcastEpisodeControllerError::Service)?;

    Ok(PodcastEpisodeWithHistory {
        podcast_episode,
        podcast_history_item,
    })
}

pub fn get_podcast_episodes_with_history<E, H, Err, FetchEpisodes>(
    podcast_id: &str,
    username: &str,
    last_podcast_episode: Option<String>,
    only_unlistened: Option<bool>,
    fetch_episodes: FetchEpisodes,
) -> Result<Vec<PodcastEpisodeWithHistory<E, H>>, PodcastEpisodeControllerError<Err>>
where
    Err: Display,
    FetchEpisodes:
        FnOnce(i32, Option<String>, Option<bool>, &str) -> Result<Vec<(E, Option<H>)>, Err>,
{
    let parsed_id = podcast_id.parse::<i32>().map_err(|_| {
        PodcastEpisodeControllerError::BadRequest("podcast id must be an integer".to_string())
    })?;

    let episodes = fetch_episodes(parsed_id, last_podcast_episode, only_unlistened, username)
        .map_err(PodcastEpisodeControllerError::Service)?;

    Ok(episodes
        .into_iter()
        .map(
            |(podcast_episode, podcast_history_item)| PodcastEpisodeWithHistory {
                podcast_episode,
                podcast_history_item,
            },
        )
        .collect())
}

pub fn require_privileged<Err: Display>(
    is_privileged: bool,
) -> Result<(), PodcastEpisodeControllerError<Err>> {
    if is_privileged {
        Ok(())
    } else {
        Err(PodcastEpisodeControllerError::Forbidden)
    }
}
