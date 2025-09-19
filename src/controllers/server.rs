use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::constants::inner_constants::{MAIN_ROOM, PodcastType};
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use futures::executor::block_on;
use serde::Serialize;
use socketioxide::SocketIo;
use std::sync::OnceLock;

type RoomId = String;

#[derive(Serialize)]
pub struct PodcastEpisodeOfflineAvailableMessage {
    podcast: PodcastDto,
    type_of: PodcastType,
    podcast_episode: PodcastEpisodeDto,
}

#[derive(Serialize)]
pub struct PodcastRefreshedMessage {
    type_of: PodcastType,
    message: String,
    podcast: PodcastDto,
}

#[derive(Serialize)]
pub struct OpmlErrorMessage {
    type_of: PodcastType,
    message: String,
}

#[derive(Serialize)]
pub struct PodcastEpisodeDeleteMesage {
    podcast_episode: PodcastEpisodeDto,
    type_of: PodcastType,
    message: String,
}

#[derive(Serialize)]
pub struct PodcastEpisodesAdded {
    type_of: PodcastType,
    message: String,
    podcast: PodcastDto,
    podcast_episodes: Vec<PodcastEpisodeDto>,
}

#[derive(Serialize)]
pub struct OpmlAddedMessage {
    type_of: PodcastType,
    message: String,
    podcast: PodcastDto,
}

#[derive(Serialize)]
pub struct PodcastAddedMessage {
    type_of: PodcastType,
    message: String,
    podcast: PodcastDto,
}

impl From<Podcast> for OpmlAddedMessage {
    fn from(podcast: Podcast) -> Self {
        OpmlAddedMessage {
            type_of: PodcastType::OpmlAdded,
            message: format!("Podcast {} has been added", podcast.name),
            podcast: podcast.into(),
        }
    }
}

impl From<(Podcast, Vec<PodcastEpisode>)> for PodcastEpisodesAdded {
    fn from(value: (Podcast, Vec<PodcastEpisode>)) -> Self {
        Self {
            podcast_episodes: value
                .1
                .into_iter()
                .map(|episode| (episode, None::<User>, None::<FavoritePodcastEpisode>).into())
                .collect(),
            podcast: value.0.clone().into(),
            type_of: PodcastType::AddPodcastEpisodes,
            message: format!("Added podcast episodes: {}", &value.0.name),
        }
    }
}

impl From<String> for OpmlErrorMessage {
    fn from(message: String) -> Self {
        OpmlErrorMessage {
            type_of: PodcastType::OpmlErrored,
            message,
        }
    }
}

impl From<Podcast> for PodcastAddedMessage {
    fn from(value: Podcast) -> Self {
        PodcastAddedMessage {
            type_of: PodcastType::AddPodcast,
            message: format!("Podcast {} has been added", value.name),
            podcast: value.into(),
        }
    }
}

pub static SOCKET_IO_LAYER: OnceLock<SocketIo> = OnceLock::new();

/// Handle and command sender for chat server.
///
/// Reduces boilerplate of setting up response channels in WebSocket handlers.
#[derive(Debug, Clone)]
pub struct ChatServerHandle;

impl ChatServerHandle {
    fn send_broadcast_sync<T>(room_id: RoomId, msg: &T, event: impl AsRef<str>)
    where
        T: ?Sized + Serialize,
    {
        let room_id = "/".to_owned() + room_id.as_str();
        block_on(
            SOCKET_IO_LAYER
                .get()
                .unwrap()
                .of(room_id)
                .unwrap()
                .emit(event, &msg),
        )
        .unwrap();
    }

    pub fn broadcast_podcast_episode_offline_available(
        podcast_episode: &PodcastEpisode,
        podcast: &Podcast,
    ) {
        let podcast_episode: PodcastEpisodeDto = (
            podcast_episode.clone(),
            None::<User>,
            None::<FavoritePodcastEpisode>,
        )
            .clone()
            .into();
        let podcast = podcast.clone().into();
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &PodcastEpisodeOfflineAvailableMessage {
                podcast,
                type_of: PodcastType::AddPodcastEpisode,
                podcast_episode,
            },
            "offlineAvailable",
        );
    }

    pub fn broadcast_podcast_refreshed(podcast: &Podcast) {
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &PodcastRefreshedMessage {
                type_of: PodcastType::RefreshPodcast,
                message: format!("Podcast {} has been refreshed", podcast.name),
                podcast: podcast.clone().into(),
            },
            "refreshedPodcast",
        );
    }

    pub fn broadcast_opml_error(message: String) {
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &OpmlErrorMessage::from(message),
            "opmlError",
        )
    }

    pub fn broadcast_opml_added(podcast: &Podcast) {
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &OpmlAddedMessage::from(podcast.clone()),
            "opmlAdded",
        );
    }

    pub fn broadcast_podcast_episode_deleted_locally(podcast_episode: &PodcastEpisode) {
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &PodcastEpisodeDeleteMesage {
                podcast_episode: PodcastEpisodeDto::from((
                    podcast_episode.clone(),
                    None::<User>,
                    None::<FavoritePodcastEpisode>,
                )),
                type_of: PodcastType::DeletePodcastEpisode,
                message: "Deleted podcast episode locally".to_string(),
            },
            "deletedPodcastEpisodeLocally",
        );
    }

    pub fn broadcast_podcast_downloaded(podcast: Podcast) {
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &PodcastAddedMessage::from(podcast),
            "addedPodcast",
        );
    }

    pub fn broadcast_added_podcast_episodes(podcast: &Podcast, episodes: Vec<PodcastEpisode>) {
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &PodcastEpisodesAdded::from((podcast.clone(), episodes)),
            "addedEpisodes",
        );
    }
}
