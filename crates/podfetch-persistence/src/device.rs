use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::RunQueryDsl;
use diesel::{ExpressionMethods, QueryDsl};
use podfetch_domain::device::{Device, DeviceRepository};
use uuid::Uuid;

diesel::table! {
    devices (id) {
        id -> Nullable<Text>,
        deviceid -> Text,
        kind -> Text,
        name -> Text,
        user_id -> Text,
        chromecast_uuid -> Nullable<Text>,
        agent_id -> Nullable<Text>,
        last_seen_at -> Nullable<Timestamp>,
        ip -> Nullable<Text>,
        base_url -> Nullable<Text>,
    }
}

#[derive(diesel::Queryable, diesel::Insertable, Clone)]
#[diesel(table_name = devices)]
struct DeviceEntity {
    id: Option<String>,
    deviceid: String,
    kind: String,
    name: String,
    user_id: String,
    chromecast_uuid: Option<String>,
    agent_id: Option<String>,
    last_seen_at: Option<NaiveDateTime>,
    ip: Option<String>,
    base_url: Option<String>,
}

impl From<Device> for DeviceEntity {
    fn from(value: Device) -> Self {
        Self {
            // Every inserted device must carry a non-null TEXT id; generate one
            // when the domain object has none yet.
            id: Some(
                value
                    .id
                    .unwrap_or_else(podfetch_domain::ids::new_id)
                    .to_string(),
            ),
            deviceid: value.deviceid,
            kind: value.kind,
            name: value.name,
            user_id: value.user_id.to_string(),
            chromecast_uuid: value.chromecast_uuid,
            agent_id: value.agent_id,
            last_seen_at: value.last_seen_at,
            ip: value.ip,
            base_url: value.base_url,
        }
    }
}

impl From<DeviceEntity> for Device {
    fn from(value: DeviceEntity) -> Self {
        Self {
            id: value
                .id
                .as_deref()
                .map(|s| Uuid::parse_str(s).expect("valid uuid in db")),
            deviceid: value.deviceid,
            kind: value.kind,
            name: value.name,
            user_id: Uuid::parse_str(&value.user_id).expect("valid uuid in db"),
            chromecast_uuid: value.chromecast_uuid,
            agent_id: value.agent_id,
            last_seen_at: value.last_seen_at,
            ip: value.ip,
            base_url: value.base_url,
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

    fn get_devices_of_user(&self, user_id_to_find: Uuid) -> Result<Vec<Device>, Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;

        devices
            .filter(user_id.eq(user_id_to_find.to_string()))
            .load::<DeviceEntity>(&mut conn)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id_to_delete: Uuid) -> Result<(), Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;

        diesel::delete(devices.filter(user_id.eq(user_id_to_delete.to_string())))
            .execute(&mut conn)
            .map(|_| ())
            .map_err(Into::into)
    }
}
