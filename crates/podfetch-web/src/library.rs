//! Library services for code that has no injected dependencies yet.
//!
//! Transitional: the podcast and episode pipeline still consists of
//! associated functions. Once those get their own services (migration
//! step 5), they receive the library services through `AppState` and this
//! module goes away.

use podfetch_library::persistence::{
    FavoritePodcastEpisodeRepositoryImpl, FilterRepositoryImpl, TagRepositoryImpl,
};
use podfetch_library::services::favorite_podcast_episode::FavoritePodcastEpisodeService;
use podfetch_library::services::filter::FilterService;
use podfetch_library::services::tag::TagService;
use podfetch_persistence::db::database;
use std::sync::Arc;

pub fn tags() -> TagService {
    TagService::new(Arc::new(TagRepositoryImpl::new(database())))
}

pub fn filters() -> FilterService {
    FilterService::new(Arc::new(FilterRepositoryImpl::new(database())))
}

pub fn favorite_episodes() -> FavoritePodcastEpisodeService {
    FavoritePodcastEpisodeService::new(Arc::new(FavoritePodcastEpisodeRepositoryImpl::new(
        database(),
    )))
}
