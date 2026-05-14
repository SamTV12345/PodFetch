use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::dto::media_progress::MediaProgressDto;
use crate::audiobookshelf_api::dto::user::AbsUserDto;
use crate::audiobookshelf_api::socket_io::broadcaster;
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{Datelike, Utc};
use common_infrastructure::error::CustomError;
use podfetch_domain::audiobookshelf::library_item_id::LibraryItemId;
use podfetch_domain::audiobookshelf::listening_session::ListeningSession;
use podfetch_domain::audiobookshelf::media_progress::MediaProgress;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Deserialize)]
pub struct ListeningSessionsQuery {
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/me",
    responses((status = 200, description = "Current user", body = AbsUserDto)),
    tag = "audiobookshelf"
)]
pub async fn get_me(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<AbsUserDto>, CustomError> {
    let progress = state
        .audiobookshelf_media_progress_service
        .list_for_user(user.id)
        .unwrap_or_default();
    let progress_dtos: Vec<MediaProgressDto> = progress.iter().map(MediaProgressDto::from).collect();

    Ok(Json(AbsUserDto::from_user(&user, progress_dtos)))
}

#[utoipa::path(
    get,
    path = "/api/me/listening-sessions",
    responses((status = 200, description = "User's listening session history")),
    tag = "audiobookshelf"
)]
pub async fn list_listening_sessions(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<ListeningSessionsQuery>,
) -> Result<Json<Value>, CustomError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 500);
    let sessions = state
        .audiobookshelf_listening_session_service
        .list_for_user(user.id, limit)?;
    let total = sessions.len() as i64;
    let entries: Vec<Value> = sessions.iter().map(session_to_value).collect();
    Ok(Json(json!({
        "total": total,
        "limit": limit,
        "page": 0,
        "numPages": 1,
        "itemsPerPage": limit,
        "sessions": entries,
    })))
}

/// Audiobookshelf-shaped serialisation for one listening-session row.
/// Includes `date`, `dayOfWeek` and a synthetic `mediaMetadata` snapshot
/// because upstream's stats aggregator + Vue dashboard read them off
/// each session, and PodFetch doesn't store them as separate columns.
fn session_to_value(s: &ListeningSession) -> Value {
    let started = s.started_at.and_utc();
    let date_str = started.format("%Y-%m-%d").to_string();
    let day_of_week = match started.weekday() {
        chrono::Weekday::Mon => "Monday",
        chrono::Weekday::Tue => "Tuesday",
        chrono::Weekday::Wed => "Wednesday",
        chrono::Weekday::Thu => "Thursday",
        chrono::Weekday::Fri => "Friday",
        chrono::Weekday::Sat => "Saturday",
        chrono::Weekday::Sun => "Sunday",
    };
    json!({
        "id": s.id,
        "userId": s.user_id.to_string(),
        "libraryId": s.library_id,
        "libraryItemId": s.library_item_id,
        "episodeId": s.episode_id,
        "mediaType": s.media_type,
        "playMethod": s.play_method,
        "duration": s.duration,
        "currentTime": s.current_time,
        "timeListening": s.time_listening,
        "startedAt": s.started_at.and_utc().timestamp_millis(),
        "updatedAt": s.updated_at.and_utc().timestamp_millis(),
        "lastUpdate": s.updated_at.and_utc().timestamp_millis(),
        "displayTitle": s.display_title,
        "displayAuthor": s.display_author,
        "coverPath": s.cover_path,
        "date": date_str,
        "dayOfWeek": day_of_week,
        "mediaMetadata": json!({
            "title": s.display_title.clone().unwrap_or_default(),
            "author": s.display_author.clone().unwrap_or_default(),
            "coverPath": s.cover_path,
        }),
    })
}

/// Audiobookshelf-compatible progress update payload. Mobile apps send a
/// subset of these per PATCH; missing fields keep their previous values.
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProgressUpdatePayload {
    pub duration: Option<f64>,
    pub progress: Option<f64>,
    pub current_time: Option<f64>,
    pub is_finished: Option<bool>,
    pub hide_from_continue_listening: Option<bool>,
    #[allow(dead_code)]
    pub ebook_location: Option<String>,
    #[allow(dead_code)]
    pub ebook_progress: Option<f64>,
}

#[utoipa::path(
    patch,
    path = "/api/me/progress/{libraryItemId}",
    params(("libraryItemId" = String, Path)),
    request_body = ProgressUpdatePayload,
    responses((status = 200, description = "Progress upserted")),
    tag = "audiobookshelf"
)]
pub async fn patch_progress_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(library_item_id): Path<String>,
    Json(payload): Json<ProgressUpdatePayload>,
) -> Result<Json<Value>, CustomError> {
    upsert_progress_from_payload(&state, &user, &library_item_id, None, payload)?;
    Ok(Json(json!({ "success": true })))
}

