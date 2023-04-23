use actix_session::{Session, SessionExt};
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use crate::gpodder::device::dto::device_post::DevicePost;
use crate::models::device::Device;
use actix_web::{post, get};
use actix_web::web::Data;
use serde::de::Unexpected::Str;
use crate::controllers::user_controller::get_user;
use crate::controllers::watch_time_controller::get_username;
use crate::DbPool;
use crate::models::user::User;

#[post("/devices/{username}/{deviceid}.json")]
pub async fn post_device(
    query: web::Path<(String, String)>,
    device_post: web::Json<DevicePost>,
    conn: Data<DbPool>,
    session:Session,
    rq: HttpRequest
) -> impl Responder {

    let headers = rq.get_session();


    let username:Option<String> = session.get("test").unwrap();

    if username.is_none() {
        return HttpResponse::Unauthorized().finish();
    }
    let username = query.clone().0;
    let deviceid = query.clone().1;

    let device = Device::new(device_post.into_inner(), deviceid, username);

    let result = device.save(&mut conn.get().unwrap()).unwrap();

    HttpResponse::Ok().json(result)
}

#[get("/devices/{username}.json")]
pub async fn get_devices_of_user(query: web::Path<String>, conn: Data<DbPool>, rq: HttpRequest) -> impl Responder {
    let username = get_username(rq);
    if username.is_err(){
        return username.err().unwrap()
    }


    let username = query.clone();

    if query.clone() != username {
        return HttpResponse::Unauthorized().finish();
    }
    let devices = Device::get_devices_of_user(&mut conn.get().unwrap(), username).unwrap();
    HttpResponse::Ok().json(devices)
}