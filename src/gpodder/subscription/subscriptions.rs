use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::{get, post};
use actix_web::web::Data;
use crate::DbPool;
use crate::gpodder::auth::auth::{auth_checker, extract_from_http_request};
use crate::models::session::Session;
use crate::models::subscription::SubscriptionChangesToClient;
use crate::utils::time::get_current_timestamp;

#[derive(Deserialize, Serialize)]
pub struct SubscriptionRetrieveRequest {
    pub since: i32
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SubscriptionUpdateRequest {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct SubscriptionPostResponse {
    pub timestamp: i64,
    pub update_urls: Vec<Vec<String>>
}

#[get("/subscriptions/{username}/{deviceid}.json")]
pub async fn get_subscriptions(paths: web::Path<(String, String)>,opt_flag:
                                Option<web::ReqData<Session>>,
                               query:web::Query<SubscriptionRetrieveRequest>, conn: Data<DbPool>) -> impl
Responder {
    match opt_flag {
        Some(flag) => {
            let username = paths.clone().0;
            let deviceid = paths.clone().1;
            if flag.username != username.clone() {
                return HttpResponse::Unauthorized().finish();
            }

            let res = SubscriptionChangesToClient::get_device_subscriptions(&deviceid, &username, query
                .since,
                                                                            &mut *conn.get().unwrap()).await;

            match res {
                Ok(res) => {
                    HttpResponse::Ok().json(res)
                },
                Err(_) => HttpResponse::InternalServerError().finish()
            }
        }
        None => {
            HttpResponse::Unauthorized().finish()
        }
    }
}

#[post("/subscriptions/{username}/{deviceid}.json")]
pub async fn upload_subscription_changes(upload_request: web::Json<SubscriptionUpdateRequest>,
                                         opt_flag: Option<web::ReqData<Session>>,
                                         paths: web::Path<(String, String)>, conn: Data<DbPool>,
                                         rq:HttpRequest)->impl Responder {
    match opt_flag {
        Some(flag) => {
            let username = paths.clone().0;
            let deviceid = paths.clone().1;
            if flag.username != username.clone() {
                return HttpResponse::Unauthorized().finish();
            }
            let res = SubscriptionChangesToClient::update_subscriptions(&deviceid, &username,
                                                              upload_request,
                                                              &mut *conn.get().unwrap()).await.expect("TODO: panic message");

            HttpResponse::Ok().json(SubscriptionPostResponse {
                update_urls: vec![],
                timestamp: get_current_timestamp()
            })
        }
        None => {
            HttpResponse::Unauthorized().finish()
        }
    }
}