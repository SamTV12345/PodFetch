//! The ports other contexts declare towards the catalog, answered through
//! the catalog's API (translation at the boundary).

use common_infrastructure::error::CustomError;
use podfetch_catalog::api::CatalogApi;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Feed URL ↔ podcast lookups for the library and gPodder sync.
pub struct CatalogFeeds(pub Arc<dyn CatalogApi>);

impl podfetch_library::handlers::FeedResolver for CatalogFeeds {
    fn podcasts_by_feed(&self, feeds: &[String]) -> Result<HashMap<String, Uuid>, CustomError> {
        let mut found = HashMap::new();
        for feed in feeds {
            if let Some(podcast) = self.0.podcast_by_feed(feed)? {
                found.insert(feed.clone(), podcast.id);
            }
        }
        Ok(found)
    }
}

impl podfetch_gpodder::handlers::PodcastFeeds for CatalogFeeds {
    fn feed_of(&self, podcast_id: Uuid) -> Result<Option<String>, CustomError> {
        Ok(self.0.podcast(podcast_id)?.map(|podcast| podcast.rssfeed))
    }
}
