use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use crate::gpodder::device::dto::device_post::DevicePost;
use crate::models::device::{Device, DeviceResponse};
use actix_web::{post, get};
use actix_web::web::Data;
use crate::DbPool;
use crate::gpodder::auth::auth::{auth_checker, extract_from_http_request};
use crate::models::session::Session;


#[post("/devices/{username}/{deviceid}.json")]
pub async fn post_device(
    query: web::Path<(String, String)>,
    device_post: web::Json<DevicePost>,
    opt_flag: Option<web::ReqData<Session>>,
    conn: Data<DbPool>) -> impl Responder {

    match opt_flag{
        Some(flag)=>{
            let username = query.clone().0;
            let deviceid = query.clone().1;
            if flag.username!= username{
                return HttpResponse::Unauthorized().finish();
            }

            let device = Device::new(device_post.into_inner(), deviceid, username);

            let result = device.save(&mut conn.get().unwrap()).unwrap();

            HttpResponse::Ok().json(result)
        }
        None=>{
            HttpResponse::Unauthorized().finish()
        }
    }
}

#[get("/devices/{username}.json")]
pub async fn get_devices_of_user(query: web::Path<String>,opt_flag: Option<web::ReqData<Session>>, conn: Data<DbPool>) -> impl Responder {
    match opt_flag {
        Some(flag) => {
            if flag.username!= query.clone(){
                return HttpResponse::Unauthorized().finish();
            }
            let devices = Device::get_devices_of_user(&mut conn.get().unwrap(), query.clone()).unwrap();

            let dtos = devices.iter().map(|d| d.to_dto()).collect::<Vec<DeviceResponse>>();
            HttpResponse::Ok().json(dtos)
        }
        None => {
            HttpResponse::Unauthorized().finish()
        }
    }
}