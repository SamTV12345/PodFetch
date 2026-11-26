use crate::events::{
    CastEndedMessage, CastEndedReason, CastStatusMessage, OpmlAddedMessage, OpmlErrorMessage,
    PodcastAddedMessage, PodcastEpisodeDeleteMessage, PodcastEpisodeOfflineAvailableMessage,
    PodcastEpisodesAdded, PodcastRefreshedMessage, PodcastType, TranscriptionStatusMessage,
};
use crate::podcast::PodcastDto;
use crate::podcast::map_podcast_to_dto;
use crate::podcast_episode_dto::PodcastEpisodeDto;
use common_infrastructure::runtime::MAIN_ROOM;
use futures::executor::block_on;
use podfetch_cast::{CastSessionId, CastStatus};
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_identity::domain::user::User;
use podfetch_persistence::podcast::PodcastEntity as Podcast;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use serde::Serialize;
use socketioxide::SocketIo;
use std::sync::OnceLock;

pub static SOCKET_IO_LAYER: OnceLock<SocketIo> = OnceLock::new();

/// Handle and command sender for chat server.
///
/// Reduces boilerplate of setting up response channels in WebSocket handlers.
#[derive(Debug, Clone)]
pub struct ChatServerHandle;

impl ChatServerHandle {
    fn send_broadcast_sync<T>(room_id: &str, msg: &T, event: impl AsRef<str>)
    where
        T: ?Sized + Serialize,
    {
        let room_id = format!("/{room_id}");
        let Some(socket) = SOCKET_IO_LAYER.get() else {
            tracing::warn!(
                "Skipping websocket broadcast for event '{}' because socket layer is not initialized",
                event.as_ref()
            );
            return;
        };
        let Some(namespace) = socket.of(room_id) else {
            tracing::warn!(
                "Skipping websocket broadcast for event '{}' because namespace is unavailable",
                event.as_ref()
            );
            return;
        };
        if let Err(err) = block_on(namespace.emit(event.as_ref(), msg)) {
            tracing::warn!(
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
        let podcast_episode: PodcastEpisodeDto = PodcastEpisodeDto::from_episode_with_user(
            podcast_episode.clone(),
            None::<User>,
            None::<FavoritePodcastEpisode>,
            "",
        );
        let podcast: PodcastDto = map_podcast_to_dto(podcast.clone().into(), "");
        Self::send_broadcast_sync(
            MAIN_ROOM,
            &PodcastEpisodeOfflineAvailableMessage::<PodcastDto, PodcastEpisodeDto> {
                podcast,
                type_of: PodcastType::AddPodcastEpisode,
                podcast_episode,
            },
            "offlineAvailable",
        );
    }

    pub fn broadcast_podcast_refreshed(podcast: &Podcast) {
        let podcast: PodcastDto = map_podcast_to_dto(podcast.clone().into(), "");
        Self::send_broadcast_sync(
            MAIN_ROOM,
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
            MAIN_ROOM,
            &OpmlErrorMessage {
                type_of: PodcastType::OpmlErrored,
                message,
            },
            "opmlError",
        );
    }

    pub fn broadcast_opml_added(podcast: &Podcast) {
        let podcast: PodcastDto = map_podcast_to_dto(podcast.clone().into(), "");
        Self::send_broadcast_sync(
            MAIN_ROOM,
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
            MAIN_ROOM,
            &PodcastEpisodeDeleteMessage {
                podcast_episode: PodcastEpisodeDto::from_episode_with_user(
                    podcast_episode.clone(),
                    None::<User>,
                    None::<FavoritePodcastEpisode>,
                    "",
                ),
                type_of: PodcastType::DeletePodcastEpisode,
                message: "Deleted podcast episode locally".to_string(),
            },
            "deletedPodcastEpisodeLocally",
        );
    }

    pub fn broadcast_podcast_downloaded(podcast: Podcast) {
        let podcast_name = podcast.name.clone();
        let podcast: PodcastDto = map_podcast_to_dto(podcast.into(), "");
        Self::send_broadcast_sync(
            MAIN_ROOM,
            &PodcastAddedMessage::<PodcastDto> {
                type_of: PodcastType::AddPodcast,
                message: format!("Podcast {podcast_name} has been added"),
                podcast,
            },
            "addedPodcast",
        );
    }

    /// Push a single cast status snapshot to subscribers. The session owner
    /// filters by `session.session_id` on the client. The current namespace
    /// model does not split by user, so we broadcast to MAIN_ROOM and rely
    /// on the orchestrator to only emit for sessions it owns.
    pub fn broadcast_cast_status(status: CastStatus) {
        Self::send_broadcast_sync(MAIN_ROOM, &CastStatusMessage { status }, "cast:status");
    }

    pub fn broadcast_cast_ended(session_id: CastSessionId, reason: CastEndedReason) {
        Self::send_broadcast_sync(
            MAIN_ROOM,
            &CastEndedMessage { session_id, reason },
            "cast:ended",
        );
    }

    pub fn broadcast_added_podcast_episodes(podcast: &Podcast, episodes: Vec<PodcastEpisode>) {
        let podcast: PodcastDto = map_podcast_to_dto(podcast.clone().into(), "");
        let podcast_name = podcast.name.clone();
        let podcast_episodes: Vec<PodcastEpisodeDto> = episodes
            .into_iter()
            .map(|episode| {
                PodcastEpisodeDto::from_episode_with_user(
                    episode,
                    None::<User>,
                    None::<FavoritePodcastEpisode>,
                    "",
                )
            })
            .collect();
        Self::send_broadcast_sync(
            MAIN_ROOM,
            &PodcastEpisodesAdded::<PodcastDto, PodcastEpisodeDto> {
                podcast_episodes,
                podcast,
                type_of: PodcastType::AddPodcastEpisodes,
                message: format!("Added podcast episodes: {podcast_name}"),
            },
            "addedEpisodes",
        );
    }

    /// Broadcasts a transcription job's status change (`pending` after a
    /// retryable failure, `running`, `done`, or `failed`) so the UI can show
    /// live progress for an in-flight generated transcript.
    pub fn broadcast_transcription_status(episode_id: &str, status: &str, error: Option<&str>) {
        Self::send_broadcast_sync(
            MAIN_ROOM,
            &TranscriptionStatusMessage {
                episode_id: episode_id.to_string(),
                status: status.to_string(),
                error: error.map(ToString::to_string),
            },
            "transcriptionStatus",
        );
    }
}
