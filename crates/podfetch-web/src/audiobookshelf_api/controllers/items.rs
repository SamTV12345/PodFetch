use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::mapping::book::map_book;
use crate::audiobookshelf_api::mapping::podcast::map_podcast;
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Json, Response};
use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::audiobookshelf::library::MediaType;
use podfetch_domain::audiobookshelf::library_item_id::LibraryItemId;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::fs;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Deserialize)]
pub struct ItemsQuery {
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/libraries/{id}/items",
    params(
        ("id" = String, Path, description = "Library id"),
        ("limit" = Option<i64>, Query, description = "Page size"),
        ("page" = Option<i64>, Query, description = "0-indexed page")
    ),
    responses((status = 200, description = "Items in library")),
    tag = "audiobookshelf"
)]
pub async fn list_library_items(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(library_id): Path<String>,
    Query(query): Query<ItemsQuery>,
) -> Result<Json<Value>, CustomError> {
    let library = state
        .audiobookshelf_library_service
        .find_by_id(&library_id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;

    let mut all_items: Vec<Value> = match library.media_type {
        MediaType::Podcast => {
            let podcasts = PodcastService::get_all_podcasts_raw()?;
            podcasts
                .into_iter()
                .map(|podcast_entity| {
                    let domain_podcast: podfetch_domain::podcast::Podcast =
                        podcast_entity.into();
                    let episodes: Vec<podfetch_domain::podcast_episode::PodcastEpisode> =
                        PodcastEpisodeService::get_episodes_by_podcast_id(domain_podcast.id)
                            .unwrap_or_default()
                            .into_iter()
                            .map(Into::into)
                            .collect();
                    map_podcast(&domain_podcast, &episodes, &library.id)
                })
                .collect()
        }
        MediaType::Book => {
            let books = state
                .audiobookshelf_book_service
                .list_by_library(&library.id)?;
            books
                .into_iter()
                .filter_map(|book| state.audiobookshelf_book_service.hydrate(book).ok())
                .map(|agg| map_book(&agg))
                .collect()
        }
    };

    let total = all_items.len() as i64;
    let limit = query.limit.unwrap_or(50).clamp(1, 1000);
    let page = query.page.unwrap_or(0).max(0);
    let start = ((page * limit) as usize).min(all_items.len());
    let end = ((page * limit + limit) as usize).min(all_items.len());
    let results: Vec<Value> = all_items.drain(start..end).collect();

    Ok(Json(json!({
        "results": results,
        "total": total,
        "limit": limit,
        "page": page,
    })))
}

#[utoipa::path(
    get,
    path = "/api/items/{id}",
    params(("id" = String, Path, description = "Library item id (li_pod_<n> | li_book_<uuid>)")),
    responses(
        (status = 200, description = "Library item"),
        (status = 404, description = "Not found")
    ),
    tag = "audiobookshelf"
)]
pub async fn get_library_item(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, CustomError> {
    let parsed = LibraryItemId::parse(&id)
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;

    match parsed {
        LibraryItemId::Podcast(podcast_id) => {
            let podcast_entity = PodcastService::get_podcast(podcast_id)?;
            let domain_podcast: podfetch_domain::podcast::Podcast = podcast_entity.into();
            let episodes: Vec<podfetch_domain::podcast_episode::PodcastEpisode> =
                PodcastEpisodeService::get_episodes_by_podcast_id(podcast_id)?
                    .into_iter()
                    .map(Into::into)
                    .collect();
            let library = state
                .audiobookshelf_library_service
                .find_default_podcasts_library()?
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            Ok(Json(map_podcast(&domain_podcast, &episodes, &library.id)))
        }
        LibraryItemId::Book(_) => {
            let book = state
                .audiobookshelf_book_service
                .find_by_id(&id)?
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            let aggregate = state.audiobookshelf_book_service.hydrate(book)?;
            Ok(Json(map_book(&aggregate)))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/items/{id}/cover",
    params(("id" = String, Path, description = "Library item id")),
    responses(
        (status = 200, description = "Cover image bytes"),
        (status = 404, description = "Not found")
    ),
    tag = "audiobookshelf"
)]
pub async fn get_item_cover(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Response, CustomError> {
    let parsed = LibraryItemId::parse(&id)
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;

    match parsed {
        LibraryItemId::Podcast(pid) => {
            let podcast_entity = PodcastService::get_podcast(pid)?;
            let podcast: podfetch_domain::podcast::Podcast = podcast_entity.into();
            // Prefer a local file on disk; if image_url is actually a remote
            // URL (most podcasts: image lives in the RSS feed), redirect the
            // client there so the mobile app's image loader handles it.
            let candidate = PathBuf::from(&podcast.image_url);
            if candidate.is_file() {
                return serve_file(candidate).await;
            }
            let local_in_dir = PathBuf::from(&podcast.directory_name).join("cover.jpg");
            if local_in_dir.is_file() {
                return serve_file(local_in_dir).await;
            }
            if podcast.image_url.starts_with("http://")
                || podcast.image_url.starts_with("https://")
            {
                let mut headers = HeaderMap::new();
                if let Ok(loc) = HeaderValue::from_str(&podcast.image_url) {
                    headers.insert(header::LOCATION, loc);
                }
                return Ok((StatusCode::FOUND, headers, Body::empty()).into_response());
            }
            Err(CustomErrorInner::NotFound(Debug).into())
        }
        LibraryItemId::Book(_) => {
            let book = state
                .audiobookshelf_book_service
                .find_by_id(&id)?
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            if let Some(cover_path) = book.cover_path.as_deref() {
                let candidate = PathBuf::from(cover_path);
                if candidate.is_file() {
                    return serve_file(candidate).await;
                }
            }
            Err(CustomErrorInner::NotFound(Debug).into())
        }
    }
}

async fn serve_file(path: PathBuf) -> Result<Response, CustomError> {
    let bytes = fs::read(&path)
        .await
        .map_err(|_| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    let mime = mime_guess::from_path(&path).first_or_octet_stream();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref()).unwrap_or(HeaderValue::from_static("image/jpeg")),
    );

    Ok((StatusCode::OK, headers, Body::from(bytes)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/items/{id}/file/{ino}",
    params(
        ("id" = String, Path, description = "Library item id"),
        ("ino" = String, Path, description = "Audio file ino (ino_ep_<id> for podcasts, ino_book_<id> for books)")
    ),
    responses(
        (status = 200, description = "Full file"),
        (status = 206, description = "Range partial content")
    ),
    tag = "audiobookshelf"
)]
pub async fn get_item_file(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path((id, ino)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, CustomError> {
    let parsed = LibraryItemId::parse(&id)
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    let local_path: PathBuf = match parsed {
        LibraryItemId::Podcast(podcast_id) => {
            let episode_db_id = ino
                .strip_prefix("ino_ep_")
                .and_then(|s| s.parse::<i32>().ok())
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            let _ = PodcastService::get_podcast(podcast_id)?;
            let episode = PodcastEpisodeService::get_episodes_by_podcast_id(podcast_id)?
                .into_iter()
                .map(podfetch_domain::podcast_episode::PodcastEpisode::from)
                .find(|e| e.id == episode_db_id)
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            let local = episode
                .file_episode_path
                .clone()
                .or_else(|| episode.download_location.clone())
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            PathBuf::from(local)
        }
        LibraryItemId::Book(_) => {
            let book = state
                .audiobookshelf_book_service
                .find_by_id(&id)?
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            let aggregate = state.audiobookshelf_book_service.hydrate(book)?;
            let file = aggregate
                .audio_files
                .iter()
                .find(|af| af.id == ino || af.ino.as_deref() == Some(&ino))
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            PathBuf::from(file.path.clone())
        }
    };
    crate::audiobookshelf_api::controllers::public_session::serve_file_with_range(
        &local_path,
        &headers,
    )
    .await
}

pub fn get_items_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_library_items))
        .routes(routes!(get_library_item))
        .routes(routes!(get_item_cover))
        .routes(routes!(get_item_file))
}
