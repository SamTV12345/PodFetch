use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::constants::inner_constants::{PodcastType, MAIN_ROOM};
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use rand::random;
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::OnceLock;
use futures::executor::block_on;
use socketioxide::SocketIo;
use tokio::sync::{mpsc, oneshot};

type RoomId = String;
pub type ConnId = usize;

pub type Msg = String;

#[derive(Serialize)]
pub struct PodcastEpisodeOfflineAvailableMessage {
    message: String,
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

/// A command received by the [`ChatServer`].
#[derive(Debug)]
enum Command {
    Broadcast {
        room: RoomId,
        msg: Msg,
    },
    Connect {
        conn_tx: mpsc::UnboundedSender<Msg>,
    },

    Disconnect {
        conn: ConnId,
    },
    Message {
        msg: Msg,
        conn: ConnId,
    },
}

static SOCKET_IO_LAYER: OnceLock<SocketIo> = OnceLock::new();

/// Handle and command sender for chat server.
///
/// Reduces boilerplate of setting up response channels in WebSocket handlers.
#[derive(Debug, Clone)]
pub struct ChatServerHandle;

impl ChatServerHandle {

    fn send_broadcast_sync(&self, room_id: RoomId, msg: impl Into<Msg>) {
        block_on(SOCKET_IO_LAYER.get().unwrap().to(&room_id).emit("message", &msg.into()))
            .unwrap();
    }

    pub fn broadcast_podcast_episode_offline_available(
        &self,
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
        self.send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            serde_json::to_string(&PodcastEpisodeOfflineAvailableMessage {
                message: format!("Episode {} is now available offline", podcast_episode.name),
                podcast,
                type_of: PodcastType::AddPodcastEpisode,
                podcast_episode,
            })
            .unwrap(),
        );
    }

    pub fn broadcast_podcast_refreshed(&self, podcast: &Podcast) {
        self.send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            serde_json::to_string(&PodcastRefreshedMessage {
                type_of: PodcastType::RefreshPodcast,
                message: format!("Podcast {} has been refreshed", podcast.name),
                podcast: podcast.clone().into(),
            })
            .unwrap(),
        );
    }

    pub fn broadcast_opml_error(&self, message: String) {
        self.send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            serde_json::to_string(&OpmlErrorMessage::from(message)).unwrap(),
        )
    }

    pub fn broadcast_opml_added(&self, podcast: &Podcast) {
        self.send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            serde_json::to_string(&OpmlAddedMessage::from(podcast.clone())).unwrap(),
        );
    }

    pub fn broadcast_podcast_episode_deleted_locally(&self, podcast_episode: &PodcastEpisode) {
        self.send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            serde_json::to_string(&PodcastEpisodeDeleteMesage {
                podcast_episode: PodcastEpisodeDto::from((
                    podcast_episode.clone(),
                    None::<User>,
                    None::<FavoritePodcastEpisode>,
                )),
                type_of: PodcastType::DeletePodcastEpisode,
                message: "Deleted podcast episode locally".to_string(),
            })
            .unwrap(),
        );
    }

    pub fn broadcast_podcast_downloaded(&self, podcast: Podcast) {
        self.send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            serde_json::to_string(&PodcastAddedMessage::from(podcast)).unwrap(),
        );
    }

    pub fn broadcast_added_podcast_episodes(
        &self,
        podcast: Podcast,
        episodes: Vec<PodcastEpisode>,
    ) {
        self.send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            serde_json::to_string(&PodcastEpisodesAdded::from((podcast, episodes))).unwrap(),
        );
    }
}
