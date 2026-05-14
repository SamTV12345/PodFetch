use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::dto::library::{LibrariesListResponse, LibraryDto};
use crate::audiobookshelf_api::mapping::podcast::map_episode_for_recent;
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::Json;
use axum::extract::{Path, Query, State};
use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::audiobookshelf::library::MediaType;
use serde::Deserialize;
use serde_json::{Value, json};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/api/libraries",
    responses((status = 200, description = "All libraries", body = LibrariesListResponse)),
    tag = "audiobookshelf"
)]
pub async fn list_libraries(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<LibrariesListResponse>, CustomError> {
    let libraries = state.audiobookshelf_library_service.list()?;
    Ok(Json(LibrariesListResponse {
        libraries: libraries.iter().map(LibraryDto::from).collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/libraries/{id}",
    params(("id" = String, Path, description = "Library id")),
    responses(
        (status = 200, description = "Library", body = LibraryDto),
        (status = 404, description = "Not found")
    ),
    tag = "audiobookshelf"
)]
pub async fn get_library(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<LibraryDto>, CustomError> {
    let library = state
        .audiobookshelf_library_service
        .find_by_id(&id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    Ok(Json(LibraryDto::from(&library)))
}

#[derive(Deserialize)]
pub struct RecentEpisodesQuery {
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

/// Mirrors upstream `LibraryController.getRecentEpisodes`:
/// `GET /api/libraries/:id/recent-episodes?limit=N&page=N` returns the
/// newest episodes across all podcasts in the library, each with the
/// parent podcast metadata embedded under `podcast`. Used by the mobile
/// apps' "Latest" home screen — failing this endpoint produces the
/// "Failed to get recent episodes" toast.
#[utoipa::path(
    get,
    path = "/api/libraries/{id}/recent-episodes",
    params(
        ("id" = String, Path),
        ("limit" = Option<i64>, Query),
        ("page" = Option<i64>, Query),
    ),
    responses((status = 200, description = "Recent episodes envelope")),
    tag = "audiobookshelf"
)]
pub async fn get_recent_episodes(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(library_id): Path<String>,
    Query(query): Query<RecentEpisodesQuery>,
) -> Result<Json<Value>, CustomError> {
    let library = state
        .audiobookshelf_library_service
        .find_by_id(&library_id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    if !matches!(library.media_type, MediaType::Podcast) {
        return Err(CustomErrorInner::NotFound(Debug).into());
    }

    // Collect every active episode tagged with its parent podcast, then
    // sort by publishedAt DESC. PodFetch stores publishedAt as the
    // episode's `date_of_recording` (RFC-2822 or RFC-3339); we parse both
    // for sorting and fall back to download_time.
    let podcasts = PodcastService::get_all_podcasts_raw()?;
    let mut episodes_with_podcast: Vec<(podfetch_domain::podcast::Podcast, podfetch_domain::podcast_episode::PodcastEpisode, i64)> = Vec::new();
    for podcast_entity in podcasts {
        let domain_podcast: podfetch_domain::podcast::Podcast = podcast_entity.into();
        let eps: Vec<podfetch_domain::podcast_episode::PodcastEpisode> =
            PodcastEpisodeService::get_episodes_by_podcast_id(domain_podcast.id)
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect();
        for ep in eps.into_iter().filter(|e| !e.deleted) {
            let ts = parse_episode_timestamp(&ep);
            episodes_with_podcast.push((domain_podcast.clone(), ep, ts));
        }
    }
    episodes_with_podcast.sort_by(|a, b| b.2.cmp(&a.2));

    let limit_raw = query.limit.unwrap_or(0);
    let limit = if limit_raw <= 0 {
        episodes_with_podcast.len() as i64
    } else {
        limit_raw
    };
    let page = query.page.unwrap_or(0).max(0);
    let offset = (page * limit).max(0) as usize;
    let end = (offset + limit as usize).min(episodes_with_podcast.len());
    let window = if offset < episodes_with_podcast.len() {
        &episodes_with_podcast[offset..end]
    } else {
        &[]
    };

    let episodes: Vec<Value> = window
        .iter()
        .enumerate()
        .map(|(idx, (podcast, ep, _))| {
            map_episode_for_recent(podcast, ep, (offset + idx) as i32 + 1, &library.id)
        })
        .collect();

    Ok(Json(json!({
        "episodes": episodes,
        "limit": limit_raw,
        "page": page,
        "total": episodes_with_podcast.len(),
    })))
}

fn parse_episode_timestamp(ep: &podfetch_domain::podcast_episode::PodcastEpisode) -> i64 {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&ep.date_of_recording) {
        return dt.timestamp_millis();
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(&ep.date_of_recording) {
        return dt.timestamp_millis();
    }
    ep.download_time
        .map(|t| t.and_utc().timestamp_millis())
        .unwrap_or(0)
}

/// Stub for upstream `GET /api/libraries/:id/personalized`. The mobile
/// apps probe this for the home-screen shelves; returning `[]` makes
/// them show an empty home without surfacing an error. Real shelves
/// (Continue Listening / Recently Added) can be implemented later
/// without breaking the apps.
#[utoipa::path(
    get,
    path = "/api/libraries/{id}/personalized",
    params(("id" = String, Path)),
    responses((status = 200, description = "Personalized shelves (empty stub)")),
    tag = "audiobookshelf"
)]
pub async fn get_personalized(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(library_id): Path<String>,
) -> Result<Json<Value>, CustomError> {
    let _ = state
        .audiobookshelf_library_service
        .find_by_id(&library_id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    Ok(Json(Value::Array(Vec::new())))
}

pub fn get_libraries_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_libraries))
        .routes(routes!(get_library))
        .routes(routes!(get_recent_episodes))
        .routes(routes!(get_personalized))
}
