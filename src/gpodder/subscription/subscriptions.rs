use crate::models::session::Session;
use crate::models::subscription::SubscriptionChangesToClient;
use crate::utils::error::CustomError;
use crate::utils::time::get_current_timestamp;
use actix_web::{get, post};
use actix_web::{web, HttpResponse};

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

#[get("/subscriptions/{username}/{deviceid}.json")]
pub async fn get_subscriptions(
    paths: web::Path<(String, String)>,
    opt_flag: Option<web::ReqData<Session>>,
    query: web::Query<SubscriptionRetrieveRequest>,
) -> Result<HttpResponse, CustomError> {
    match opt_flag {
        Some(flag) => {
            let username = paths.clone().0;
            let deviceid = paths.clone().1;
            if flag.username != username.clone() {
                return Err(CustomError::Forbidden);
            }

            let res = SubscriptionChangesToClient::get_device_subscriptions(
                &deviceid,
                &username,
                query.since,
            )
            .await;

            match res {
                Ok(res) => Ok(HttpResponse::Ok().json(res)),
                Err(_) => Ok(HttpResponse::InternalServerError().finish()),
            }
        }
        None => Err(CustomError::Forbidden),
    }
}

#[post("/subscriptions/{username}/{deviceid}.json")]
pub async fn upload_subscription_changes(
    upload_request: web::Json<SubscriptionUpdateRequest>,
    opt_flag: Option<web::ReqData<Session>>,
    paths: web::Path<(String, String)>,
) -> Result<HttpResponse, CustomError> {
    match opt_flag {
        Some(flag) => {
            let username = paths.clone().0;
            let deviceid = paths.clone().1;
            if flag.username != username.clone() {
                return Err(CustomError::Forbidden);
            }
            SubscriptionChangesToClient::update_subscriptions(&deviceid, &username, upload_request)
                .await
                .unwrap();

            Ok(HttpResponse::Ok().json(SubscriptionPostResponse {
                update_urls: vec![],
                timestamp: get_current_timestamp(),
            }))
        }
        None => Err(CustomError::Forbidden),
    }
}
