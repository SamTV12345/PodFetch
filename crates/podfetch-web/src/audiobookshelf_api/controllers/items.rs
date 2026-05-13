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
                    let dto = map_podcast(&domain_podcast, &episodes, &library.id);
                    serde_json::to_value(dto).unwrap_or(Value::Null)
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
            let dto = map_podcast(&domain_podcast, &episodes, &library.id);
            Ok(Json(serde_json::to_value(dto).unwrap_or(Value::Null)))
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
            let candidate = PathBuf::from(&podcast.image_url);
            if candidate.is_file() {
                return serve_file(candidate).await;
            }
            let local_in_dir = PathBuf::from(&podcast.directory_name).join("cover.jpg");
            if local_in_dir.is_file() {
                return serve_file(local_in_dir).await;
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

pub fn get_items_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_library_items))
        .routes(routes!(get_library_item))
        .routes(routes!(get_item_cover))
}
