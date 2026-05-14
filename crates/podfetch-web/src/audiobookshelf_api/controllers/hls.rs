//! HLS endpoints. Reads back the live `PlaybackSession` to determine which
//! audio file to transcode for the requested segment.

use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::services::audiobookshelf::hls_transcoder::{
    build_master_playlist, build_media_playlist,
};
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::audiobookshelf::library_item_id::{EpisodeId, LibraryItemId};
use std::path::PathBuf;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/hls/{stream_id}/{file}",
    params(
        ("stream_id" = String, Path, description = "Session id (used as stream id)"),
        ("file" = String, Path, description = "master.m3u8 | index.m3u8 | seg-<N>.ts"),
    ),
    responses((status = 200, description = "HLS playlist or transcoded segment")),
    tag = "audiobookshelf"
)]
pub async fn hls_dispatch(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((stream_id, file)): Path<(String, String)>,
) -> Result<Response, CustomError> {
    let session = state
        .audiobookshelf_playback_session_service
        .find_by_id(&stream_id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    if session.user_id != user.id {
        return Err(CustomErrorInner::Forbidden(Debug).into());
    }

    if file == "master.m3u8" {
        return Ok(m3u8_response(build_master_playlist(&stream_id)));
    }
    if file == "index.m3u8" {
        return Ok(m3u8_response(build_media_playlist(
            &stream_id,
            session.duration,
        )));
    }
    if let Some(idx_str) = file
        .strip_prefix("seg-")
        .and_then(|rest| rest.strip_suffix(".ts"))
        && let Ok(segment_idx) = idx_str.parse::<u32>()
    {
        return serve_segment(state, session, stream_id, segment_idx).await;
    }
    Err(CustomErrorInner::NotFound(Debug).into())
}

async fn serve_segment(
    state: AppState,
    session: podfetch_domain::audiobookshelf::playback_session::PlaybackSession,
    stream_id: String,
    segment_idx: u32,
) -> Result<Response, CustomError> {
    let source = resolve_source_audio(&state, &session).await?;
    let segment_path = state
        .audiobookshelf_hls_transcoder
        .ensure_segment(&stream_id, &source, segment_idx)
        .await
        .map_err(|e| {
            CustomError::from(CustomErrorInner::Conflict(
                format!("HLS transcode failed: {e}"),
                common_infrastructure::error::ErrorSeverity::Warning,
            ))
        })?;
    let bytes = tokio::fs::read(&segment_path)
        .await
        .map_err(|_| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("video/mp2t"));
    Ok((StatusCode::OK, headers, Body::from(bytes)).into_response())
}

async fn resolve_source_audio(
    state: &AppState,
    session: &podfetch_domain::audiobookshelf::playback_session::PlaybackSession,
) -> Result<PathBuf, CustomError> {
    let parsed = LibraryItemId::parse(&session.library_item_id)
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    match parsed {
        LibraryItemId::Podcast(pid) => {
            let episode_id = session
                .episode_id
                .as_deref()
                .and_then(EpisodeId::parse)
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            let _ = PodcastService::get_podcast(pid)?;
            let episode = PodcastEpisodeService::get_episodes_by_podcast_id(pid)?
                .into_iter()
                .map(podfetch_domain::podcast_episode::PodcastEpisode::from)
                .find(|e| e.id == episode_id.0)
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            let path = episode
                .file_episode_path
                .clone()
                .or_else(|| episode.download_location.clone())
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            Ok(PathBuf::from(path))
        }
        LibraryItemId::Book(_) => {
            let book = state
                .audiobookshelf_book_service
                .find_by_id(&session.library_item_id)?
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            let aggregate = state.audiobookshelf_book_service.hydrate(book)?;
            let first = aggregate
                .audio_files
                .into_iter()
                .next()
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            Ok(PathBuf::from(first.path))
        }
    }
}

fn m3u8_response(body: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    (StatusCode::OK, headers, body).into_response()
}

pub fn get_hls_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(hls_dispatch))
}
