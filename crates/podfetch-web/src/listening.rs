//! Listening services for code that has no injected dependencies yet.
//!
//! Transitional, like [`crate::library`]: goes away once the podcast and
//! episode pipeline gets its own services (migration step 5).

use podfetch_listening::persistence::ListeningEventRepositoryImpl;
use podfetch_listening::persistence::episode::DieselEpisodeRepository;
use podfetch_listening::services::listening_event::ListeningEventService;
use podfetch_persistence::db::database;
use std::sync::Arc;

pub fn listening_events() -> ListeningEventService {
    ListeningEventService::new(Arc::new(ListeningEventRepositoryImpl::new(database())))
}

pub fn episode_actions() -> DieselEpisodeRepository {
    DieselEpisodeRepository::new(database())
}

/// The catalog side of the statistics' [`PodcastLookup`] port.
pub struct CatalogPodcasts;

impl podfetch_listening::services::stats::PodcastLookup for CatalogPodcasts {
    fn describe(
        &self,
        podcast_ids: &[uuid::Uuid],
    ) -> Result<
        std::collections::HashMap<uuid::Uuid, podfetch_listening::services::stats::PodcastInfo>,
        common_infrastructure::error::CustomError,
    > {
        Ok(
            crate::services::podcast::service::PodcastService::get_podcasts_by_ids(podcast_ids)?
                .into_iter()
                .filter_map(|podcast| {
                    let id = uuid::Uuid::parse_str(&podcast.id).ok()?;
                    Some((id, podcast_listening_info(podcast.name, podcast.image_url)))
                })
                .collect(),
        )
    }
}

fn podcast_listening_info(
    name: String,
    image_url: String,
) -> podfetch_listening::services::stats::PodcastInfo {
    podfetch_listening::services::stats::PodcastInfo { name, image_url }
}
