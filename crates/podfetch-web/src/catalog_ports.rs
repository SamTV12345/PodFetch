//! Catalog-side implementations of the ports other contexts declare.
//!
//! The catalog is not a crate of its own yet (migration step 5), so its
//! answers to other contexts are given here, on top of the podcast service.

use crate::services::podcast::service::PodcastService;
use common_infrastructure::error::CustomError;
use std::collections::HashMap;
use uuid::Uuid;

/// Feed URL ↔ podcast lookups for the library and gPodder sync.
pub struct CatalogFeeds;

impl podfetch_library::handlers::FeedResolver for CatalogFeeds {
    fn podcasts_by_feed(&self, feeds: &[String]) -> Result<HashMap<String, Uuid>, CustomError> {
        let mut found = HashMap::new();
        for feed in feeds {
            if let Some(podcast) = PodcastService::find_by_rss_feed(feed)? {
                found.insert(feed.clone(), podcast.id);
            }
        }
        Ok(found)
    }
}

impl podfetch_gpodder::handlers::PodcastFeeds for CatalogFeeds {
    fn feed_of(&self, podcast_id: Uuid) -> Result<Option<String>, CustomError> {
        Ok(PodcastService::find_by_id(podcast_id)?.map(|podcast| podcast.rssfeed))
    }
}
