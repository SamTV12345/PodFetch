use crate::domain::models::device::model::Device;
use utoipa::ToSchema;

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
        DeviceResponse {
            id: device.deviceid.clone(),
            caption: device.name.clone(),
            type_: device.kind.clone(),
            subscriptions: 0,
        }
    }
}
