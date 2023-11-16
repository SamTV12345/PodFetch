use crate::dbconfig::schema::devices;
use crate::gpodder::device::dto::device_post::DevicePost;
use crate::{execute_with_conn, DBType as DbConnection};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{Insertable, Queryable, QueryableByName, RunQueryDsl};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Queryable, Insertable, QueryableByName, Clone, ToSchema)]
#[diesel(table_name=devices)]
pub struct Device {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub deviceid: String,
    pub kind: String,
    pub name: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceResponse {
    id: String,
    caption: String,
    #[serde(rename = "type")]
    type_: String,
    subscriptions: u32,
}

impl Device {
    pub fn new(device_post: DevicePost, device_id: String, username: String) -> Device {
        Device {
            id: None,
            deviceid: device_id,
            kind: device_post.kind,
            name: device_post.caption,
            username,
        }
    }

    #[allow(clippy::redundant_closure_call)]
    pub fn save(&self, conn: &mut DbConnection) -> Result<Device, diesel::result::Error> {
        use crate::dbconfig::schema::devices::dsl::*;

        execute_with_conn!(conn, |conn| diesel::insert_into(devices)
            .values(self)
            .get_result(conn));
    }

    pub fn get_devices_of_user(
        conn: &mut DbConnection,
        username_to_insert: String,
    ) -> Result<Vec<Device>, diesel::result::Error> {
        use crate::dbconfig::schema::devices::dsl::*;
        devices
            .filter(username.eq(username_to_insert))
            .load::<Device>(conn)
    }

    pub fn to_dto(&self) -> DeviceResponse {
        DeviceResponse {
            id: self.deviceid.clone(),
            caption: self.name.clone(),
            type_: self.kind.clone(),
            subscriptions: 0,
        }
    }
    pub fn delete_by_username(
        username1: String,
        conn: &mut DbConnection,
    ) -> Result<usize, diesel::result::Error> {
        use crate::dbconfig::schema::devices::dsl::*;
        diesel::delete(devices.filter(username.eq(username1))).execute(conn)
    }
}
