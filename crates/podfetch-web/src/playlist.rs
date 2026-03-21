use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PlaylistItem {
    pub episode: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PlaylistDtoPost {
    pub name: String,
    pub items: Vec<PlaylistItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PlaylistDto<T> {
    pub id: String,
    pub name: String,
    pub items: Vec<T>,
}

pub trait PlaylistApplicationService {
    type Error;
    type PlaylistDto;

    fn add_playlist(
        &self,
        user_id: i32,
        username: String,
        playlist: PlaylistDtoPost,
    ) -> Result<Self::PlaylistDto, Self::Error>;
    fn update_playlist(
        &self,
        user_id: i32,
        username: String,
        playlist_id: String,
        playlist: PlaylistDtoPost,
    ) -> Result<Self::PlaylistDto, Self::Error>;
    fn get_all_playlists(
        &self,
        user_id: i32,
        username: String,
    ) -> Result<Vec<Self::PlaylistDto>, Self::Error>;
    fn get_playlist_by_id(
        &self,
        user_id: i32,
        username: String,
        playlist_id: String,
    ) -> Result<Self::PlaylistDto, Self::Error>;
    fn delete_playlist_by_id(&self, user_id: i32, playlist_id: String) -> Result<(), Self::Error>;
    fn delete_playlist_item(
        &self,
        user_id: i32,
        playlist_id: String,
        episode_id: i32,
    ) -> Result<(), Self::Error>;
}

pub fn add_playlist<S>(
    service: &S,
    user_id: i32,
    username: String,
    playlist: PlaylistDtoPost,
) -> Result<S::PlaylistDto, S::Error>
where
    S: PlaylistApplicationService,
{
    service.add_playlist(user_id, username, playlist)
}

pub fn update_playlist<S>(
    service: &S,
    user_id: i32,
    username: String,
    playlist_id: String,
    playlist: PlaylistDtoPost,
) -> Result<S::PlaylistDto, S::Error>
where
    S: PlaylistApplicationService,
{
    service.update_playlist(user_id, username, playlist_id, playlist)
}

pub fn get_all_playlists<S>(
    service: &S,
    user_id: i32,
    username: String,
) -> Result<Vec<S::PlaylistDto>, S::Error>
where
    S: PlaylistApplicationService,
{
    service.get_all_playlists(user_id, username)
}

pub fn get_playlist_by_id<S>(
    service: &S,
    user_id: i32,
    username: String,
    playlist_id: String,
) -> Result<S::PlaylistDto, S::Error>
where
    S: PlaylistApplicationService,
{
    service.get_playlist_by_id(user_id, username, playlist_id)
}

pub fn delete_playlist_by_id<S>(
    service: &S,
    user_id: i32,
    playlist_id: String,
) -> Result<(), S::Error>
where
    S: PlaylistApplicationService,
{
    service.delete_playlist_by_id(user_id, playlist_id)
}

pub fn delete_playlist_item<S>(
    service: &S,
    user_id: i32,
    playlist_id: String,
    episode_id: i32,
) -> Result<(), S::Error>
where
    S: PlaylistApplicationService,
{
    service.delete_playlist_item(user_id, playlist_id, episode_id)
}
