use std::ops::DerefMut;
use actix_web::{HttpResponse, web};
use crate::gpodder::device::dto::device_post::DevicePost;
use crate::models::device::{Device, DeviceResponse};
use actix_web::{post, get};
use actix_web::web::Data;
use crate::DbPool;
use crate::models::session::Session;
use crate::utils::error::{CustomError, map_r2d2_error};


#[post("/devices/{username}/{deviceid}.json")]
pub async fn post_device(
    query: web::Path<(String, String)>,
    device_post: web::Json<DevicePost>,
    opt_flag: Option<web::ReqData<Session>>,
    conn: Data<DbPool>) -> Result<HttpResponse, CustomError> {

    match opt_flag{
        Some(flag)=>{
            let username = query.clone().0;
            let deviceid = query.clone().1;
            if flag.username!= username{
                return Err(CustomError::Forbidden);
            }

            let device = Device::new(device_post.into_inner(), deviceid, username);

            let result = device.save(&mut conn.get().map_err(map_r2d2_error)?.deref_mut()).unwrap();

            Ok(HttpResponse::Ok().json(result))
        }
        None=>{
            return Err(CustomError::Forbidden);
        }
    }
}

#[get("/devices/{username}.json")]
pub async fn get_devices_of_user(query: web::Path<String>,opt_flag:
Option<web::ReqData<Session>>, conn: Data<DbPool>) -> Result<HttpResponse, CustomError> {
    match opt_flag {
        Some(flag) => {
            if flag.username!= query.clone(){
                return Err(CustomError::Forbidden);
            }
            let devices = Device::get_devices_of_user(&mut conn.get().map_err(map_r2d2_error)?.deref_mut(), query.clone()).unwrap();

            let dtos = devices.iter().map(|d| d.to_dto()).collect::<Vec<DeviceResponse>>();
            Ok(HttpResponse::Ok().json(dtos))
        }
        None => {
            return Err(CustomError::Forbidden);
        }
    }
}