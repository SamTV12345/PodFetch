use crate::devices::{CastDevice, CastDeviceRepository, kind as device_kind};
use chrono::NaiveDateTime;
use common_infrastructure::db::{Database, PersistenceError};
use common_infrastructure::error::CustomError;
use diesel::BoolExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{ExpressionMethods, QueryDsl};
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

impl From<CastDevice> for DeviceEntity {
    fn from(value: CastDevice) -> Self {
        Self {
            // Every inserted device must carry a non-null TEXT id; generate one
            // when the domain object has none yet.
            id: Some(value.id.unwrap_or_else(Uuid::now_v7).to_string()),
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

impl From<DeviceEntity> for CastDevice {
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

pub struct DieselCastDeviceRepository {
    database: Database,
}

impl DieselCastDeviceRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl CastDeviceRepository for DieselCastDeviceRepository {
    type Error = PersistenceError;

    fn register(&self, device: CastDevice) -> Result<CastDevice, Self::Error> {
        use self::devices::dsl::*;

        let device_entity: DeviceEntity = device.into();
        let mut conn = self.database.connection()?;

        diesel::insert_into(devices)
            .values(device_entity)
            .get_result::<DeviceEntity>(&mut conn)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn delete_owned_by(&self, owner: Uuid) -> Result<usize, Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;
        diesel::delete(
            devices
                .filter(user_id.eq(owner.to_string()))
                .filter(kind.eq_any(device_kind::CASTABLE)),
        )
        .execute(&mut conn)
        .map_err(Into::into)
    }

    fn find_by_chromecast_uuid(
        &self,
        chromecast_uuid_to_find: &str,
    ) -> Result<Option<CastDevice>, Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;

        match devices
            .filter(chromecast_uuid.eq(chromecast_uuid_to_find))
            .first::<DeviceEntity>(&mut conn)
        {
            Ok(entity) => Ok(Some(entity.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn upsert_chromecast_from_agent(
        &self,
        chromecast_uuid_value: &str,
        agent_id_value: &str,
        owner_user_id: Uuid,
        name_value: &str,
        ip_value: Option<&str>,
        last_seen_at_value: NaiveDateTime,
    ) -> Result<CastDevice, Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;

        // Try to find existing row for this chromecast UUID.
        let existing: Option<DeviceEntity> = devices
            .filter(chromecast_uuid.eq(chromecast_uuid_value))
            .first::<DeviceEntity>(&mut conn)
            .map(Some)
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                other => Err(other),
            })?;

        match existing {
            Some(row) => {
                let row_id = row.id;
                // Preserve the existing kind so admin-promoted shared
                // devices stay shared even when the agent reports them.
                diesel::update(devices.filter(chromecast_uuid.eq(chromecast_uuid_value)))
                    .set((
                        agent_id.eq(Some(agent_id_value)),
                        name.eq(name_value),
                        ip.eq(ip_value),
                        last_seen_at.eq(Some(last_seen_at_value)),
                    ))
                    .execute(&mut conn)?;

                let updated = devices
                    .filter(chromecast_uuid.eq(chromecast_uuid_value))
                    .first::<DeviceEntity>(&mut conn)?;
                let _ = row_id;
                Ok(updated.into())
            }
            None => {
                let entity = DeviceEntity {
                    id: Some(Uuid::now_v7().to_string()),
                    deviceid: chromecast_uuid_value.to_string(),
                    kind: device_kind::CHROMECAST_PERSONAL.to_string(),
                    name: name_value.to_string(),
                    user_id: owner_user_id.to_string(),
                    chromecast_uuid: Some(chromecast_uuid_value.to_string()),
                    agent_id: Some(agent_id_value.to_string()),
                    last_seen_at: Some(last_seen_at_value),
                    ip: ip_value.map(ToString::to_string),
                    base_url: None,
                };
                diesel::insert_into(devices)
                    .values(&entity)
                    .get_result::<DeviceEntity>(&mut conn)
                    .map(Into::into)
                    .map_err(Into::into)
            }
        }
    }

    fn list_visible_to(&self, viewer_user_id: Uuid) -> Result<Vec<CastDevice>, Self::Error> {
        use self::devices::dsl::*;
        let mut conn = self.database.connection()?;
        let viewer = viewer_user_id.to_string();
        devices
            .filter(
                kind.eq(device_kind::CHROMECAST_SHARED)
                    .or(kind.eq(device_kind::MOPIDY_SHARED))
                    .or(kind
                        .eq(device_kind::CHROMECAST_PERSONAL)
                        .and(user_id.eq(&viewer)))
                    .or(kind
                        .eq(device_kind::MOPIDY_PERSONAL)
                        .and(user_id.eq(&viewer))),
            )
            .load::<DeviceEntity>(&mut conn)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_by_id(&self, id_to_find: Uuid) -> Result<Option<CastDevice>, Self::Error> {
        use self::devices::dsl::*;
        let mut conn = self.database.connection()?;
        match devices
            .filter(id.eq(id_to_find.to_string()))
            .first::<DeviceEntity>(&mut conn)
        {
            Ok(entity) => Ok(Some(entity.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn delete_by_id(&self, id_to_delete: Uuid) -> Result<usize, Self::Error> {
        use self::devices::dsl::*;
        let mut conn = self.database.connection()?;
        diesel::delete(devices.filter(id.eq(id_to_delete.to_string())))
            .execute(&mut conn)
            .map_err(Into::into)
    }
}

/// [`CastDeviceRepository`] with the error type the web layer works with.
pub struct CastDeviceRepositoryImpl {
    inner: DieselCastDeviceRepository,
}

impl CastDeviceRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselCastDeviceRepository::new(database),
        }
    }
}

impl CastDeviceRepository for CastDeviceRepositoryImpl {
    type Error = CustomError;

    fn register(&self, device: CastDevice) -> Result<CastDevice, CustomError> {
        self.inner.register(device).map_err(Into::into)
    }

    fn delete_owned_by(&self, owner: Uuid) -> Result<usize, CustomError> {
        self.inner.delete_owned_by(owner).map_err(Into::into)
    }

    fn list_visible_to(&self, user_id: Uuid) -> Result<Vec<CastDevice>, CustomError> {
        self.inner.list_visible_to(user_id).map_err(Into::into)
    }

    fn find_by_chromecast_uuid(
        &self,
        chromecast_uuid: &str,
    ) -> Result<Option<CastDevice>, CustomError> {
        self.inner
            .find_by_chromecast_uuid(chromecast_uuid)
            .map_err(Into::into)
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<CastDevice>, CustomError> {
        self.inner.find_by_id(id).map_err(Into::into)
    }

    fn delete_by_id(&self, id: Uuid) -> Result<usize, CustomError> {
        self.inner.delete_by_id(id).map_err(Into::into)
    }

    fn upsert_chromecast_from_agent(
        &self,
        chromecast_uuid: &str,
        agent_id: &str,
        owner_user_id: Uuid,
        name: &str,
        ip: Option<&str>,
        last_seen_at: NaiveDateTime,
    ) -> Result<CastDevice, CustomError> {
        self.inner
            .upsert_chromecast_from_agent(
                chromecast_uuid,
                agent_id,
                owner_user_id,
                name,
                ip,
                last_seen_at,
            )
            .map_err(Into::into)
    }
}
