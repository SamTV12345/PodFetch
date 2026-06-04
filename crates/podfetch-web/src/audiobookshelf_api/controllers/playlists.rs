//! Audiobookshelf-compatible playlist endpoints. PodFetch already has a
//! domain `Playlist` (UUID id, name, owner, ordered podcast-episode items)
//! so each handler just wraps `PlaylistRepositoryImpl` and maps to/from the
//! upstream `Playlist.toOldJSONExpanded` byte shape.
//!
//! Books are not yet supported (PodFetch has no audiobook playlists), but
//! the request body grammar mirrors upstream — books can be added later
//! without breaking the API surface.

use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::id_resolution::{resolve_episode, resolve_podcast_library_item};
use crate::audiobookshelf_api::mapping::podcast::{map_episode, map_podcast_without_episodes};
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use common_infrastructure::error::ErrorSeverity::{Debug, Warning};
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::audiobookshelf::library_item_id::{EpisodeId, LibraryItemId};
use podfetch_domain::playlist::{Playlist, PlaylistItem, PlaylistRepository};
use podfetch_persistence::adapters::PlaylistRepositoryImpl;
use podfetch_persistence::db::database;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

fn repo() -> PlaylistRepositoryImpl {
    PlaylistRepositoryImpl::new(database())
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItemInput {
    pub library_item_id: String,
    pub episode_id: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
    #[allow(dead_code)]
    pub library_id: Option<String>,
    #[serde(default)]
    pub items: Vec<PlaylistItemInput>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlaylistRequest {
    pub name: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
    pub items: Option<Vec<PlaylistItemInput>>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchItemsRequest {
    pub items: Vec<PlaylistItemInput>,
}

// ── handlers ────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/playlists",
    responses((status = 200, description = "All playlists owned by the user")),
    tag = "audiobookshelf"
)]
pub async fn list_playlists(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, CustomError> {
    let library_id = library_id(&state)?;
    let playlists = repo().list_by_user(user.id)?;
    let dtos: Vec<Value> = playlists
        .into_iter()
        .map(|p| expand_playlist(&p, &library_id))
        .collect::<Result<_, _>>()?;
    Ok(Json(json!({ "playlists": dtos })))
}

#[utoipa::path(
    get,
    path = "/api/playlists/{id}",
    params(("id" = String, Path)),
    responses((status = 200, description = "A single playlist")),
    tag = "audiobookshelf"
)]
pub async fn get_playlist(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, CustomError> {
    let playlist = ensure_owned(&id, user.id)?;
    let library_id = library_id(&state)?;
    Ok(Json(expand_playlist(&playlist, &library_id)?))
}

#[utoipa::path(
    post,
    path = "/api/playlists",
    request_body = CreatePlaylistRequest,
    responses((status = 200, description = "Created playlist")),
    tag = "audiobookshelf"
)]
pub async fn create_playlist(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(body): Json<CreatePlaylistRequest>,
) -> Result<Json<Value>, CustomError> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(CustomErrorInner::BadRequest("name is required".into(), Warning).into());
    }
    let library_id = library_id(&state)?;
    let playlist = repo().insert_playlist(Playlist {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        user_id: user.id,
    })?;
    let mut seen = HashSet::new();
    for (idx, item) in body.items.iter().enumerate() {
        let episode_id = require_episode_id(item)?;
        if !seen.insert(episode_id) {
            continue;
        }
        repo().insert_playlist_item(PlaylistItem {
            playlist_id: playlist.id.clone(),
            episode: episode_id,
            position: idx as i32,
        })?;
    }
    Ok(Json(expand_playlist(&playlist, &library_id)?))
}

