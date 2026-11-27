//! What other contexts may ask the catalog. Today a direct call
//! ([`LocalCatalog`]); a catalog run as its own service would be reached
//! through an HTTP client implementing the same trait.

use crate::domain::podcast::{Podcast, PodcastRepository};
use crate::domain::podcast_episode::{PodcastEpisode, PodcastEpisodeRepository};
use crate::persistence::podcast::DieselPodcastRepository;
use crate::persistence::podcast_episode::DieselPodcastEpisodeRepository;
use common_infrastructure::db::Database;
use common_infrastructure::error::CustomError;
use uuid::Uuid;

pub trait CatalogApi: Send + Sync {
    fn podcast(&self, podcast_id: Uuid) -> Result<Option<Podcast>, CustomError>;
    fn podcast_by_feed(&self, feed_url: &str) -> Result<Option<Podcast>, CustomError>;
    fn episode(&self, episode_id: Uuid) -> Result<Option<PodcastEpisode>, CustomError>;
    fn episodes_of_podcast(&self, podcast_id: Uuid) -> Result<Vec<PodcastEpisode>, CustomError>;
    /// The podcast's `n` most recent episodes.
    fn latest_episodes(&self, podcast_id: Uuid, n: i64)
    -> Result<Vec<PodcastEpisode>, CustomError>;
}

/// The catalog in this process.
pub struct LocalCatalog {
    podcasts: DieselPodcastRepository,
    episodes: DieselPodcastEpisodeRepository,
}

impl LocalCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            podcasts: DieselPodcastRepository::new(database.clone()),
            episodes: DieselPodcastEpisodeRepository::new(database),
        }
    }
}

impl CatalogApi for LocalCatalog {
    fn podcast(&self, podcast_id: Uuid) -> Result<Option<Podcast>, CustomError> {
        Ok(self.podcasts.find_by_id(podcast_id)?)
    }

    fn podcast_by_feed(&self, feed_url: &str) -> Result<Option<Podcast>, CustomError> {
        Ok(self.podcasts.find_by_rss_feed(feed_url)?)
    }

    fn episode(&self, episode_id: Uuid) -> Result<Option<PodcastEpisode>, CustomError> {
        Ok(self.episodes.find_by_id(episode_id)?)
    }

    fn episodes_of_podcast(&self, podcast_id: Uuid) -> Result<Vec<PodcastEpisode>, CustomError> {
        Ok(self.episodes.find_by_podcast_id(podcast_id)?)
    }

    fn latest_episodes(
        &self,
        podcast_id: Uuid,
        n: i64,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        Ok(self.episodes.get_last_n_episodes(podcast_id, n)?)
    }
}
