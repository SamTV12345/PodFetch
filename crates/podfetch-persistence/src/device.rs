use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::BoolExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{ExpressionMethods, QueryDsl};
use podfetch_domain::device::{Device, DeviceRepository, kind as device_kind};

diesel::table! {
    devices (id) {
        id -> Nullable<Integer>,
        deviceid -> Text,
        kind -> Text,
        name -> Text,
        user_id -> Integer,
        chromecast_uuid -> Nullable<Text>,
        agent_id -> Nullable<Text>,
        last_seen_at -> Nullable<Timestamp>,
        ip -> Nullable<Text>,
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
    chromecast_uuid: Option<String>,
    agent_id: Option<String>,
    last_seen_at: Option<NaiveDateTime>,
    ip: Option<String>,
}

impl From<Device> for DeviceEntity {
    fn from(value: Device) -> Self {
        Self {
            id: value.id,
            deviceid: value.deviceid,
            kind: value.kind,
            name: value.name,
            user_id: value.user_id,
            chromecast_uuid: value.chromecast_uuid,
            agent_id: value.agent_id,
            last_seen_at: value.last_seen_at,
            ip: value.ip,
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
            chromecast_uuid: value.chromecast_uuid,
            agent_id: value.agent_id,
            last_seen_at: value.last_seen_at,
            ip: value.ip,
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

    fn list_castable_for_user(
        &self,
        viewer_user_id: i32,
    ) -> Result<Vec<Device>, Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;

        // Owned personal Chromecasts OR any shared Chromecast on the instance.
        devices
            .filter(
                kind.eq(device_kind::CHROMECAST_SHARED).or(kind
                    .eq(device_kind::CHROMECAST_PERSONAL)
                    .and(user_id.eq(viewer_user_id))),
            )
            .load::<DeviceEntity>(&mut conn)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
}
