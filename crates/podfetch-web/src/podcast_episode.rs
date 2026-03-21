use serde::{Deserialize, Serialize};
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
}

#[derive(Debug, Serialize, Deserialize, Clone, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TimelineQueryParams {
    pub favored_only: bool,
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
pub struct TimeLinePodcastEpisode<E, P, H, F> {
    pub podcast_episode: E,
    pub podcast: P,
    pub history: Option<H>,
    pub favorite: Option<F>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimeLinePodcastItem<T> {
    pub data: Vec<T>,
    pub total_elements: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FavoritePut {
    pub favored: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EpisodeFormatDto {
    pub content: String,
}
