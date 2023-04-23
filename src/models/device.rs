use diesel::{Queryable, QueryableByName, RunQueryDsl, Insertable, SqliteConnection};
use diesel::associations::HasTable;
use utoipa::ToSchema;
use crate::gpodder::device::dto::device_post::DevicePost;
use crate::schema::devices;
use diesel::QueryDsl;
use diesel::ExpressionMethods;

#[derive(Serialize, Deserialize, Queryable,Insertable, QueryableByName, Clone, ToSchema)]
#[diesel(table_name=devices)]
pub struct Device {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub deviceid: String,
    pub kind: String,
    pub name: String,
    pub username: String
}

impl Device {

    pub fn new(device_post: DevicePost, device_id: String, username: String) -> Device {
        Device{
            id: None,
            deviceid:device_id,
            kind: device_post.kind,
            name: device_post.caption,
            username,
        }
    }

    pub fn save(&self, conn: &mut SqliteConnection) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::insert_into(devices)
            .values(self)
            .get_result(conn)
    }

    pub fn get_devices_of_user(conn: &mut SqliteConnection, username_to_insert: String) ->
                                                                               Result<Vec<Device>, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices.filter(username.eq(username_to_insert))
            .load::<Device>(conn)
    }
}