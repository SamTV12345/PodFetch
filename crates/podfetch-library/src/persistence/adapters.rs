use common_infrastructure::db::Database;
use common_infrastructure::error::CustomError;
use uuid::Uuid;

// ── Filter ──────────────────────────────────────────────────────────────────

use crate::domain::filter::{Filter, FilterRepository};
use crate::persistence::filter::DieselFilterRepository;

pub struct FilterRepositoryImpl {
    inner: DieselFilterRepository,
}

impl FilterRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselFilterRepository::new(database),
        }
    }
}

impl FilterRepository for FilterRepositoryImpl {
    type Error = CustomError;

    fn get_by_user_id(&self, user_id: Uuid) -> Result<Option<Filter>, Self::Error> {
        self.inner.get_by_user_id(user_id).map_err(Into::into)
    }

    fn save(&self, filter: Filter) -> Result<(), Self::Error> {
        self.inner.save(filter).map_err(Into::into)
    }

    fn save_timeline_decision(&self, user_id: Uuid, only_favored: bool) -> Result<(), Self::Error> {
        self.inner
            .save_timeline_decision(user_id, only_favored)
            .map_err(Into::into)
    }
}

// ── FavoritePodcastEpisode ──────────────────────────────────────────────────

use crate::domain::favorite_podcast_episode::{
    FavoritePodcastEpisode, FavoritePodcastEpisodeRepository,
};
use crate::persistence::favorite_podcast_episode::DieselFavoritePodcastEpisodeRepository;

pub struct FavoritePodcastEpisodeRepositoryImpl {
    inner: DieselFavoritePodcastEpisodeRepository,
}

impl FavoritePodcastEpisodeRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselFavoritePodcastEpisodeRepository::new(database),
        }
    }
}

impl FavoritePodcastEpisodeRepository for FavoritePodcastEpisodeRepositoryImpl {
    type Error = CustomError;

    fn get_by_user_id_and_episode_id(
        &self,
        user_id: Uuid,
        episode_id: Uuid,
    ) -> Result<Option<FavoritePodcastEpisode>, Self::Error> {
        self.inner
            .get_by_user_id_and_episode_id(user_id, episode_id)
            .map_err(Into::into)
    }

    fn save_or_update(&self, favorite: FavoritePodcastEpisode) -> Result<(), Self::Error> {
        self.inner.save_or_update(favorite).map_err(Into::into)
    }

    fn is_liked_by_someone(&self, episode_id: Uuid) -> Result<bool, Self::Error> {
        self.inner
            .is_liked_by_someone(episode_id)
            .map_err(Into::into)
    }

    fn get_favorites_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FavoritePodcastEpisode>, Self::Error> {
        self.inner
            .get_favorites_by_user_id(user_id)
            .map_err(Into::into)
    }

    fn get_by_user_id_and_episode_ids(
        &self,
        user_id: Uuid,
        episode_ids: &[Uuid],
    ) -> Result<Vec<FavoritePodcastEpisode>, Self::Error> {
        self.inner
            .get_by_user_id_and_episode_ids(user_id, episode_ids)
            .map_err(Into::into)
    }

    fn delete_by_episode_ids(&self, episode_ids: &[Uuid]) -> Result<usize, Self::Error> {
        self.inner
            .delete_by_episode_ids(episode_ids)
            .map_err(Into::into)
    }
}

// ── EpisodeTriage ───────────────────────────────────────────────────────────

use crate::domain::episode_triage::{EpisodeTriage, EpisodeTriageRepository, TriageStatus};
use crate::persistence::episode_triage::DieselEpisodeTriageRepository;

pub struct EpisodeTriageRepositoryImpl {
    inner: DieselEpisodeTriageRepository,
}

impl EpisodeTriageRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselEpisodeTriageRepository::new(database),
        }
    }
}

impl EpisodeTriageRepository for EpisodeTriageRepositoryImpl {
    type Error = CustomError;

    fn get(&self, user_id: Uuid, episode_id: Uuid) -> Result<Option<EpisodeTriage>, Self::Error> {
        self.inner.get(user_id, episode_id).map_err(Into::into)
    }

    fn upsert(&self, triage: EpisodeTriage) -> Result<(), Self::Error> {
        self.inner.upsert(triage).map_err(Into::into)
    }

    fn delete(&self, user_id: Uuid, episode_id: Uuid) -> Result<usize, Self::Error> {
        self.inner.delete(user_id, episode_id).map_err(Into::into)
    }

    fn delete_by_episode_id(&self, episode_id: Uuid) -> Result<usize, Self::Error> {
        self.inner
            .delete_by_episode_id(episode_id)
            .map_err(Into::into)
    }

