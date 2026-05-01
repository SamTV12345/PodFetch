use podfetch_domain::device::Device;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

pub struct DeviceCreate {
    pub id: String,
    pub caption: String,
    pub type_: String,
    pub user_id: i32,
}

impl From<DeviceCreate> for Device {
    fn from(val: DeviceCreate) -> Self {
        Device {
            id: None,
            deviceid: val.id,
            kind: val.type_,
            name: val.caption,
            user_id: val.user_id,
            chromecast_uuid: None,
            agent_id: None,
            last_seen_at: None,
            ip: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct DeviceResponse {
    id: String,
    caption: String,
    #[serde(rename = "type")]
    type_: String,
    subscriptions: u32,
}

impl From<&Device> for DeviceResponse {
    fn from(device: &Device) -> Self {
        Self {
            id: device.deviceid.clone(),
            caption: device.name.clone(),
            type_: device.kind.clone(),
            subscriptions: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct DevicePost {
    pub caption: String,
    #[serde(rename = "type")]
    pub kind: String,
}

pub trait DeviceApplicationService {
    type Error;

    fn create(&self, device: Device) -> Result<Device, Self::Error>;
    fn query_by_user_id(&self, user_id: i32) -> Result<Vec<Device>, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum DeviceControllerError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("{0}")]
    Service(E),
}

pub fn post_device<S>(
    service: &S,
    session_username: &str,
    session_user_id: i32,
    username: &str,
    device_id: &str,
    device_post: DevicePost,
) -> Result<DeviceResponse, DeviceControllerError<S::Error>>
where
    S: DeviceApplicationService,
    S::Error: Display,
{
    if session_username != username {
        return Err(DeviceControllerError::Forbidden);
    }

    let device = service
        .create(
            DeviceCreate {
                id: device_id.to_string(),
                caption: device_post.caption,
                type_: device_post.kind,
                user_id: session_user_id,
            }
            .into(),
        )
        .map_err(DeviceControllerError::Service)?;

    Ok(DeviceResponse::from(&device))
}

pub fn get_devices_of_user<S>(
    service: &S,
    session_username: &str,
    session_user_id: i32,
    username: &str,
) -> Result<Vec<DeviceResponse>, DeviceControllerError<S::Error>>
where
    S: DeviceApplicationService,
    S::Error: Display,
{
    if session_username != username {
        return Err(DeviceControllerError::Forbidden);
    }

    service
        .query_by_user_id(session_user_id)
        .map(|devices| devices.iter().map(DeviceResponse::from).collect())
        .map_err(DeviceControllerError::Service)
}
