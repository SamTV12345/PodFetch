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

/// The catalog ids of the podcast's episodes that some user marked as
/// favorite.
pub fn liked_episodes_of_podcast(
    podcast_id: uuid::Uuid,
) -> Result<std::collections::HashSet<String>, common_infrastructure::error::CustomError> {
    let favorites = favorite_episodes();
    let mut liked = std::collections::HashSet::new();
    for episode in
        crate::usecases::podcast_episode::PodcastEpisodeUseCase::get_episodes_by_podcast_id(
            podcast_id,
        )?
    {
        let id = uuid::Uuid::parse_str(&episode.id).map_err(|_| {
            common_infrastructure::error::CustomError::from(
                common_infrastructure::error::CustomErrorInner::Unknown(
                    common_infrastructure::error::ErrorSeverity::Error,
                ),
            )
        })?;
        if favorites.is_liked_by_someone(id)? {
            liked.insert(episode.id);
        }
    }
    Ok(liked)
}