#[utoipa::path(
    patch,
    path = "/api/playlists/{id}",
    params(("id" = String, Path)),
    request_body = UpdatePlaylistRequest,
    responses((status = 200, description = "Updated playlist")),
    tag = "audiobookshelf"
)]
pub async fn update_playlist(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(body): Json<UpdatePlaylistRequest>,
) -> Result<Json<Value>, CustomError> {
    let playlist = ensure_owned(&id, user.id)?;
    let library_id = library_id(&state)?;

    if let Some(name) = body.name.as_deref() {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            repo().update_playlist_name(&playlist.id, user.id, trimmed)?;
        }
    }
    if let Some(items) = body.items.as_ref() {
        repo().delete_items_by_playlist_id(&playlist.id)?;
        for (idx, item) in items.iter().enumerate() {
            let episode_id = require_episode_id(item)?;
            repo().insert_playlist_item(PlaylistItem {
                playlist_id: playlist.id.clone(),
                episode: episode_id,
                position: idx as i32,
            })?;
        }
    }
    let playlist = ensure_owned(&id, user.id)?;
    Ok(Json(expand_playlist(&playlist, &library_id)?))
}

#[utoipa::path(
    delete,
    path = "/api/playlists/{id}",
    params(("id" = String, Path)),
    responses((status = 200, description = "Deleted")),
    tag = "audiobookshelf"
)]
pub async fn delete_playlist(
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, CustomError> {
    ensure_owned(&id, user.id)?;
    repo().delete_items_by_playlist_id(&id)?;
    repo().delete_playlist(&id, user.id)?;
    Ok(Json(json!({ "success": true })))
}

#[utoipa::path(
    post,
    path = "/api/playlists/{id}/item",
    params(("id" = String, Path)),
    request_body = PlaylistItemInput,
    responses((status = 200, description = "Item added")),
    tag = "audiobookshelf"
)]
pub async fn add_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(item): Json<PlaylistItemInput>,
) -> Result<Json<Value>, CustomError> {
    let playlist = ensure_owned(&id, user.id)?;
    let library_id = library_id(&state)?;
    let episode_id = require_episode_id(&item)?;
    let existing = repo().list_items_by_playlist_id(&playlist.id)?;
    if existing.iter().any(|i| i.episode == episode_id) {
        return Err(CustomErrorInner::Conflict("Item already in playlist".into(), Warning).into());
    }
    repo().insert_playlist_item(PlaylistItem {
        playlist_id: playlist.id.clone(),
        episode: episode_id,
        position: existing.len() as i32,
    })?;
    Ok(Json(expand_playlist(&playlist, &library_id)?))
}

#[utoipa::path(
    delete,
    path = "/api/playlists/{id}/item/{libraryItemId}",
    params(("id" = String, Path), ("libraryItemId" = String, Path)),
    responses((status = 200, description = "Item removed")),
    tag = "audiobookshelf"
)]
pub async fn remove_item_by_library_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, library_item_id)): Path<(String, String)>,
) -> Result<Json<Value>, CustomError> {
    remove_items_in(&id, user.id, |items| {
        // Accept legacy `li_pod_{int}` as well as `li_pod_{uuid}`.
        let podcast_id = match resolve_podcast_library_item(&library_item_id) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };
        items
            .iter()
            .filter(|i| {
                PodcastEpisodeService::get_podcast_episode_by_internal_id(i.episode)
                    .ok()
                    .flatten()
                    .map(|e| e.podcast_id == podcast_id.to_string())
                    .unwrap_or(false)
            })
            .map(|i| i.episode)
            .collect()
    })?;
    let playlist = ensure_owned(&id, user.id)?;
    let library_id = library_id(&state)?;
    Ok(Json(expand_playlist(&playlist, &library_id)?))
}

