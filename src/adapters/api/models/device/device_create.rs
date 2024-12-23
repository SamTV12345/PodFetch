use crate::domain::models::device::model::Device;

pub struct DeviceCreate {
    pub(crate) id: String,
    pub(crate) caption: String,
    pub(crate) type_: String,
    pub(crate) username: String,
}

impl From<DeviceCreate> for Device {
    fn from(val: DeviceCreate) -> Self {
        Device {
            id: None,
            deviceid: val.id,
            kind: val.type_,
            name: val.caption,
            username: val.username,
        }
    }
}
