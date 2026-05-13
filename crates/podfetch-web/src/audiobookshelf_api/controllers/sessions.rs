//! Playback session lifecycle: start, sync, close.

use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::dto::playback_session::{
    AudioTrackMetadataDto, PlayRequestBody, PlaybackAudioTrackDto, PlaybackSessionDto,
    SyncRequestBody,
};
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::Utc;
use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::audiobookshelf::library_item_id::{EpisodeId, LibraryItemId};
use podfetch_domain::audiobookshelf::listening_session::ListeningSession;
use podfetch_domain::audiobookshelf::media_progress::MediaProgress;
use podfetch_domain::audiobookshelf::playback_session::{PlayMethod, PlaybackSession};
use serde::Deserialize;
use std::path::Path as StdPath;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[derive(Deserialize)]
#[allow(dead_code)]
struct EmptyBody {}

#[utoipa::path(
    post,
    path = "/api/items/{id}/play",
    params(("id" = String, Path, description = "Library item id")),
    request_body = PlayRequestBody,
    responses((status = 200, description = "Playback session", body = PlaybackSessionDto)),
    tag = "audiobookshelf"
)]
pub async fn play_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(item_id): Path<String>,
    body: Option<Json<PlayRequestBody>>,
) -> Result<Json<PlaybackSessionDto>, CustomError> {
    start_session(state, user, &item_id, None, body.map(|b| b.0)).await
}

#[utoipa::path(
    post,
    path = "/api/items/{id}/play/{episodeId}",
    params(
        ("id" = String, Path, description = "Library item id"),
        ("episodeId" = String, Path, description = "Episode id")
    ),
    request_body = PlayRequestBody,
    responses((status = 200, description = "Playback session", body = PlaybackSessionDto)),
    tag = "audiobookshelf"
)]
pub async fn play_item_episode(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((item_id, episode_id)): Path<(String, String)>,
    body: Option<Json<PlayRequestBody>>,
) -> Result<Json<PlaybackSessionDto>, CustomError> {
    start_session(state, user, &item_id, Some(&episode_id), body.map(|b| b.0)).await
}

