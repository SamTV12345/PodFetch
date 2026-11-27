//! What the library keeps about a podcast, removed before the catalog
//! deletes it. Doesn't rely on the foreign keys: several of them don't
//! cascade, and the context must clean up after itself either way.

use crate::domain::episode_triage::EpisodeTriageRepository;
use crate::domain::favorite::FavoriteRepository;
use crate::domain::favorite_podcast_episode::FavoritePodcastEpisodeRepository;
use crate::domain::playlist::PlaylistRepository;
use crate::domain::tag::TagRepository;
use crate::persistence::adapters::{
    EpisodeTriageRepositoryImpl, FavoritePodcastEpisodeRepositoryImpl, FavoriteRepositoryImpl,
    PlaylistRepositoryImpl, TagRepositoryImpl,
};
use common_infrastructure::db::Database;
use common_infrastructure::error::CustomError;
use uuid::Uuid;

pub struct LibraryPodcastRemoval {
    favorites: FavoriteRepositoryImpl,
    favorite_episodes: FavoritePodcastEpisodeRepositoryImpl,
    tags: TagRepositoryImpl,
    playlists: PlaylistRepositoryImpl,
    triage: EpisodeTriageRepositoryImpl,
}

impl LibraryPodcastRemoval {
    pub fn new(database: Database) -> Self {
        Self {
            favorites: FavoriteRepositoryImpl::new(database.clone()),
            favorite_episodes: FavoritePodcastEpisodeRepositoryImpl::new(database.clone()),
            tags: TagRepositoryImpl::new(database.clone()),
            playlists: PlaylistRepositoryImpl::new(database.clone()),
            triage: EpisodeTriageRepositoryImpl::new(database),
        }
    }

    /// Removes the podcast's favorites and tag assignments, and its
    /// episodes' favorites, playlist entries and triage states.
    pub fn forget_podcast(
        &self,
        podcast_id: Uuid,
        episode_ids: &[Uuid],
    ) -> Result<(), CustomError> {
        self.favorites.delete_by_podcast_id(podcast_id)?;
        self.tags.delete_tag_podcasts_by_podcast_id(podcast_id)?;
        self.favorite_episodes.delete_by_episode_ids(episode_ids)?;
        for &episode_id in episode_ids {
            self.playlists.delete_items_by_episode_id(episode_id)?;
            self.triage.delete_by_episode_id(episode_id)?;
        }
        Ok(())
    }
}
