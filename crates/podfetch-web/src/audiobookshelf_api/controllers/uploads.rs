//! Manual audiobook upload. Drops files into the target library's first
//! `folder_paths[0]` under `<libraryFolder>/<authorOrFolder>/<title>/` and
//! kicks off a single-folder scan.
//!
//! Field names mirror the audiobookshelf-web admin uploader so any client
//! that already speaks audiobookshelf can drop books in unchanged.

use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::mapping::book::map_book;
use crate::audiobookshelf_api::socket_io::broadcaster;
use crate::audiobookshelf_api::socket_io::events::{EVENT_ITEM_ADDED, EVENT_ITEM_UPDATED};
use axum::Json;
use axum::extract::{Multipart, State};
use common_infrastructure::error::ErrorSeverity::{Debug, Warning};
use common_infrastructure::error::{CustomError, CustomErrorInner};
use serde_json::{Value, json};
use std::path::PathBuf;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    post,
    path = "/api/upload",
    responses(
        (status = 200, description = "Upload accepted; scan result returned"),
        (status = 400, description = "Bad multipart request")
    ),
    tag = "audiobookshelf"
)]
pub async fn upload(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<Value>, CustomError> {
    let mut library_id: Option<String> = None;
    let mut author: Option<String> = None;
    let mut title: Option<String> = None;
    // (filename, bytes)
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| bad_request(format!("multipart parse error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "library" | "libraryId" => {
                library_id = Some(field.text().await.map_err(|e| bad_request(e.to_string()))?);
            }
            "author" => {
                author = Some(field.text().await.map_err(|e| bad_request(e.to_string()))?);
            }
            "title" => {
                title = Some(field.text().await.map_err(|e| bad_request(e.to_string()))?);
            }
            _ => {
                let filename = field.file_name().map(sanitize_filename).unwrap_or_default();
                if filename.is_empty() {
                    continue;
                }
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| bad_request(format!("file read failed: {e}")))?
                    .to_vec();
                files.push((filename, bytes));
            }
        }
    }

    let library_id = library_id
        .filter(|s| !s.is_empty())
        .ok_or_else(|| bad_request("missing library / libraryId field".to_string()))?;
    if files.is_empty() {
        return Err(bad_request("no files in upload".to_string()));
    }
    let library = state
        .audiobookshelf_library_service
        .find_by_id(&library_id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    let folder_root = library
        .folder_paths
        .first()
        .cloned()
        .ok_or_else(|| bad_request("library has no folder_paths configured".to_string()))?;

    let title = title.filter(|s| !s.is_empty()).unwrap_or_else(|| {
        files
            .first()
            .map(|(name, _)| strip_ext(name))
            .unwrap_or_else(|| "Untitled".to_string())
    });
    let target_dir = match author.as_deref().filter(|s| !s.is_empty()) {
        Some(author) => PathBuf::from(&folder_root)
            .join(sanitize_filename(author))
            .join(sanitize_filename(&title)),
        None => PathBuf::from(&folder_root).join(sanitize_filename(&title)),
    };
    std::fs::create_dir_all(&target_dir).map_err(|e| {
        CustomError::from(CustomErrorInner::Conflict(
            format!("could not create target directory: {e}"),
            Warning,
        ))
    })?;

    for (filename, bytes) in &files {
        let dest = target_dir.join(filename);
        std::fs::write(&dest, bytes).map_err(|e| {
            CustomError::from(CustomErrorInner::Conflict(
                format!("could not write {}: {e}", dest.display()),
                Warning,
            ))
        })?;
    }

    let scan_result = state
        .audiobookshelf_scanner
        .scan_book_folder(&library.id, &target_dir);

    let mut book_id: Option<String> = None;
    let mut audio_files = 0_usize;
    let mut chapters = 0_usize;
    let mut error: Option<String> = None;
    match scan_result {
        Ok(stats) => {
            book_id = Some(stats.book_id.clone());
            audio_files = stats.audio_files;
            chapters = stats.chapters;
            if let Ok(Some(book)) = state.audiobookshelf_book_service.find_by_id(&stats.book_id)
                && let Ok(aggregate) = state.audiobookshelf_book_service.hydrate(book)
            {
                let event = if stats.is_new {
                    EVENT_ITEM_ADDED
                } else {
                    EVENT_ITEM_UPDATED
                };
                broadcaster::emit_item_event(event, map_book(&aggregate));
            }
        }
        Err(e) => {
            error = Some(e.to_string());
        }
    }

    Ok(Json(json!({
        "libraryId": library.id,
        "targetDir": target_dir.to_string_lossy(),
        "uploadedFiles": files.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
        "bookId": book_id,
        "audioFiles": audio_files,
        "chapters": chapters,
        "scanError": error,
    })))
}

fn bad_request(msg: String) -> CustomError {
    CustomError::from(CustomErrorInner::Conflict(msg, Warning))
}

fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            c if c.is_control() => '_',
            other => other,
        })
        .collect::<String>()
        .trim_matches(|c: char| c == '.' || c.is_whitespace())
        .to_string()
}

fn strip_ext(filename: &str) -> String {
    PathBuf::from(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| filename.to_string())
}

pub fn get_upload_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(upload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_filename_strips_path_separators_and_control() {
        assert_eq!(
            sanitize_filename("safe-file.mp3"),
            "safe-file.mp3".to_string()
        );
        assert_eq!(
            sanitize_filename("ev/il\\path:name?.txt"),
            "ev_il_path_name_.txt".to_string()
        );
        assert_eq!(sanitize_filename(" .hidden. "), "hidden".to_string());
    }

    #[test]
    fn strip_ext_drops_last_suffix() {
        assert_eq!(strip_ext("foo.mp3"), "foo".to_string());
        assert_eq!(strip_ext("foo.bar.m4b"), "foo.bar".to_string());
        assert_eq!(strip_ext("noext"), "noext".to_string());
    }
}
