use crate::app_state::AppState;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::path::trim_from_path;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use podfetch_domain::session::Session;
use crate::device::{self, DeviceControllerError, DevicePost, DeviceResponse};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

fn map_controller_error(error: DeviceControllerError<CustomError>) -> CustomError {
    match error {
        DeviceControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        DeviceControllerError::Service(error) => error,
    }
}

#[utoipa::path(
    post,
    path="/devices/{username}/{deviceid}",
    request_body=DevicePost,
    responses(
        (status = 200, description = "Creates a new device.", body = DeviceResponse),
        (status = 403, description = "Forbidden.")
    ),
    tag="gpodder"
)]
pub async fn post_device(
    State(state): State<AppState>,
    query: Path<(String, String)>,
    Extension(flag): Extension<Session>,
    Json(device_post): Json<DevicePost>,
) -> Result<Json<DeviceResponse>, CustomError> {
    let username = &query.0.0;
    let deviceid = trim_from_path(&query.0.1);
    device::post_device(
        state.device_service.as_ref(),
        &flag.username,
        username,
        deviceid.0,
        device_post,
    )
    .map(Json)
    .map_err(map_controller_error)
}

#[utoipa::path(
    get,
    path="/devices/{username}",
    responses(
        (status = 200, description = "Gets all devices of a user.", body = [DeviceResponse])
    ),
    tag="devices"
)]
pub async fn get_devices_of_user(
    State(state): State<AppState>,
    Path(query): Path<String>,
    Extension(flag): Extension<Session>,
) -> Result<Json<Vec<DeviceResponse>>, CustomError> {
    let query = trim_from_path(&query);
    let user_query = query.0;
    device::get_devices_of_user(state.device_service.as_ref(), &flag.username, user_query)
        .map(Json)
        .map_err(map_controller_error)
}

pub fn get_device_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_devices_of_user))
        .routes(routes!(post_device))
}

