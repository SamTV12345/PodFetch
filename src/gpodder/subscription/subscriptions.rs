use crate::models::session::Session;
use crate::models::subscription::SubscriptionChangesToClient;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::gpodder_trimmer::trim_from_path;
use crate::utils::time::get_current_timestamp;
use axum::extract::{Path, Query};
use axum::{Extension, Json};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct SubscriptionRetrieveRequest {
    pub since: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct SubscriptionUpdateRequest {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct SubscriptionPostResponse {
    pub timestamp: i64,
    pub update_urls: Vec<Vec<String>>,
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
    Path(paths): Path<(String, String)>,
    Extension(flag): Extension<Session>,
    Query(query): Query<SubscriptionRetrieveRequest>,
) -> Result<Json<SubscriptionChangesToClient>, CustomError> {
    let username = paths.clone().0;
    let deviceid = trim_from_path(&paths.1);
    if flag.username != username.clone() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let res =
        SubscriptionChangesToClient::get_device_subscriptions(deviceid, &username, query.since)
            .await;

    match res {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(CustomErrorInner::Forbidden.into()),
    }
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
    Path(username): Path<String>,
    Extension(flag): Extension<Session>,
    Query(query): Query<SubscriptionRetrieveRequest>,
) -> Result<Json<SubscriptionChangesToClient>, CustomError> {
    let username = trim_from_path(&username);
    if flag.username != username {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let res =
        SubscriptionChangesToClient::get_user_subscriptions(&flag.username, query.since).await;

    match res {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(CustomErrorInner::Forbidden.into()),
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
    Extension(flag): Extension<Session>,
    paths: Path<(String, String)>,
    upload_request: Json<SubscriptionUpdateRequest>,
) -> Result<Json<SubscriptionPostResponse>, CustomError> {
    let username = paths.clone().0;
    let deviceid = trim_from_path(&paths.1);
    if flag.username != username.clone() {
        return Err(CustomErrorInner::Forbidden.into());
    }
    SubscriptionChangesToClient::update_subscriptions(deviceid, &username, upload_request)
        .await
        .unwrap();

    Ok(Json(SubscriptionPostResponse {
        update_urls: vec![],
        timestamp: get_current_timestamp(),
    }))
}

pub fn get_subscription_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(upload_subscription_changes))
        .routes(routes!(get_subscriptions_all))
        .routes(routes!(get_subscriptions))
}