#[utoipa::path(
    delete,
    path = "/api/playlists/{id}/item/{libraryItemId}/{episodeId}",
    params(
        ("id" = String, Path),
        ("libraryItemId" = String, Path),
        ("episodeId" = String, Path)
    ),
    responses((status = 200, description = "Episode removed")),
    tag = "audiobookshelf"
)]
pub async fn remove_item_by_episode(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, _library_item_id, episode_id)): Path<(String, String, String)>,
) -> Result<Json<Value>, CustomError> {
    // Accept legacy `ep_{int}` as well as `ep_{uuid}`.
    let ep = resolve_episode(&episode_id)?;
    remove_items_in(&id, user.id, |_| vec![ep])?;
    let playlist = ensure_owned(&id, user.id)?;
    let library_id = library_id(&state)?;
    Ok(Json(expand_playlist(&playlist, &library_id)?))
}

#[utoipa::path(
    post,
    path = "/api/playlists/{id}/batch/add",
    params(("id" = String, Path)),
    request_body = BatchItemsRequest,
    responses((status = 200, description = "Items added")),
    tag = "audiobookshelf"
)]
pub async fn batch_add(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(body): Json<BatchItemsRequest>,
) -> Result<Json<Value>, CustomError> {
    let playlist = ensure_owned(&id, user.id)?;
    let library_id = library_id(&state)?;
    let mut existing = repo().list_items_by_playlist_id(&playlist.id)?;
    let mut next_pos = existing.iter().map(|i| i.position).max().unwrap_or(-1) + 1;
    for item in &body.items {
        let episode_id = require_episode_id(item)?;
        if existing.iter().any(|i| i.episode == episode_id) {
            continue;
        }
        let new_item = PlaylistItem {
            playlist_id: playlist.id.clone(),
            episode: episode_id,
            position: next_pos,
        };
        repo().insert_playlist_item(new_item.clone())?;
        existing.push(new_item);
        next_pos += 1;
    }
    Ok(Json(expand_playlist(&playlist, &library_id)?))
}

#[utoipa::path(
    post,
    path = "/api/playlists/{id}/batch/remove",
    params(("id" = String, Path)),
    request_body = BatchItemsRequest,
    responses((status = 200, description = "Items removed")),
    tag = "audiobookshelf"
)]
pub async fn batch_remove(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(body): Json<BatchItemsRequest>,
) -> Result<Json<Value>, CustomError> {
    let mut to_remove: Vec<Uuid> = Vec::with_capacity(body.items.len());
    for item in &body.items {
        // Accept legacy `ep_{int}` as well as `ep_{uuid}`; silently skip ids
        // that don't resolve (mirrors the previous `and_then(parse)` skip).
        if let Some(ep) = item.episode_id.as_deref().and_then(|e| resolve_episode(e).ok()) {
            to_remove.push(ep);
        }
    }
    remove_items_in(&id, user.id, |_| to_remove.clone())?;
    let playlist = ensure_owned(&id, user.id)?;
    let library_id = library_id(&state)?;
    Ok(Json(expand_playlist(&playlist, &library_id)?))
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn ensure_owned(playlist_id: &str, user_id: Uuid) -> Result<Playlist, CustomError> {
    repo()
        .find_by_user_and_id(playlist_id, user_id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))
}

fn library_id(state: &AppState) -> Result<String, CustomError> {
    state
        .audiobookshelf_library_service
        .find_default_podcasts_library()?
        .map(|l| l.id)
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))
}

fn require_episode_id(item: &PlaylistItemInput) -> Result<Uuid, CustomError> {
    // Accept legacy `ep_{int}` as well as `ep_{uuid}`.
    item.episode_id
        .as_deref()
        .and_then(|e| resolve_episode(e).ok())
        .ok_or_else(|| {
            CustomError::from(CustomErrorInner::BadRequest(
                "episodeId in 'ep_<id>' form is required (PodFetch does not yet host book playlists)".into(),
                Warning,
            ))
        })
}

