//! Direct-streaming endpoint for an open playback session.
//!
//! Audiobookshelf mobile apps authenticate this via `?token=<api_key>` query
//! because `<audio>` tags can't set custom headers. Range requests are honored.

use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_audiobooks::domain::library_item_id::{EpisodeId, LibraryItemId};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio_util::io::ReaderStream;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/public/session/{sid}/track/{idx}",
    params(
        ("sid" = String, Path, description = "Session id"),
        ("idx" = i32, Path, description = "Track index (0-based)")
    ),
    responses(
        (status = 200, description = "Full file"),
        (status = 206, description = "Range")
    ),
    tag = "audiobookshelf"
)]
pub async fn serve_track(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((session_id, _idx)): Path<(String, i32)>,
    headers: HeaderMap,
) -> Result<Response, CustomError> {
    let session = state
        .audiobookshelf_playback_session_service
        .find_by_id(&session_id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    if session.user_id != user.id {
        return Err(CustomErrorInner::Forbidden(Debug).into());
    }

    let parsed_item = LibraryItemId::parse(&session.library_item_id)
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;

    let local = match parsed_item {
        LibraryItemId::Podcast(podcast_id) => {
            let episode_id = session
                .episode_id
                .as_deref()
                .and_then(EpisodeId::parse)
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            let _ = PodcastService::get_podcast(podcast_id)?;
            let episode = PodcastEpisodeService::get_episodes_by_podcast_id(podcast_id)?
                .into_iter()
                .map(podfetch_domain::podcast_episode::PodcastEpisode::from)
                .find(|e| e.id == episode_id.0)
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            episode
                .file_episode_path
                .clone()
                .or_else(|| episode.download_location.clone())
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?
        }
        LibraryItemId::Book(_) => {
            let book = state
                .audiobookshelf_book_service
                .find_by_id(&session.library_item_id)?
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            let aggregate = state.audiobookshelf_book_service.hydrate(book)?;
            aggregate
                .audio_files
                .iter()
                .find(|af| af.idx == _idx)
                .map(|af| af.path.clone())
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?
        }
    };
    let path = PathBuf::from(&local);
    serve_file_with_range(&path, &headers).await
}

/// Reusable HTTP Range-aware file server used by both /public/session/*/track
/// and /api/items/*/file/*. Returns 206 with Content-Range when Range header
/// is present, else full 200.
pub async fn serve_file_with_range(
    path: &std::path::Path,
    headers: &HeaderMap,
) -> Result<Response, CustomError> {
    if !path.is_file() {
        return Err(CustomErrorInner::NotFound(Debug).into());
    }
    let not_found = |_| CustomError::from(CustomErrorInner::NotFound(Debug));
    let mut file = fs::File::open(&path).await.map_err(not_found)?;
    let total = file.metadata().await.map_err(not_found)?.len();
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut out_headers = HeaderMap::new();
    if let Ok(mime) = HeaderValue::from_str(mime.as_ref()) {
        out_headers.insert(header::CONTENT_TYPE, mime);
    }
    out_headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));

    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_range)
        .and_then(|r| materialize_range(r, total));

    // Stream the file instead of buffering it: ranges can span a whole
    // multi-hundred-megabyte audiobook.
    let (status, len) = match range {
        Some((start, end)) => {
            file.seek(SeekFrom::Start(start)).await.map_err(not_found)?;
            let content_range = format!("bytes {start}-{end}/{total}");
            if let Ok(value) = HeaderValue::from_str(&content_range) {
                out_headers.insert(header::CONTENT_RANGE, value);
            }
            (StatusCode::PARTIAL_CONTENT, end - start + 1)
        }
        None => (StatusCode::OK, total),
    };
    out_headers.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    let body = Body::from_stream(ReaderStream::new(file.take(len)));
    Ok((status, out_headers, body).into_response())
}

fn parse_range(value: &str) -> Option<(Option<u64>, Option<u64>)> {
    let rest = value.strip_prefix("bytes=")?;
    let mut parts = rest.split('-');
    let start = parts.next()?;
    let end = parts.next()?;
    let start = if start.is_empty() {
        None
    } else {
        Some(start.parse::<u64>().ok()?)
    };
    let end = if end.is_empty() {
        None
    } else {
        Some(end.parse::<u64>().ok()?)
    };
    if start.is_none() && end.is_none() {
        return None;
    }
    Some((start, end))
}

fn materialize_range(range: (Option<u64>, Option<u64>), total: u64) -> Option<(u64, u64)> {
    let (start, end) = range;
    let last = total.checked_sub(1)?;
    match (start, end) {
        (Some(s), Some(e)) if s <= e && e <= last => Some((s, e)),
        (Some(s), None) if s <= last => Some((s, last)),
        (None, Some(suffix)) if suffix > 0 => {
            let s = total.saturating_sub(suffix);
            Some((s, last))
        }
        _ => None,
    }
}

pub fn get_public_session_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(serve_track))
}
