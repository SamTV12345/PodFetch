use crate::adapters::api::ws::messages::BroadcastMessage;
use crate::adapters::api::ws::server::{ChatServer, ChatServerHandle};
use crate::constants::inner_constants::{PodcastType, MAIN_ROOM};
use crate::domain::models::podcast::episode::PodcastEpisode;
use std::sync::OnceLock;

pub static CHAT_SERVER_WITH_HANDLE: OnceLock<(ChatServer, ChatServerHandle)> = OnceLock::new();
pub static CHAT_SERVER: OnceLock<ChatServer> = OnceLock::new();


pub async fn notify_delete_podcast_episode_locally(chat_handle: &ChatServerHandle,
                                                   deleted_podcast_episode: PodcastEpisode)  {
    chat_handle.send_broadcast(MAIN_ROOM.parse().unwrap(),serde_json::to_string(&BroadcastMessage {
        podcast_episode: Some(deleted_podcast_episode),
        podcast_episodes: None,
        type_of: PodcastType::DeletePodcastEpisode,
        podcast: None,
        message: "Deleted podcast episode locally".to_string(),
    }).unwrap()).await;
}

pub async fn notify_downloaded_podcast_episode_locally(chat_handle: &ChatServerHandle,
                                                       added_podcast_episode: PodcastEpisode) {
    chat_handle.send_broadcast(MAIN_ROOM.parse().unwrap(),serde_json::to_string(&BroadcastMessage {
        podcast_episode: Some(added_podcast_episode),
        podcast_episodes: None,
        type_of: PodcastType::AddPodcastEpisode,
        podcast: None,
        message: "Downloaded podcast episode locally".to_string(),
    }).unwrap()).await;
}