use crate::utils::error::CustomError;
use podfetch_domain::playlist::{Playlist, PlaylistItem, PlaylistRepository};
use podfetch_persistence::db::Database;
use podfetch_persistence::playlist::DieselPlaylistRepository;

pub struct PlaylistRepositoryImpl {
    inner: DieselPlaylistRepository,
}

impl PlaylistRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPlaylistRepository::new(database),
        }
    }
}

impl PlaylistRepository for PlaylistRepositoryImpl {
    type Error = CustomError;

    fn find_by_name(&self, name: &str) -> Result<Option<Playlist>, Self::Error> {
        self.inner.find_by_name(name).map_err(Into::into)
    }

    fn insert_playlist(&self, playlist: Playlist) -> Result<Playlist, Self::Error> {
        self.inner.insert_playlist(playlist).map_err(Into::into)
    }

    fn find_by_id(&self, playlist_id: &str) -> Result<Option<Playlist>, Self::Error> {
        self.inner.find_by_id(playlist_id).map_err(Into::into)
    }

    fn find_by_user_and_id(
        &self,
        playlist_id: &str,
        user_id: i32,
    ) -> Result<Option<Playlist>, Self::Error> {
        self.inner
            .find_by_user_and_id(playlist_id, user_id)
            .map_err(Into::into)
    }

    fn list_by_user(&self, user_id: i32) -> Result<Vec<Playlist>, Self::Error> {
        self.inner.list_by_user(user_id).map_err(Into::into)
    }

    fn update_playlist_name(
        &self,
        playlist_id: &str,
        user_id: i32,
        name: &str,
    ) -> Result<usize, Self::Error> {
        self.inner
            .update_playlist_name(playlist_id, user_id, name)
            .map_err(Into::into)
    }

    fn delete_playlist(&self, playlist_id: &str, user_id: i32) -> Result<usize, Self::Error> {
        self.inner
            .delete_playlist(playlist_id, user_id)
            .map_err(Into::into)
    }

    fn insert_playlist_item(&self, item: PlaylistItem) -> Result<PlaylistItem, Self::Error> {
        self.inner.insert_playlist_item(item).map_err(Into::into)
    }

    fn list_items_by_playlist_id(
        &self,
        playlist_id: &str,
    ) -> Result<Vec<PlaylistItem>, Self::Error> {
        self.inner
            .list_items_by_playlist_id(playlist_id)
            .map_err(Into::into)
    }

    fn delete_items_by_playlist_id(&self, playlist_id: &str) -> Result<usize, Self::Error> {
        self.inner
            .delete_items_by_playlist_id(playlist_id)
            .map_err(Into::into)
    }

    fn delete_playlist_item(
        &self,
        playlist_id: &str,
        episode_id: i32,
    ) -> Result<usize, Self::Error> {
        self.inner
            .delete_playlist_item(playlist_id, episode_id)
            .map_err(Into::into)
    }

    fn delete_items_by_episode_id(&self, episode_id: i32) -> Result<usize, Self::Error> {
        self.inner
            .delete_items_by_episode_id(episode_id)
            .map_err(Into::into)
    }
}
