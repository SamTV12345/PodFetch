use actix_web::{HttpRequest, HttpResponse, Responder, web};
use crate::gpodder::device::dto::device_post::DevicePost;
use crate::models::device::{Device, DeviceResponse};
use actix_web::{post, get};
use actix_web::web::Data;
use crate::controllers::watch_time_controller::get_username;
use crate::DbPool;
use crate::gpodder::auth::auth::{auth_checker, extract_from_http_request};

#[post("/devices/{username}/{deviceid}.json")]
pub async fn post_device(
    query: web::Path<(String, String)>,
    device_post: web::Json<DevicePost>,
    rq: HttpRequest,
    conn: Data<DbPool>) -> impl Responder {
    let username = query.clone().0;
    let deviceid = query.clone().1;
    let auth_check_res= auth_checker(&mut *conn.get().unwrap(), extract_from_http_request(rq),
                                   username.clone()).await;
    if auth_check_res.is_err(){
        return HttpResponse::Unauthorized().body(auth_check_res.err().unwrap().to_string());
    }
    let device = Device::new(device_post.into_inner(), deviceid, username);

    let result = device.save(&mut conn.get().unwrap()).unwrap();

    HttpResponse::Ok().json(result)
}

#[get("/devices/{username}.json")]
pub async fn get_devices_of_user(query: web::Path<String>, conn: Data<DbPool>, rq: HttpRequest) -> impl Responder {
    let auth_check_res= auth_checker(&mut *conn.get().unwrap(), extract_from_http_request(rq),
                                     query.clone()).await;
    if auth_check_res.is_err(){
        return HttpResponse::Unauthorized().body(auth_check_res.err().unwrap().to_string());
    }

    let devices = Device::get_devices_of_user(&mut conn.get().unwrap(), query.clone()).unwrap();

    let dtos = devices.iter().map(|d|d.to_dto()).collect::<Vec<DeviceResponse>>();
    HttpResponse::Ok().json(dtos)
}