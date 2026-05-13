use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::mapping::book::map_book;
use crate::audiobookshelf_api::socket_io::broadcaster;
use crate::audiobookshelf_api::socket_io::events::{EVENT_ITEM_ADDED, EVENT_ITEM_UPDATED};
use axum::Json;
use axum::extract::{Path, State};
use common_infrastructure::error::CustomError;
use serde_json::{Value, json};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    post,
    path = "/api/libraries/{id}/scan",
    params(("id" = String, Path, description = "Library id to scan")),
    responses((status = 200, description = "Scan report")),
    tag = "audiobookshelf"
)]
pub async fn scan_library(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(library_id): Path<String>,
) -> Result<Json<Value>, CustomError> {
    let report = state.audiobookshelf_scanner.scan_library(&library_id)?;
    if let Ok(Some(library)) = state.audiobookshelf_library_service.find_by_id(&library_id) {
        broadcaster::emit_library_updated(&library);
    }
    // Emit item_added/item_updated per book (audiobookshelf parity:
    // LibraryScanner.libraryItemEmitter('item_added'|'item_updated', item)).
    for result in &report.book_results {
        if let Ok(Some(book)) = state.audiobookshelf_book_service.find_by_id(&result.book_id)
            && let Ok(aggregate) = state.audiobookshelf_book_service.hydrate(book)
        {
            let event = if result.is_new {
                EVENT_ITEM_ADDED
            } else {
                EVENT_ITEM_UPDATED
            };
            broadcaster::emit_item_event(event, map_book(&aggregate));
        }
    }
    Ok(Json(json!({
        "scannedFolders": report.scanned_folders,
        "booksUpserted": report.books_upserted,
        "booksAdded": report.books_added,
        "booksUpdated": report.books_updated,
        "audioFiles": report.audio_files,
        "chapters": report.chapters,
        "errors": report.errors,
    })))
}

pub fn get_scan_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(scan_library))
}
