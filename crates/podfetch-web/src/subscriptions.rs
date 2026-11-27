//! Wiring of the catalog's subscriptions. The media library stores the
//! podcast's directory and cover; the first refresh runs right after the
//! podcast is added, and the UI hears about both.

use crate::server::ChatServerHandle;
use crate::services::file::service::FileService;
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_catalog::domain::podcast::Podcast;
use podfetch_catalog::feed::HttpFeedFetcher;
use podfetch_catalog::persistence::podcast::PodcastEntity;
use podfetch_catalog::subscribe::{Directories, PodcastStorage, SubscribeRequest, Subscriptions};
use podfetch_identity::api::Principal;
use rss::Channel;
use std::sync::Arc;
use uuid::Uuid;

/// The media library's side of a new podcast: its directory and cover.
struct MediaPodcastStorage;

impl PodcastStorage for MediaPodcastStorage {
    fn create_directory(
        &self,
        channel: &Channel,
        directory_id_known: bool,
    ) -> Result<String, CustomError> {
        FileService::create_podcast_directory(channel, directory_id_known)
    }

    fn store_cover(&self, podcast: &Podcast) -> Result<(), CustomError> {
        // Subscribing runs on a blocking thread of the runtime.
        tokio::runtime::Handle::current().block_on(FileService::download_podcast_image(
            &podcast.directory_name,
            podcast.image_url.clone(),
            &podcast.directory_id,
            &crate::media::cover_filename(podcast.id),
        ))
    }
}

fn environment() -> Arc<common_infrastructure::config::EnvironmentService> {
    Arc::new(ENVIRONMENT_SERVICE.clone())
}

/// The catalog's subscriptions for code without injected dependencies.
pub fn subscriptions() -> Subscriptions {
    Subscriptions::new(
        podfetch_persistence::db::database(),
        Arc::new(HttpFeedFetcher::new(environment())),
        Arc::new(MediaPodcastStorage),
        ENVIRONMENT_SERVICE.user_podcast_limit,
    )
}

pub fn directories() -> Directories {
    Directories::new(environment())
}

/// Whether `user` may add `adding` more podcasts. Every endpoint that
/// subscribes the server to a feed - native API and compatibility layers
/// alike - must call this.
pub fn ensure_can_add_podcasts(user: &Principal, adding: u32) -> Result<(), CustomError> {
    subscriptions().ensure_can_add(user.id.into(), user.is_privileged_user(), adding)
}

async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, CustomError> + Send + 'static,
) -> Result<T, CustomError> {
    tokio::task::spawn_blocking(work).await.map_err(|e| {
        tracing::error!("Blocking task failed: {e}");
        CustomError::from(CustomErrorInner::Unknown(ErrorSeverity::Error))
    })?
}

/// Adds a podcast and runs its first refresh.
pub async fn subscribe(request: SubscribeRequest) -> Result<PodcastEntity, CustomError> {
    let podcast: PodcastEntity = blocking(move || subscriptions().subscribe(request))
        .await?
        .into();
    ChatServerHandle::broadcast_podcast_downloaded(podcast.clone());
    let refreshed = podcast.clone();
    blocking(move || {
        tracing::debug!("Inserting podcast episodes of {}", refreshed.name);
        match PodcastEpisodeUseCase::insert_podcast_episodes(&refreshed) {
            Ok(episodes) => {
                ChatServerHandle::broadcast_added_podcast_episodes(&refreshed, episodes);
                PodcastService::announce_refreshed(&refreshed);
            }
            // The podcast is added; the next refresh tries again.
            Err(e) => tracing::error!(
                "Error inserting podcast episodes for {}: {e}",
                refreshed.name
            ),
        }
        Ok(())
    })
    .await?;
    Ok(podcast)
}

/// Adds a podcast from iTunes.
pub async fn subscribe_itunes(
    collection_id: i32,
    added_by: Option<Uuid>,
) -> Result<PodcastEntity, CustomError> {
    let request = blocking(move || directories().itunes(collection_id)).await?;
    subscribe(SubscribeRequest {
        added_by,
        ..request
    })
    .await
}

/// Adds a podcast from Podindex.
pub async fn subscribe_podindex(
    feed_id: i32,
    added_by: Option<Uuid>,
) -> Result<PodcastEntity, CustomError> {
    let request = blocking(move || directories().podindex(feed_id)).await?;
    subscribe(SubscribeRequest {
        added_by,
        ..request
    })
    .await
}
