use podfetch_cast::{CastSessionId, CastStatus};
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

/// Reason a cast session terminated, for the `cast:ended` event.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CastEndedReason {
    Stopped,
    Finished,
    DeviceGone,
    Error,
}

/// Live status update for an in-flight cast session.
#[derive(Serialize)]
pub struct CastStatusMessage {
    pub status: CastStatus,
}

/// Sent once when a cast session ends. The UI uses this to drop the
/// remote-control overlay and resume normal local-player behaviour.
#[derive(Serialize)]
pub struct CastEndedMessage {
    pub session_id: CastSessionId,
    pub reason: CastEndedReason,
}
