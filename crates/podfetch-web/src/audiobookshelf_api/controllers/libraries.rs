use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::dto::library::{LibrariesListResponse, LibraryDto};
use axum::Json;
use axum::extract::{Path, State};
use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/api/libraries",
    responses((status = 200, description = "All libraries", body = LibrariesListResponse)),
    tag = "audiobookshelf"
)]
pub async fn list_libraries(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<LibrariesListResponse>, CustomError> {
    let libraries = state.audiobookshelf_library_service.list()?;
    Ok(Json(LibrariesListResponse {
        libraries: libraries.iter().map(LibraryDto::from).collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/libraries/{id}",
    params(("id" = String, Path, description = "Library id")),
    responses(
        (status = 200, description = "Library", body = LibraryDto),
        (status = 404, description = "Not found")
    ),
    tag = "audiobookshelf"
)]
pub async fn get_library(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<LibraryDto>, CustomError> {
    let library = state
        .audiobookshelf_library_service
        .find_by_id(&id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    Ok(Json(LibraryDto::from(&library)))
}

pub fn get_libraries_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_libraries))
        .routes(routes!(get_library))
}