    fn list_episode_ids_by_status(
        &self,
        user_id: Uuid,
        status: TriageStatus,
    ) -> Result<Vec<Uuid>, Self::Error> {
        self.inner
            .list_episode_ids_by_status(user_id, status)
            .map_err(Into::into)
    }

    fn list_triaged_episode_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, Self::Error> {
        self.inner
            .list_triaged_episode_ids(user_id)
            .map_err(Into::into)
    }
}

// ── Playlist ────────────────────────────────────────────────────────────────

use crate::domain::playlist::{Playlist, PlaylistItem, PlaylistRepository};
use crate::persistence::playlist::DieselPlaylistRepository;

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
        user_id: Uuid,
    ) -> Result<Option<Playlist>, Self::Error> {
        self.inner
            .find_by_user_and_id(playlist_id, user_id)
            .map_err(Into::into)
    }

    fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Playlist>, Self::Error> {
        self.inner.list_by_user(user_id).map_err(Into::into)
    }

    fn update_playlist_name(
        &self,
        playlist_id: &str,
        user_id: Uuid,
        name: &str,
    ) -> Result<usize, Self::Error> {
        self.inner
            .update_playlist_name(playlist_id, user_id, name)
            .map_err(Into::into)
    }

    fn delete_playlist(&self, playlist_id: &str, user_id: Uuid) -> Result<usize, Self::Error> {
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
        episode_id: Uuid,
    ) -> Result<usize, Self::Error> {
        self.inner
            .delete_playlist_item(playlist_id, episode_id)
            .map_err(Into::into)
    }

    fn delete_items_by_episode_id(&self, episode_id: Uuid) -> Result<usize, Self::Error> {
        self.inner
            .delete_items_by_episode_id(episode_id)
            .map_err(Into::into)
    }
}

// ── Tag ─────────────────────────────────────────────────────────────────────

use crate::domain::tag::{Tag, TagRepository, TagUpdate, TagsPodcast};
use crate::persistence::tag::DieselTagRepository;

pub struct TagRepositoryImpl {
    inner: DieselTagRepository,
}

impl TagRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselTagRepository::new(database),
        }
    }
}

impl TagRepository for TagRepositoryImpl {
    type Error = CustomError;

    fn create(&self, tag: Tag) -> Result<Tag, Self::Error> {
        self.inner.create(tag).map_err(Into::into)
    }

    fn get_tags(&self, user_id: Uuid) -> Result<Vec<Tag>, Self::Error> {
        self.inner.get_tags(user_id).map_err(Into::into)
    }

    fn get_tags_of_podcast(
        &self,
        podcast_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Tag>, Self::Error> {
        self.inner
            .get_tags_of_podcast(podcast_id, user_id)
            .map_err(Into::into)
    }

    fn get_tags_of_podcasts(
        &self,
        podcast_ids: &[Uuid],
        user_id: Uuid,
    ) -> Result<Vec<(Uuid, Tag)>, Self::Error> {
        self.inner
            .get_tags_of_podcasts(podcast_ids, user_id)
            .map_err(Into::into)
    }

    fn get_tag_by_id_and_user_id(
        &self,
        tag_id: &str,
        user_id: Uuid,
    ) -> Result<Option<Tag>, Self::Error> {
        self.inner
            .get_tag_by_id_and_user_id(tag_id, user_id)
            .map_err(Into::into)
    }

    fn update(&self, tag_id: &str, update: TagUpdate) -> Result<Tag, Self::Error> {
        self.inner.update(tag_id, update).map_err(Into::into)
    }

    fn delete(&self, tag_id: &str) -> Result<(), Self::Error> {
        self.inner.delete(tag_id).map_err(Into::into)
    }

    fn add_podcast_to_tag(
        &self,
        tag_id_to_insert: String,
        podcast_id_to_insert: Uuid,
    ) -> Result<TagsPodcast, Self::Error> {
        self.inner
            .add_podcast_to_tag(tag_id_to_insert, podcast_id_to_insert)
            .map_err(Into::into)
    }

    fn delete_tag_podcasts(&self, tag_id: &str) -> Result<(), Self::Error> {
        self.inner.delete_tag_podcasts(tag_id).map_err(Into::into)
    }

    fn delete_tag_podcasts_by_podcast_id_tag_id(
        &self,
        podcast_id: Uuid,
        tag_id: &str,
    ) -> Result<(), Self::Error> {
        self.inner
            .delete_tag_podcasts_by_podcast_id_tag_id(podcast_id, tag_id)
            .map_err(Into::into)
    }

    fn delete_tag_podcasts_by_podcast_id(&self, podcast_id: Uuid) -> Result<(), Self::Error> {
        self.inner
            .delete_tag_podcasts_by_podcast_id(podcast_id)
            .map_err(Into::into)
    }
}
