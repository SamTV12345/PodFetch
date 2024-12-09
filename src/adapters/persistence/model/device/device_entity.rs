use crate::dbconfig::schema::devices;
use crate::gpodder::device::dto::device_post::DevicePost;
use crate::{execute_with_conn, DBType as DbConnection};
use diesel::QueryDsl;
use diesel::{Insertable, Queryable, QueryableByName, RunQueryDsl};
use utoipa::ToSchema;
use diesel::ExpressionMethods;
use crate::domain::models::device::model::Device;

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


impl From<Device> for DeviceEntity{
    fn from(value: Device) -> Self {
        DeviceEntity{
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
            username: val.username
        }
    }
}

impl DeviceEntity {
    pub fn new(device_post: DevicePost, device_id: String, username: String) -> DeviceEntity {
        DeviceEntity {
            id: None,
            deviceid: device_id,
            kind: device_post.kind,
            name: device_post.caption,
            username,
        }
    }

    #[allow(clippy::redundant_closure_call)]
    pub fn save(&self, conn: &mut DbConnection) -> Result<DeviceEntity, diesel::result::Error> {
        use crate::dbconfig::schema::devices::dsl::*;

        execute_with_conn!(conn, |conn| diesel::insert_into(devices)
            .values(self)
            .get_result(conn));
    }

    pub fn get_devices_of_user(
        conn: &mut DbConnection,
        username_to_insert: String,
    ) -> Result<Vec<DeviceEntity>, diesel::result::Error> {
        use crate::dbconfig::schema::devices::dsl::*;
        devices
            .filter(username.eq(username_to_insert))
            .load::<DeviceEntity>(conn)
    }


    pub fn delete_by_username(
        username1: &str,
        conn: &mut DbConnection,
    ) -> Result<usize, diesel::result::Error> {
        use crate::dbconfig::schema::devices::dsl::*;
        diesel::delete(devices.filter(username.eq(username1))).execute(conn)
    }
}
