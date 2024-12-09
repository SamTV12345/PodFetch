use crate::gpodder::device::dto::device_post::DevicePost;
use crate::models::session::Session;
use crate::utils::error::CustomError;
use crate::DbPool;
use actix_web::web::Data;
use actix_web::{get, post, Scope};
use actix_web::{web, HttpResponse};
use crate::adapters::api::models::device::device_response::DeviceResponse;
use crate::adapters::api::models::device::device_create::DeviceCreate;
use crate::application::services::device::service::DeviceService;
use crate::application::usecases::devices::create_use_case::CreateUseCase;
use crate::application::usecases::devices::query_use_case::QueryUseCase;

#[post("/devices/{username}/{deviceid}.json")]
pub async fn post_device(
    query: web::Path<(String, String)>,
    device_post: web::Json<DevicePost>,
    opt_flag: Option<web::ReqData<Session>>
) -> Result<HttpResponse, CustomError> {
    match opt_flag {
        Some(flag) => {
            let username = query.clone().0;
            let deviceid = query.clone().1;
            if flag.username != username {
                return Err(CustomError::Forbidden);
            }

            let device_create = DeviceCreate{
                id: deviceid.clone(),
                username: username.clone(),
                type_: device_post.kind.clone(),
                caption: device_post.caption.clone(),
            };

            let device = DeviceService::create(device_create.into())?;
            let result = DeviceResponse::from(&device);

            Ok(HttpResponse::Ok().json(result))
        }
        None => Err(CustomError::Forbidden),
    }
}

#[get("/devices/{username}.json")]
pub async fn get_devices_of_user(
    query: web::Path<String>,
    opt_flag: Option<web::ReqData<Session>>,
) -> Result<HttpResponse, CustomError> {
    match opt_flag {
        Some(flag) => {
            if flag.username != query.clone() {
                return Err(CustomError::Forbidden);
            }
            let devices = DeviceService::query_by_username(
                query.clone(),
            )?;

            let dtos = devices
                .iter()
                .map(DeviceResponse::from)
                .collect::<Vec<DeviceResponse>>();
            Ok(HttpResponse::Ok().json(dtos))
        }
        None => Err(CustomError::Forbidden),
    }
}

pub fn device_routes() -> Scope {
    Scope::new("")
        .service(post_device)
        .service(get_devices_of_user)
}
