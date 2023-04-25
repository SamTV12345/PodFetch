use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::{get, post};
use actix_web::web::Data;
use crate::DbPool;
use crate::gpodder::auth::auth::{auth_checker, extract_from_http_request};
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
    pub update_urls: Vec<String>
}

#[get("/subscriptions/{username}/{deviceid}.json")]
pub async fn get_subscriptions(paths: web::Path<(String, String)>,
                               query:web::Query<SubscriptionRetrieveRequest>, conn: Data<DbPool>,
                               rq:HttpRequest
) -> impl
Responder {
    let username = paths.clone().0;
    let deviceid = paths.clone().1;

    let auth_check_res= auth_checker(&mut *conn.get().unwrap(), extract_from_http_request(rq),
                                     username.clone()).await;
    if auth_check_res.is_err(){
        return HttpResponse::Unauthorized().body(auth_check_res.err().unwrap().to_string());
    }

    let res = SubscriptionChangesToClient::get_device_subscriptions(&deviceid, &username,query
        .since,
                                                          &mut *conn.get().unwrap()).await;

    println!("res: {:?}", res);
    match res {
        Ok(res) => {
            HttpResponse::Ok().json(res)
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[post("/subscriptions/{username}/{deviceid}.json")]
pub async fn upload_subscription_changes(upload_request: web::Json<SubscriptionUpdateRequest>,
                                         paths: web::Path<(String, String)>, conn: Data<DbPool>,
                                         rq:HttpRequest)->impl Responder{
    let username = paths.clone().0;
    let deviceid = paths.clone().1;

    let auth_check_res= auth_checker(&mut *conn.get().unwrap(), extract_from_http_request(rq),
                                     username.clone()).await;
    if auth_check_res.is_err(){
        return HttpResponse::Unauthorized().body(auth_check_res.err().unwrap().to_string());
    }
    SubscriptionChangesToClient::update_subscriptions(&deviceid, &username,
                                                                upload_request,
                                                      &mut *conn.get().unwrap()).await.expect
    ("TODO: panic message");

    HttpResponse::Ok().json(SubscriptionPostResponse{
        update_urls: vec![],
        timestamp: get_current_timestamp()
    })
}