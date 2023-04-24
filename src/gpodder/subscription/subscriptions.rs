use actix_web::{HttpResponse, Responder, web};
use actix_web::get;
use actix_web::web::Data;
use chrono::NaiveDateTime;
use crate::DbPool;
use crate::models::subscription::SubscriptionChangesToClient;

#[derive(Deserialize, Serialize)]
pub struct SubscriptionUpdateRequest{
    pub since: NaiveDateTime
}

#[get("/subscriptions/{username}/{deviceid}.json")]
pub async fn get_subscriptions(paths: web::Path<(String, String)>,
                         query:web::Query<SubscriptionUpdateRequest>, conn: Data<DbPool>) -> impl
Responder {
    let username = paths.clone().0;
    let deviceid = paths.clone().1;

    let res = SubscriptionChangesToClient::get_device_subscriptions(&deviceid, &username,query
        .since,
                                                          &mut *conn.get().unwrap()).await;
    match res {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::InternalServerError().finish()
    }
}