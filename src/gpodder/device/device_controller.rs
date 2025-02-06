use crate::adapters::api::models::device::device_create::DeviceCreate;
use crate::adapters::api::models::device::device_response::DeviceResponse;
use crate::application::services::device::service::DeviceService;
use crate::application::usecases::devices::create_use_case::CreateUseCase;
use crate::application::usecases::devices::query_use_case::QueryUseCase;
use crate::gpodder::device::dto::device_post::DevicePost;
use crate::models::session::Session;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::gpodder_trimmer::trim_from_path;
use axum::extract::Path;
use axum::{Extension, Json};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    post,
    path="/devices/{username}/{deviceid}",
    request_body=DevicePost,
    responses(
        (status = 200, description = "Creates a new device.", body = DeviceResponse),
        (status = 403, description = "Forbidden.")
    ),
    tag="gpodder"
)]
pub async fn post_device(
    query: Path<(String, String)>,
    Extension(flag): Extension<Session>,
    Json(device_post): Json<DevicePost>,
) -> Result<Json<DeviceResponse>, CustomError> {
    let username = &query.0 .0;
    let deviceid = trim_from_path(&query.0 .1);
    if &flag.username != username {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let device_create = DeviceCreate {
        id: deviceid.0.to_string(),
        username: username.clone(),
        type_: device_post.kind.clone(),
        caption: device_post.caption.clone(),
    };

    let device = DeviceService::create(device_create.into())?;
    let result = DeviceResponse::from(&device);

    Ok(Json(result))
}

#[utoipa::path(
    get,
    path="/devices/{username}",
    responses(
        (status = 200, description = "Gets all devices of a user.", body = [DeviceResponse])
    ),
    tag="devices"
)]
pub async fn get_devices_of_user(
    Path(query): Path<String>,
    Extension(flag): Extension<Session>,
) -> Result<Json<Vec<DeviceResponse>>, CustomError> {
    let query = trim_from_path(&query);
    let user_query = query.0;
    if flag.username != user_query {
        return Err(CustomErrorInner::Forbidden.into());
    }
    let devices = DeviceService::query_by_username(user_query)?;

    let dtos = devices
        .iter()
        .map(DeviceResponse::from)
        .collect::<Vec<DeviceResponse>>();
    Ok(Json(dtos))
}

pub fn get_device_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_devices_of_user))
        .routes(routes!(post_device))
}