#[utoipa::path(
    patch,
    path = "/api/me/progress/{libraryItemId}/{episodeId}",
    params(
        ("libraryItemId" = String, Path),
        ("episodeId" = String, Path)
    ),
    request_body = ProgressUpdatePayload,
    responses((status = 200, description = "Episode progress upserted")),
    tag = "audiobookshelf"
)]
pub async fn patch_progress_item_episode(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((library_item_id, episode_id)): Path<(String, String)>,
    Json(payload): Json<ProgressUpdatePayload>,
) -> Result<Json<Value>, CustomError> {
    upsert_progress_from_payload(
        &state,
        &user,
        &library_item_id,
        Some(episode_id),
        payload,
    )?;
    Ok(Json(json!({ "success": true })))
}

#[utoipa::path(
    patch,
    path = "/api/me/progress/batch/update",
    request_body = Vec<Value>,
    responses(
        (status = 200, description = "Batch progress upserted"),
        (status = 400, description = "Missing payload")
    ),
    tag = "audiobookshelf"
)]
pub async fn patch_progress_batch(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payloads): Json<Vec<Value>>,
) -> Result<Json<Value>, CustomError> {
    if payloads.is_empty() {
        return Err(common_infrastructure::error::CustomErrorInner::Conflict(
            "Missing request payload".to_string(),
            common_infrastructure::error::ErrorSeverity::Warning,
        )
        .into());
    }
    for entry in &payloads {
        let Some(item_id) = entry.get("libraryItemId").and_then(|v| v.as_str()) else {
            continue;
        };
        let episode_id = entry
            .get("episodeId")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let payload = ProgressUpdatePayload {
            duration: entry.get("duration").and_then(|v| v.as_f64()),
            progress: entry.get("progress").and_then(|v| v.as_f64()),
            current_time: entry.get("currentTime").and_then(|v| v.as_f64()),
            is_finished: entry.get("isFinished").and_then(|v| v.as_bool()),
            hide_from_continue_listening: entry
                .get("hideFromContinueListening")
                .and_then(|v| v.as_bool()),
            ebook_location: entry
                .get("ebookLocation")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            ebook_progress: entry.get("ebookProgress").and_then(|v| v.as_f64()),
        };
        let _ = upsert_progress_from_payload(&state, &user, item_id, episode_id, payload);
    }
    Ok(Json(json!({ "success": true })))
}

