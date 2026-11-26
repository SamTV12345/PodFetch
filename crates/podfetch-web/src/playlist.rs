use podfetch_identity::api::Principal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PlaylistItem {
    pub episode: String,
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
        user: Principal,
        api_key: Option<&str>,
        playlist: PlaylistDtoPost,
    ) -> Result<Self::PlaylistDto, Self::Error>;
    fn update_playlist(
        &self,
        user: Principal,
        api_key: Option<&str>,
        playlist_id: String,
        playlist: PlaylistDtoPost,
    ) -> Result<Self::PlaylistDto, Self::Error>;
    fn get_all_playlists(
        &self,
        user: Principal,
        api_key: Option<&str>,
    ) -> Result<Vec<Self::PlaylistDto>, Self::Error>;
    fn get_playlist_by_id(
        &self,
        user: Principal,
        api_key: Option<&str>,
        playlist_id: String,
    ) -> Result<Self::PlaylistDto, Self::Error>;
    fn delete_playlist_by_id(&self, user_id: Uuid, playlist_id: String) -> Result<(), Self::Error>;
    fn delete_playlist_item(
        &self,
        user_id: Uuid,
        playlist_id: String,
        episode_id: Uuid,
    ) -> Result<(), Self::Error>;
}

pub fn add_playlist<S>(
    service: &S,
    user: Principal,
    api_key: Option<&str>,
    playlist: PlaylistDtoPost,
) -> Result<S::PlaylistDto, S::Error>
where
    S: PlaylistApplicationService,
{
    service.add_playlist(user, api_key, playlist)
}

pub fn update_playlist<S>(
    service: &S,
    user: Principal,
    api_key: Option<&str>,
    playlist_id: String,
    playlist: PlaylistDtoPost,
) -> Result<S::PlaylistDto, S::Error>
where
    S: PlaylistApplicationService,
{
    service.update_playlist(user, api_key, playlist_id, playlist)
}

pub fn get_all_playlists<S>(
    service: &S,
    user: Principal,
    api_key: Option<&str>,
) -> Result<Vec<S::PlaylistDto>, S::Error>
where
    S: PlaylistApplicationService,
{
    service.get_all_playlists(user, api_key)
}

pub fn get_playlist_by_id<S>(
    service: &S,
    user: Principal,
    api_key: Option<&str>,
    playlist_id: String,
) -> Result<S::PlaylistDto, S::Error>
where
    S: PlaylistApplicationService,
{
    service.get_playlist_by_id(user, api_key, playlist_id)
}

pub fn delete_playlist_by_id<S>(
    service: &S,
    user_id: Uuid,
    playlist_id: String,
) -> Result<(), S::Error>
where
    S: PlaylistApplicationService,
{
    service.delete_playlist_by_id(user_id, playlist_id)
}

pub fn delete_playlist_item<S>(
    service: &S,
    user_id: Uuid,
    playlist_id: String,
    episode_id: Uuid,
) -> Result<(), S::Error>
where
    S: PlaylistApplicationService,
{
    service.delete_playlist_item(user_id, playlist_id, episode_id)
}
