use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::socket_io::broadcaster;
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
    Ok(Json(json!({
        "scannedFolders": report.scanned_folders,
        "booksUpserted": report.books_upserted,
        "audioFiles": report.audio_files,
        "chapters": report.chapters,
        "errors": report.errors,
    })))
}

pub fn get_scan_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(scan_library))
}
