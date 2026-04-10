use crate::events::{
    OpmlAddedMessage, OpmlErrorMessage, PodcastAddedMessage, PodcastEpisodeDeleteMessage,
    PodcastEpisodeOfflineAvailableMessage, PodcastEpisodesAdded, PodcastRefreshedMessage,
    PodcastType,
};
use crate::podcast::PodcastDto;
use crate::podcast::map_podcast_to_dto;
use crate::podcast_episode_dto::PodcastEpisodeDto;
use common_infrastructure::runtime::MAIN_ROOM;
use futures::executor::block_on;
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_domain::user::User;
use podfetch_persistence::podcast::PodcastEntity as Podcast;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use serde::Serialize;
use socketioxide::SocketIo;
use std::sync::OnceLock;

type RoomId = String;

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
        let socket = match SOCKET_IO_LAYER.get() {
            Some(socket) => socket,
            None => {
                log::warn!(
                    "Skipping websocket broadcast for event '{}' because socket layer is not initialized",
                    event.as_ref()
                );
                return;
            }
        };
        let namespace = match socket.of(room_id) {
            Some(namespace) => namespace,
            None => {
                log::warn!(
                    "Skipping websocket broadcast for event '{}' because namespace is unavailable",
                    event.as_ref()
                );
                return;
            }
        };
        if let Err(err) = block_on(namespace.emit(event.as_ref(), msg)) {
            log::warn!(
                "Websocket broadcast failed for event '{}': {}",
                event.as_ref(),
                err
            );
        }
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
        let podcast: PodcastDto = map_podcast_to_dto(podcast.clone().into());
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &PodcastEpisodeOfflineAvailableMessage::<PodcastDto, PodcastEpisodeDto> {
                podcast,
                type_of: PodcastType::AddPodcastEpisode,
                podcast_episode,
            },
            "offlineAvailable",
        );
    }

    pub fn broadcast_podcast_refreshed(podcast: &Podcast) {
        let podcast: PodcastDto = map_podcast_to_dto(podcast.clone().into());
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &PodcastRefreshedMessage::<PodcastDto> {
                type_of: PodcastType::RefreshPodcast,
                message: format!("Podcast {} has been refreshed", podcast.name),
                podcast,
            },
            "refreshedPodcast",
        );
    }

    pub fn broadcast_opml_error(message: String) {
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &OpmlErrorMessage {
                type_of: PodcastType::OpmlErrored,
                message,
            },
            "opmlError",
        )
    }

    pub fn broadcast_opml_added(podcast: &Podcast) {
        let podcast: PodcastDto = map_podcast_to_dto(podcast.clone().into());
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &OpmlAddedMessage::<PodcastDto> {
                type_of: PodcastType::OpmlAdded,
                message: format!("Podcast {} has been added", podcast.name),
                podcast,
            },
            "opmlAdded",
        );
    }

    pub fn broadcast_podcast_episode_deleted_locally(podcast_episode: &PodcastEpisode) {
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &PodcastEpisodeDeleteMessage {
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
        let podcast_name = podcast.name.clone();
        let podcast: PodcastDto = map_podcast_to_dto(podcast.into());
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &PodcastAddedMessage::<PodcastDto> {
                type_of: PodcastType::AddPodcast,
                message: format!("Podcast {} has been added", podcast_name),
                podcast,
            },
            "addedPodcast",
        );
    }

    pub fn broadcast_added_podcast_episodes(podcast: &Podcast, episodes: Vec<PodcastEpisode>) {
        let podcast: PodcastDto = map_podcast_to_dto(podcast.clone().into());
        let podcast_name = podcast.name.clone();
        let podcast_episodes: Vec<PodcastEpisodeDto> = episodes
            .into_iter()
            .map(|episode| (episode, None::<User>, None::<FavoritePodcastEpisode>).into())
            .collect();
        Self::send_broadcast_sync(
            MAIN_ROOM.parse().unwrap(),
            &PodcastEpisodesAdded::<PodcastDto, PodcastEpisodeDto> {
                podcast_episodes,
                podcast,
                type_of: PodcastType::AddPodcastEpisodes,
                message: format!("Added podcast episodes: {}", podcast_name),
            },
            "addedEpisodes",
        );
    }
}
