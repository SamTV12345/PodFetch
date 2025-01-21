use axum::{debug_handler, Extension, Json, Router};
use axum::extract::{Path, Query};
use axum::http::Response;
use axum::routing::{get, post};
use crate::models::session::Session;
use crate::models::subscription::SubscriptionChangesToClient;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::time::get_current_timestamp;

#[derive(Deserialize, Serialize)]
pub struct SubscriptionRetrieveRequest {
    pub since: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SubscriptionUpdateRequest {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct SubscriptionPostResponse {
    pub timestamp: i64,
    pub update_urls: Vec<Vec<String>>,
}

#[debug_handler]
pub async fn get_subscriptions(
    Path(paths): Path<(String, String)>,
    Extension(opt_flag): Extension<Option<Session>>,
    Query(query): Query<SubscriptionRetrieveRequest>,
) -> Result<Json<SubscriptionChangesToClient>, CustomError> {
    match opt_flag {
        Some(flag) => {
            let username = paths.clone().0;
            let deviceid = paths.clone().1;
            if flag.username != username.clone() {
                return Err(CustomErrorInner::Forbidden.into());
            }

            let res = SubscriptionChangesToClient::get_device_subscriptions(
                &deviceid,
                &username,
                query.since,
            )
                .await;

            match res {
                Ok(res) => Ok(Json(res)),
                Err(_) => Err(CustomErrorInner::Forbidden.into()),
            }
        }
        None => Err(CustomErrorInner::Forbidden.into()),
    }
}

pub async fn get_subscriptions_all(
    Path(paths): Path<String>,
    Extension(opt_flag): Extension<Option<Session>>,
    Query(query): Query<SubscriptionRetrieveRequest>,
) -> Result<Json<SubscriptionChangesToClient>, CustomError> {
    let flag_username = match opt_flag {
        Some(flag) => flag.username,
        None => return Err(CustomErrorInner::Forbidden.into()),
    };
    if flag_username != paths.as_str() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let res = SubscriptionChangesToClient::get_user_subscriptions(&flag_username, query.since)
        .await;

    match res {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(CustomErrorInner::Forbidden.into()),
    }
}


pub async fn upload_subscription_changes(
        Extension(opt_flag): Extension<Option<Session>>,
        paths: Path<(String, String)>,
        upload_request: Json<SubscriptionUpdateRequest>,
) -> Result<Json<SubscriptionPostResponse>, CustomError> {
        match opt_flag {
            Some(flag) => {
                let username = paths.clone().0;
                let deviceid = paths.clone().1;
                if flag.username != username.clone() {
                    return Err(CustomErrorInner::Forbidden.into());
                }
                SubscriptionChangesToClient::update_subscriptions(&deviceid, &username, upload_request)
                    .await
                    .unwrap();

                Ok(Json(SubscriptionPostResponse {
                    update_urls: vec![],
                    timestamp: get_current_timestamp(),
                }))
            }
            None => Err(CustomErrorInner::Forbidden.into()),
        }
}


pub fn get_subscription_router() -> Router {
    Router::new()
        .route("/subscriptions/{username}/{deviceid}.json", post(upload_subscription_changes))
        .route("/subscriptions/{username}.json", get(get_subscriptions_all))
        .route("/subscriptions/{username}/{deviceid}.json", get(get_subscriptions))
}
