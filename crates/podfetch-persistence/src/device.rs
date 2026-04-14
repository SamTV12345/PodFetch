use crate::db::{Database, PersistenceError};
use diesel::RunQueryDsl;
use diesel::{ExpressionMethods, QueryDsl};
use podfetch_domain::device::{Device, DeviceRepository};

diesel::table! {
    devices (id) {
        id -> Nullable<Integer>,
        deviceid -> Text,
        kind -> Text,
        name -> Text,
        user_id -> Integer,
    }
}

#[derive(diesel::Queryable, diesel::Insertable, Clone)]
#[diesel(table_name = devices)]
struct DeviceEntity {
    id: Option<i32>,
    deviceid: String,
    kind: String,
    name: String,
    user_id: i32,
}

impl From<Device> for DeviceEntity {
    fn from(value: Device) -> Self {
        Self {
            id: value.id,
            deviceid: value.deviceid,
            kind: value.kind,
            name: value.name,
            user_id: value.user_id,
        }
    }
}

impl From<DeviceEntity> for Device {
    fn from(value: DeviceEntity) -> Self {
        Self {
            id: value.id,
            deviceid: value.deviceid,
            kind: value.kind,
            name: value.name,
            user_id: value.user_id,
        }
    }
}

pub struct DieselDeviceRepository {
    database: Database,
}

impl DieselDeviceRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl DeviceRepository for DieselDeviceRepository {
    type Error = PersistenceError;

    fn create(&self, device: Device) -> Result<Device, Self::Error> {
        use self::devices::dsl::*;

        let device_entity: DeviceEntity = device.into();
        let mut conn = self.database.connection()?;

        diesel::insert_into(devices)
            .values(device_entity)
            .get_result::<DeviceEntity>(&mut conn)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn get_devices_of_user(&self, user_id_to_find: i32) -> Result<Vec<Device>, Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;

        devices
            .filter(user_id.eq(user_id_to_find))
            .load::<DeviceEntity>(&mut conn)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id_to_delete: i32) -> Result<(), Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;

        diesel::delete(devices.filter(user_id.eq(user_id_to_delete)))
            .execute(&mut conn)
            .map(|_| ())
            .map_err(Into::into)
    }
}
