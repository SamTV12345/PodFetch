use axum::{debug_handler, Extension, Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use crate::adapters::api::models::device::device_create::DeviceCreate;
use crate::adapters::api::models::device::device_response::DeviceResponse;
use crate::application::services::device::service::DeviceService;
use crate::application::usecases::devices::create_use_case::CreateUseCase;
use crate::application::usecases::devices::query_use_case::QueryUseCase;
use crate::gpodder::device::dto::device_post::DevicePost;
use crate::models::session::Session;
use crate::utils::error::{CustomError, CustomErrorInner};

pub async fn post_device(
    query: Path<(String, String)>,
    Extension(opt_flag): Extension<Option<Session>>,
    Json(device_post): Json<DevicePost>,
) -> Result<Json<DeviceResponse>, CustomError> {
    match opt_flag {
        Some(flag) => {
            let username = &query.0.0;
            let deviceid = &query.0.1;
            if &flag.username != username {
                return Err(CustomErrorInner::Forbidden.into());
            }

            let device_create = DeviceCreate {
                id: deviceid.clone(),
                username: username.clone(),
                type_: device_post.kind.clone(),
                caption: device_post.caption.clone(),
            };

            let device = DeviceService::create(device_create.into())?;
            let result = DeviceResponse::from(&device);

            Ok(Json(result))
        }
        None => Err(CustomErrorInner::Forbidden.into()),
    }
}

#[debug_handler]
pub async fn get_devices_of_user(
    Path(query): Path<String>,
    Extension(opt_flag): Extension<Option<Session>>,
) -> Result<Json<Vec<DeviceResponse>>, CustomError> {
    match opt_flag {
        Some(flag) => {
            let user_query = query;
            if flag.username != user_query {
                return Err(CustomErrorInner::Forbidden.into());
            }
            let devices = DeviceService::query_by_username(&user_query)?;

            let dtos = devices
                .iter()
                .map(DeviceResponse::from)
                .collect::<Vec<DeviceResponse>>();
            Ok(Json(dtos))
        }
        None => Err(CustomErrorInner::Forbidden.into()),
    }
}

pub fn get_device_router() -> Router {
    Router::new()
        .route("/devices/{username}.json", get(get_devices_of_user))
        .route("/devices/{username}/{deviceid}.json", post(post_device))
}