fn upsert_progress_from_payload(
    state: &AppState,
    user: &podfetch_domain::user::User,
    library_item_id: &str,
    episode_id: Option<String>,
    payload: ProgressUpdatePayload,
) -> Result<MediaProgress, CustomError> {
    // Validate the library_item_id shape so we don't store garbage.
    let media_type = LibraryItemId::parse(library_item_id)
        .map(|id| id.media_type_str().to_string())
        .unwrap_or_else(|| "podcast".to_string());

    let now = Utc::now().naive_utc();
    let existing = state.audiobookshelf_media_progress_service.find(
        user.id,
        library_item_id,
        episode_id.as_deref(),
    )?;
    let started_at = existing.as_ref().map(|p| p.started_at).unwrap_or(now);
    let was_finished = existing.as_ref().is_some_and(|p| p.is_finished);
    let prev_current = existing.as_ref().map(|p| p.current_time).unwrap_or(0.0);
    let prev_duration = existing.as_ref().map(|p| p.duration).unwrap_or(0.0);
    let duration = payload.duration.unwrap_or(prev_duration);
    let current_time = payload.current_time.unwrap_or(prev_current);
    let progress = payload.progress.unwrap_or_else(|| {
        if duration > 0.0 {
            (current_time / duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    });
    let is_finished = payload
        .is_finished
        .unwrap_or(duration > 0.0 && current_time / duration > 0.95);
    let finished_at = if is_finished && !was_finished {
        Some(now)
    } else {
        existing.as_ref().and_then(|p| p.finished_at)
    };
    let hide = payload.hide_from_continue_listening.unwrap_or(
        existing
            .as_ref()
            .map(|p| p.hide_from_continue_listening)
            .unwrap_or(false),
    );
    let progress_id = MediaProgress::compose_id(library_item_id, episode_id.as_deref());
    let updated = state.audiobookshelf_media_progress_service.upsert(MediaProgress {
        id: progress_id,
        user_id: user.id,
        library_item_id: library_item_id.to_string(),
        episode_id,
        media_type,
        duration,
        current_time,
        progress,
        is_finished,
        hide_from_continue_listening: hide,
        last_update: now,
        started_at,
        finished_at,
    })?;
    // Mirror upstream: emit user_updated to all of the user's sockets after
    // a progress write. Mobile apps refresh the continue-listening list off
    // this event.
    let all_progress = state
        .audiobookshelf_media_progress_service
        .list_for_user(user.id)
        .unwrap_or_default();
    broadcaster::emit_user_updated(user, &all_progress);
    Ok(updated)
}

/// Mirrors upstream `GET /api/me/listening-stats`. Aggregates the closed
/// listening sessions into the totals the web/mobile dashboards graph:
/// `totalTime`, per-day, per-weekday and per-libraryItem buckets, plus
/// the 10 most recent sessions. Exact key set is fixed by upstream's
/// ApiRouter.getUserListeningStatsHelpers and Vue dashboard reads each
/// key without a default.
#[utoipa::path(
    get,
    path = "/api/me/listening-stats",
    responses((status = 200, description = "Aggregated listening stats for the user")),
    tag = "audiobookshelf"
)]
pub async fn get_listening_stats(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, CustomError> {
    // Audit cap: the dashboard graphs the lifetime of stats; pull every
    // session up to a generous safety limit so totals are correct.
    let sessions = state
        .audiobookshelf_listening_session_service
        .list_for_user(user.id, 100_000)?;

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let mut total_time: f64 = 0.0;
    let mut today_time: f64 = 0.0;
    let mut days: Map<String, Value> = Map::new();
    let mut day_of_week: Map<String, Value> = Map::new();
    let mut items: Map<String, Value> = Map::new();

    for s in &sessions {
        let listening = s.time_listening.max(0.0);
        let started = s.started_at.and_utc();
        let date_str = started.format("%Y-%m-%d").to_string();
        let weekday = match started.weekday() {
            chrono::Weekday::Mon => "Monday",
            chrono::Weekday::Tue => "Tuesday",
            chrono::Weekday::Wed => "Wednesday",
            chrono::Weekday::Thu => "Thursday",
            chrono::Weekday::Fri => "Friday",
            chrono::Weekday::Sat => "Saturday",
            chrono::Weekday::Sun => "Sunday",
        };
        let last_update_ms = s.updated_at.and_utc().timestamp_millis();

        if listening > 0.0 {
            let day = days
                .get(&date_str)
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
                + listening;
            days.insert(date_str.clone(), json!(day));
            if date_str == today {
                today_time += listening;
            }
        }
        let dow = day_of_week
            .get(weekday)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            + listening;
        day_of_week.insert(weekday.to_string(), json!(dow));

        let entry = items
            .entry(s.library_item_id.clone())
            .or_insert_with(|| {
                json!({
                    "id": s.library_item_id,
                    "timeListening": 0.0,
                    "mediaMetadata": {
                        "title": s.display_title.clone().unwrap_or_default(),
                        "author": s.display_author.clone().unwrap_or_default(),
                        "coverPath": s.cover_path,
                    },
                    "lastUpdate": last_update_ms,
                })
            });
        if let Some(obj) = entry.as_object_mut() {
            let prev = obj
                .get("timeListening")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            obj.insert("timeListening".to_string(), json!(prev + listening));
            let prev_update = obj
                .get("lastUpdate")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            if last_update_ms > prev_update {
                obj.insert("lastUpdate".to_string(), json!(last_update_ms));
            }
        }
        total_time += listening;
    }

    let recent: Vec<Value> = sessions.iter().take(10).map(session_to_value).collect();
    Ok(Json(json!({
        "totalTime": total_time,
        "items": items,
        "days": days,
        "dayOfWeek": day_of_week,
        "today": today_time,
        "recentSessions": recent,
    })))
}

/// Mirrors upstream `GET /api/me/items-in-progress`. The mobile apps
/// poll this for the "Continue Listening" shelf. Returns a flat list of
/// progress entries (we already build these for `/api/me`).
#[utoipa::path(
    get,
    path = "/api/me/items-in-progress",
    responses((status = 200, description = "Library items in progress")),
    tag = "audiobookshelf"
)]
pub async fn list_items_in_progress(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, CustomError> {
    let progress = state
        .audiobookshelf_media_progress_service
        .list_for_user(user.id)
        .unwrap_or_default();
    // Only entries with progress > 0 and not finished count for the shelf.
    let in_progress: Vec<MediaProgressDto> = progress
        .iter()
        .filter(|p| p.current_time > 0.0 && !p.is_finished)
        .map(MediaProgressDto::from)
        .collect();
    Ok(Json(json!({
        "libraryItems": in_progress,
    })))
}

pub fn get_me_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_me))
        .routes(routes!(list_listening_sessions))
        .routes(routes!(list_items_in_progress))
        .routes(routes!(get_listening_stats))
        .routes(routes!(patch_progress_item))
        .routes(routes!(patch_progress_item_episode))
        .routes(routes!(patch_progress_batch))
}
