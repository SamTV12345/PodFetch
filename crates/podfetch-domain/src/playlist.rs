use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub user_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PlaylistItem {
    pub playlist_id: String,
    pub episode: i32,
    pub position: i32,
}

pub trait PlaylistRepository: Send + Sync {
    type Error;

    fn find_by_name(&self, name: &str) -> Result<Option<Playlist>, Self::Error>;
    fn insert_playlist(&self, playlist: Playlist) -> Result<Playlist, Self::Error>;
    fn find_by_id(&self, playlist_id: &str) -> Result<Option<Playlist>, Self::Error>;
    fn find_by_user_and_id(
        &self,
        playlist_id: &str,
        user_id: i32,
    ) -> Result<Option<Playlist>, Self::Error>;
    fn list_by_user(&self, user_id: i32) -> Result<Vec<Playlist>, Self::Error>;
    fn update_playlist_name(
        &self,
        playlist_id: &str,
        user_id: i32,
        name: &str,
    ) -> Result<usize, Self::Error>;
    fn delete_playlist(&self, playlist_id: &str, user_id: i32) -> Result<usize, Self::Error>;
    fn insert_playlist_item(&self, item: PlaylistItem) -> Result<PlaylistItem, Self::Error>;
    fn list_items_by_playlist_id(
        &self,
        playlist_id: &str,
    ) -> Result<Vec<PlaylistItem>, Self::Error>;
    fn delete_items_by_playlist_id(&self, playlist_id: &str) -> Result<usize, Self::Error>;
    fn delete_playlist_item(
        &self,
        playlist_id: &str,
        episode_id: i32,
    ) -> Result<usize, Self::Error>;
    fn delete_items_by_episode_id(&self, episode_id: i32) -> Result<usize, Self::Error>;
}