async fn start_session(
    state: AppState,
    user: podfetch_domain::user::User,
    item_id: &str,
    episode_id: Option<&str>,
    _body: Option<PlayRequestBody>,
) -> Result<Json<PlaybackSessionDto>, CustomError> {
    let parsed_item = LibraryItemId::parse(item_id)
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;

    if let LibraryItemId::Book(_) = parsed_item {
        return start_book_session(state, user, item_id).await;
    }
    let LibraryItemId::Podcast(podcast_id) = parsed_item else {
        return Err(CustomErrorInner::NotFound(Debug).into());
    };
    let ep_id_value = episode_id
        .and_then(EpisodeId::parse)
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;

    let podcast: podfetch_domain::podcast::Podcast = PodcastService::get_podcast(podcast_id)?.into();
    let episodes: Vec<podfetch_domain::podcast_episode::PodcastEpisode> =
        PodcastEpisodeService::get_episodes_by_podcast_id(podcast_id)?
            .into_iter()
            .map(Into::into)
            .collect();
    let episode = episodes
        .iter()
        .find(|e| e.id == ep_id_value.0)
        .cloned()
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;

    let library = state
        .audiobookshelf_library_service
        .find_default_podcasts_library()?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;

    let now = Utc::now().naive_utc();
    let session_id = format!("play_{}", Uuid::new_v4().simple());
    let duration = episode.total_time as f64;

    let local_path = episode
        .file_episode_path
        .clone()
        .or_else(|| episode.download_location.clone())
        .unwrap_or_default();
    let filename = StdPath::new(&local_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let ext = StdPath::new(&local_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime_type = mime_for_ext(&ext);
    let codec = codec_for_ext(&ext);

    // Phase A: always direct streaming. HLS decision moves to playback_session_service in Phase C.
    let play_method = PlayMethod::Direct;
    let content_url = format!("/public/session/{session_id}/track/0");

    let session = PlaybackSession {
        id: session_id.clone(),
        user_id: user.id,
        library_id: Some(library.id.clone()),
        library_item_id: item_id.to_string(),
        episode_id: Some(EpisodeId(episode.id).as_string()),
        media_type: "podcast".to_string(),
        play_method,
        current_time: 0.0,
        duration,
        started_at: now,
        updated_at: now,
        finished_at: None,
        time_listening_total: 0.0,
        display_title: Some(episode.name.clone()),
        display_author: podcast.author.clone(),
        cover_path: Some(format!("/api/items/{item_id}/cover")),
        media_metadata_json: None,
        device_info_json: None,
    };
    let session = state
        .audiobookshelf_playback_session_service
        .create(session)?;

    let progress = state
        .audiobookshelf_media_progress_service
        .find(user.id, item_id, Some(&EpisodeId(episode.id).as_string()))?;
    let mut session_with_progress = session.clone();
    if let Some(p) = progress.as_ref() {
        session_with_progress.current_time = p.current_time;
    }

    let audio_tracks = vec![PlaybackAudioTrackDto {
        index: 0,
        start_offset: 0.0,
        duration,
        title: episode.name.clone(),
        content_url,
        mime_type,
        codec,
        metadata: AudioTrackMetadataDto {
            filename,
            ext,
            path: local_path.clone(),
            rel_path: local_path,
        },
    }];

    Ok(Json(PlaybackSessionDto::from_domain(
        &session_with_progress,
        audio_tracks,
    )))
}

#[utoipa::path(
    get,
    path = "/api/session/{id}",
    params(("id" = String, Path)),
    responses((status = 200, description = "Open session", body = PlaybackSessionDto)),
    tag = "audiobookshelf"
)]
pub async fn get_session(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<PlaybackSessionDto>, CustomError> {
    let session = state
        .audiobookshelf_playback_session_service
        .find_by_id(&id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    if session.user_id != user.id {
        return Err(CustomErrorInner::Forbidden(Debug).into());
    }
    Ok(Json(PlaybackSessionDto::from_domain(&session, Vec::new())))
}

#[utoipa::path(
    post,
    path = "/api/session/{id}/sync",
    params(("id" = String, Path)),
    request_body = SyncRequestBody,
    responses((status = 200, description = "Synced")),
    tag = "audiobookshelf"
)]
pub async fn sync_session(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(body): Json<SyncRequestBody>,
) -> Result<StatusCode, CustomError> {
    let mut session = state
        .audiobookshelf_playback_session_service
        .find_by_id(&id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    if session.user_id != user.id {
        return Err(CustomErrorInner::Forbidden(Debug).into());
    }

    let now = Utc::now().naive_utc();
    session.current_time = body.current_time;
    session.time_listening_total += body.time_listened.max(0.0);
    if body.duration > 0.0 {
        session.duration = body.duration;
    }
    session.updated_at = now;
    state
        .audiobookshelf_playback_session_service
        .update(session.clone())?;

    upsert_progress(&state, &session, false)?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/session/{id}/close",
    params(("id" = String, Path)),
    request_body(content = Option<SyncRequestBody>, description = "Optional final sync body"),
    responses((status = 200, description = "Closed")),
    tag = "audiobookshelf"
)]
pub async fn close_session(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    body: Option<Json<SyncRequestBody>>,
) -> Result<StatusCode, CustomError> {
    let mut session = state
        .audiobookshelf_playback_session_service
        .find_by_id(&id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    if session.user_id != user.id {
        return Err(CustomErrorInner::Forbidden(Debug).into());
    }

    let now = Utc::now().naive_utc();
    if let Some(Json(body)) = body {
        session.current_time = body.current_time;
        session.time_listening_total += body.time_listened.max(0.0);
        if body.duration > 0.0 {
            session.duration = body.duration;
        }
    }
    session.updated_at = now;
    session.finished_at = Some(now);

    let is_finished = session.duration > 0.0 && session.current_time / session.duration > 0.95;
    upsert_progress(&state, &session, is_finished)?;

    // Persist into listening-sessions history.
    let listening = ListeningSession {
        id: session.id.clone(),
        user_id: session.user_id,
        library_id: session.library_id.clone(),
        library_item_id: session.library_item_id.clone(),
        episode_id: session.episode_id.clone(),
        media_type: session.media_type.clone(),
        duration: session.duration,
        current_time: session.current_time,
        time_listening: session.time_listening_total,
        play_method: session.play_method.as_i32(),
        started_at: session.started_at,
        updated_at: now,
        display_title: session.display_title.clone(),
        display_author: session.display_author.clone(),
        cover_path: session.cover_path.clone(),
    };
    let _ = state
        .audiobookshelf_listening_session_service
        .create(listening);

    let _ = state
        .audiobookshelf_playback_session_service
        .delete(&session.id);
    Ok(StatusCode::OK)
}

async fn start_book_session(
    state: AppState,
    user: podfetch_domain::user::User,
    item_id: &str,
) -> Result<Json<PlaybackSessionDto>, CustomError> {
    let book = state
        .audiobookshelf_book_service
        .find_by_id(item_id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    let aggregate = state.audiobookshelf_book_service.hydrate(book)?;
    if aggregate.audio_files.is_empty() {
        return Err(CustomErrorInner::NotFound(Debug).into());
    }

    let now = Utc::now().naive_utc();
    let session_id = format!("play_{}", Uuid::new_v4().simple());
    let session = PlaybackSession {
        id: session_id.clone(),
        user_id: user.id,
        library_id: Some(aggregate.book.library_id.clone()),
        library_item_id: item_id.to_string(),
        episode_id: None,
        media_type: "book".to_string(),
        play_method: PlayMethod::Direct,
        current_time: 0.0,
        duration: aggregate.book.duration_seconds,
        started_at: now,
        updated_at: now,
        finished_at: None,
        time_listening_total: 0.0,
        display_title: Some(aggregate.book.title.clone()),
        display_author: aggregate.authors.first().map(|a| a.name.clone()),
        cover_path: Some(format!("/api/items/{item_id}/cover")),
        media_metadata_json: None,
        device_info_json: None,
    };
    let session = state
        .audiobookshelf_playback_session_service
        .create(session)?;

    // Restore last progress
    let progress = state
        .audiobookshelf_media_progress_service
        .find(user.id, item_id, None)?;
    let mut session_with_progress = session.clone();
    if let Some(p) = progress.as_ref() {
        session_with_progress.current_time = p.current_time;
    }

    // Build one audio track per BookAudioFile
    let audio_tracks: Vec<PlaybackAudioTrackDto> = aggregate
        .audio_files
        .iter()
        .scan(0.0_f64, |start_offset, af| {
            let offset = *start_offset;
            *start_offset += af.duration;
            let filename = StdPath::new(&af.path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            Some(PlaybackAudioTrackDto {
                index: af.idx,
                start_offset: offset,
                duration: af.duration,
                title: filename.clone(),
                content_url: format!("/public/session/{}/track/{}", session.id, af.idx),
                mime_type: af.mime_type.clone(),
                codec: af.codec.clone(),
                metadata: AudioTrackMetadataDto {
                    filename,
                    ext: af.ext.clone(),
                    path: af.path.clone(),
                    rel_path: af.relative_path.clone(),
                },
            })
        })
        .collect();

    Ok(Json(PlaybackSessionDto::from_domain(
        &session_with_progress,
        audio_tracks,
    )))
}

fn upsert_progress(
    state: &AppState,
    session: &PlaybackSession,
    force_finished: bool,
) -> Result<(), CustomError> {
    let now = Utc::now().naive_utc();
    let progress_id = MediaProgress::compose_id(
        &session.library_item_id,
        session.episode_id.as_deref(),
    );
    let existing = state.audiobookshelf_media_progress_service.find(
        session.user_id,
        &session.library_item_id,
        session.episode_id.as_deref(),
    )?;
    let started_at = existing.as_ref().map(|p| p.started_at).unwrap_or(now);
    let was_finished = existing.as_ref().is_some_and(|p| p.is_finished);
    let is_finished = force_finished
        || (session.duration > 0.0 && session.current_time / session.duration > 0.95);
    let finished_at = if is_finished && !was_finished {
        Some(now)
    } else {
        existing.as_ref().and_then(|p| p.finished_at)
    };
    let progress_value = if session.duration > 0.0 {
        (session.current_time / session.duration).clamp(0.0, 1.0)
    } else {
        0.0
    };
    state.audiobookshelf_media_progress_service.upsert(MediaProgress {
        id: progress_id,
        user_id: session.user_id,
        library_item_id: session.library_item_id.clone(),
        episode_id: session.episode_id.clone(),
        media_type: session.media_type.clone(),
        duration: session.duration,
        current_time: session.current_time,
        progress: progress_value,
        is_finished,
        hide_from_continue_listening: false,
        last_update: now,
        started_at,
        finished_at,
    })?;
    Ok(())
}

fn mime_for_ext(ext: &str) -> String {
    match ext {
        "mp3" => "audio/mpeg",
        "m4a" | "m4b" | "mp4" | "aac" => "audio/mp4",
        "flac" => "audio/flac",
        "opus" | "ogg" | "oga" => "audio/ogg",
        "wav" => "audio/wav",
        "webm" | "webma" => "audio/webm",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn codec_for_ext(ext: &str) -> String {
    match ext {
        "mp3" => "mp3",
        "m4a" | "m4b" | "mp4" | "aac" => "aac",
        "flac" => "flac",
        "opus" => "opus",
        "ogg" | "oga" => "vorbis",
        "wav" => "pcm_s16le",
        _ => "unknown",
    }
    .to_string()
}

pub fn get_sessions_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(play_item))
        .routes(routes!(play_item_episode))
        .routes(routes!(get_session))
        .routes(routes!(sync_session))
        .routes(routes!(close_session))
}
