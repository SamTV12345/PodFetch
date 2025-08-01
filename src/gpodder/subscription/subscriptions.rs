use crate::models::session::Session;
use crate::models::subscription::SubscriptionChangesToClient;
use crate::utils::error::ErrorSeverity::Warning;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::gpodder_trimmer::trim_from_path;
use crate::utils::time::get_current_timestamp;
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use opml::{Outline, OPML};
use serde::Serialize;
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
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    let res =
        SubscriptionChangesToClient::get_device_subscriptions(deviceid.0, &username, query.since)
            .await;

    match res {
        Ok(res) => Ok(Json(res.into())),
        Err(_) => Err(CustomErrorInner::Forbidden(Warning).into()),
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
) -> Result<impl IntoResponse, CustomError> {
    let username = trim_from_path(&username);
    if flag.username != username.0 {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    let res =
        SubscriptionChangesToClient::get_user_subscriptions(&flag.username, query.since).await;

    match res {
        Ok(res) => {
            if username.1 == "opml" {
                let mut opml = OPML::default();
                res.add.iter().for_each(|s| {
                    opml.body.outlines.push(Outline {
                        text: s.podcast.to_string(),
                        r#type: Some("rss".to_string()),
                        is_comment: None,
                        is_breakpoint: None,
                        created: Some(s.created.to_string()),
                        category: None,
                        outlines: vec![],
                        xml_url: Some(s.podcast.to_string()),
                        description: None,
                        html_url: None,
                        language: None,
                        title: Some(s.podcast.to_string()),
                        version: None,
                        url: None,
                    });
                });

                Ok(opml.to_string().unwrap().into_response())
            } else {
                let tes: SubscriptionChangesToClient = res.into();
                Ok(Json(tes).into_response())
            }
        }
        Err(_) => Err(CustomErrorInner::Forbidden(Warning).into()),
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
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    SubscriptionChangesToClient::update_subscriptions(deviceid.0, &username, upload_request)
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