/// Resolves the playlist's stored `PlaylistItem` rows, deletes those whose
/// episode ids the selector returns, and renumbers `position`. Centralised
/// because remove-by-libraryItem / remove-by-episode / batch-remove all
/// need the same cleanup.
fn remove_items_in(
    playlist_id: &str,
    user_id: Uuid,
    selector: impl FnOnce(&[PlaylistItem]) -> Vec<Uuid>,
) -> Result<(), CustomError> {
    let playlist = ensure_owned(playlist_id, user_id)?;
    let items = repo().list_items_by_playlist_id(&playlist.id)?;
    let to_remove = selector(&items);
    if to_remove.is_empty() {
        return Ok(());
    }
    for episode_id in &to_remove {
        repo().delete_playlist_item(&playlist.id, *episode_id)?;
    }
    let mut remaining: Vec<PlaylistItem> = items
        .into_iter()
        .filter(|i| !to_remove.contains(&i.episode))
        .collect();
    remaining.sort_by_key(|i| i.position);
    // Diesel's repo lacks an `update_position` — re-insert via delete+add to
    // keep order tight. Cheap because playlists are bounded.
    repo().delete_items_by_playlist_id(&playlist.id)?;
    for (idx, mut item) in remaining.into_iter().enumerate() {
        item.position = idx as i32;
        repo().insert_playlist_item(item)?;
    }
    Ok(())
}

pub fn expand_playlist(playlist: &Playlist, library_id: &str) -> Result<Value, CustomError> {
    let items = repo().list_items_by_playlist_id(&playlist.id)?;
    let now_ms = Utc::now().naive_utc().and_utc().timestamp_millis();
    let mut item_values: Vec<Value> = Vec::with_capacity(items.len());
    for (idx, pmi) in items.iter().enumerate() {
        let Some(episode) = PodcastEpisodeService::get_podcast_episode_by_internal_id(pmi.episode)?
        else {
            continue;
        };
        let episode_uuid = Uuid::parse_str(&episode.id)
            .map_err(|_| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
        let podcast = PodcastService::get_podcast_by_episode_id(episode_uuid)?;
        let podcast_domain: podfetch_domain::podcast::Podcast = podcast.into();
        let episode_domain: podfetch_domain::podcast_episode::PodcastEpisode = episode.into();
        let library_item_id = LibraryItemId::Podcast(podcast_domain.id).as_string();
        let episode_id_str = EpisodeId(episode_domain.id).as_string();
        // The "minified" libraryItem upstream lacks `media.episodes`; we
        // already have a helper that returns the media JSON without
        // episodes — wrap it in the library-item envelope by hand.
        let media_minified = map_podcast_without_episodes(&podcast_domain, library_id);
        let library_item_minified = json!({
            "id": library_item_id,
            "libraryId": library_id,
            "folderId": library_id,
            "mediaType": "podcast",
            "media": media_minified,
            "path": podcast_domain.directory_name,
            "relPath": podcast_domain.directory_name,
            "isFile": false,
            "isMissing": false,
            "isInvalid": false,
        });
        let episode_json = map_episode(
            &episode_domain,
            &library_item_id,
            podcast_domain.id,
            idx as i32 + 1,
        );
        item_values.push(json!({
            "episodeId": episode_id_str,
            "episode": episode_json,
            "libraryItemId": library_item_id,
            "libraryItem": library_item_minified,
        }));
    }

    Ok(json!({
        "id": playlist.id,
        "name": playlist.name,
        "description": Value::Null,
        "libraryId": library_id,
        "userId": playlist.user_id.to_string(),
        // PodFetch's playlists table doesn't store created_at / updated_at;
        // surface 'now' so the mobile-app's sort-by-updated still works.
        "lastUpdate": now_ms,
        "createdAt": now_ms,
        "items": item_values,
    }))
}

pub fn get_playlists_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_playlists))
        .routes(routes!(get_playlist))
        .routes(routes!(create_playlist))
        .routes(routes!(update_playlist))
        .routes(routes!(delete_playlist))
        .routes(routes!(add_item))
        .routes(routes!(remove_item_by_library_item))
        .routes(routes!(remove_item_by_episode))
        .routes(routes!(batch_add))
        .routes(routes!(batch_remove))
}
