use crate::app_state::AppState;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::path::trim_from_path;
use common_infrastructure::time::get_current_timestamp;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use podfetch_domain::session::Session;
use podfetch_web::gpodder::{GpodderControllerError, ensure_session_user};
use podfetch_web::subscription::{
    SubscriptionChangesToClient, SubscriptionPostResponse, SubscriptionRetrieveRequest,
    SubscriptionUpdateRequest, build_opml, to_client_changes,
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

fn map_gpodder_error(error: GpodderControllerError<CustomError>) -> CustomError {
    match error {
        GpodderControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        GpodderControllerError::BadRequest(message) => {
            CustomErrorInner::BadRequest(message, Warning).into()
        }
        GpodderControllerError::Service(error) => error,
    }
}

#[utoipa::path(
    get,
    path="/subscriptions/{username}/{deviceid}",
    request_body=SubscriptionRetrieveRequest,
    responses(
        (status = 200, description = "Gets all subscriptions for a device"),
        (status = 403, description = "Forbidden")
    ),
    tag="gpodder"
)]
pub async fn get_subscriptions(
    State(state): State<AppState>,
    Path(paths): Path<(String, String)>,
    Extension(flag): Extension<Session>,
    Query(query): Query<SubscriptionRetrieveRequest>,
) -> Result<Json<SubscriptionChangesToClient>, CustomError> {
    let username = paths.clone().0;
    let deviceid = trim_from_path(&paths.1);
    ensure_session_user::<CustomError>(&flag.username, &username).map_err(map_gpodder_error)?;

    state
        .subscription_service
        .get_device_subscriptions(deviceid.0, &username, query.since)
        .map(Json)
}

#[utoipa::path(
    get,
    path="/subscriptions/{username}",
    request_body=SubscriptionRetrieveRequest,
    responses(
        (status = 200, description = "Gets all subscriptions"),
        (status = 403, description = "Forbidden")
    ),
    tag="gpodder"
)]
pub async fn get_subscriptions_all(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(flag): Extension<Session>,
    Query(query): Query<SubscriptionRetrieveRequest>,
) -> Result<impl IntoResponse, CustomError> {
    let username = trim_from_path(&username);
    ensure_session_user::<CustomError>(&flag.username, username.0).map_err(map_gpodder_error)?;

    let changes = state
        .subscription_service
        .get_user_subscriptions(&flag.username, query.since)?;

    if username.1 == "opml" {
        Ok(build_opml(&changes.add)
            .to_string()
            .unwrap()
            .into_response())
    } else {
        Ok(Json(to_client_changes(changes)).into_response())
    }
}

#[utoipa::path(
    post,
    path="/subscriptions/{username}/{deviceid}",
    request_body=SubscriptionUpdateRequest,
    responses(
        (status = 200, description = "Uploads subscription changes"),
        (status = 403, description = "Forbidden")
    ),
    tag="gpodder"
)]
pub async fn upload_subscription_changes(
    State(state): State<AppState>,
    Extension(flag): Extension<Session>,
    paths: Path<(String, String)>,
    upload_request: Json<SubscriptionUpdateRequest>,
) -> Result<Json<SubscriptionPostResponse>, CustomError> {
    let username = paths.clone().0;
    let deviceid = trim_from_path(&paths.1);
    ensure_session_user::<CustomError>(&flag.username, &username).map_err(map_gpodder_error)?;

    let update_urls =
        state
            .subscription_service
            .update_subscriptions(deviceid.0, &username, upload_request.0)?;

    Ok(Json(SubscriptionPostResponse {
        update_urls,
        timestamp: get_current_timestamp(),
    }))
}

pub fn get_subscription_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(upload_subscription_changes))
        .routes(routes!(get_subscriptions_all))
        .routes(routes!(get_subscriptions))
}

