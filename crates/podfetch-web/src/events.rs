use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PodcastType {
    AddPodcast,
    AddPodcastEpisode,
    AddPodcastEpisodes,
    DeletePodcastEpisode,
    RefreshPodcast,
    OpmlAdded,
    OpmlErrored,
}

#[derive(Serialize)]
pub struct PodcastEpisodeOfflineAvailableMessage<P, E> {
    pub podcast: P,
    pub type_of: PodcastType,
    pub podcast_episode: E,
}

#[derive(Serialize)]
pub struct PodcastRefreshedMessage<P> {
    pub type_of: PodcastType,
    pub message: String,
    pub podcast: P,
}

#[derive(Serialize)]
pub struct OpmlErrorMessage {
    pub type_of: PodcastType,
    pub message: String,
}

#[derive(Serialize)]
pub struct PodcastEpisodeDeleteMessage<E> {
    pub podcast_episode: E,
    pub type_of: PodcastType,
    pub message: String,
}

#[derive(Serialize)]
pub struct PodcastEpisodesAdded<P, E> {
    pub type_of: PodcastType,
    pub message: String,
    pub podcast: P,
    pub podcast_episodes: Vec<E>,
}

#[derive(Serialize)]
pub struct OpmlAddedMessage<P> {
    pub type_of: PodcastType,
    pub message: String,
    pub podcast: P,
}

#[derive(Serialize)]
pub struct PodcastAddedMessage<P> {
    pub type_of: PodcastType,
    pub message: String,
    pub podcast: P,
}
