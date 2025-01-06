use crate::adapters::persistence::dbconfig::schema::devices;
use crate::domain::models::device::model::Device;
use diesel::{Insertable, Queryable, QueryableByName};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Queryable, Insertable, QueryableByName, Clone, ToSchema)]
#[diesel(table_name=devices)]
pub struct DeviceEntity {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub deviceid: String,
    pub kind: String,
    pub name: String,
    pub username: String,
}

impl From<Device> for DeviceEntity {
    fn from(value: Device) -> Self {
        DeviceEntity {
            id: value.id,
            deviceid: value.deviceid,
            kind: value.kind,
            name: value.name,
            username: value.username,
        }
    }
}

impl From<DeviceEntity> for Device {
    fn from(val: DeviceEntity) -> Self {
        Device {
            id: val.id,
            kind: val.kind,
            name: val.name,
            deviceid: val.deviceid,
            username: val.username,
        }
    }
}
