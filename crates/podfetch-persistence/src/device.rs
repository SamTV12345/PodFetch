use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::BoolExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{ExpressionMethods, QueryDsl};
use podfetch_domain::device::{Device, DeviceRepository, kind as device_kind};
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

    fn find_by_chromecast_uuid(
        &self,
        chromecast_uuid_to_find: &str,
    ) -> Result<Option<Device>, Self::Error> {
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
    ) -> Result<Device, Self::Error> {
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
                    id: Some(podfetch_domain::ids::new_id().to_string()),
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

    fn list_castable_for_user(&self, viewer_user_id: Uuid) -> Result<Vec<Device>, Self::Error> {
        use self::devices::dsl::*;
        let mut conn = self.database.connection()?;
        let viewer = viewer_user_id.to_string();
        devices
            .filter(
                kind.eq(device_kind::CHROMECAST_SHARED)
                    .or(kind.eq(device_kind::MOPIDY_SHARED))
                    .or(kind.eq(device_kind::CHROMECAST_PERSONAL).and(user_id.eq(&viewer)))
                    .or(kind.eq(device_kind::MOPIDY_PERSONAL).and(user_id.eq(&viewer))),
            )
            .load::<DeviceEntity>(&mut conn)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_by_id(&self, id_to_find: Uuid) -> Result<Option<Device>, Self::Error> {
        use self::devices::dsl::*;
        let mut conn = self.database.connection()?;
        match devices.filter(id.eq(id_to_find.to_string())).first::<DeviceEntity>(&mut conn) {
            Ok(entity) => Ok(Some(entity.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn delete_by_id(&self, id_to_delete: Uuid) -> Result<usize, Self::Error> {
        use self::devices::dsl::*;
        let mut conn = self.database.connection()?;
        diesel::delete(devices.filter(id.eq(id_to_delete.to_string()))).execute(&mut conn).map_err(Into::into)
    }
}

#[cfg(test)]
mod mopidy_persistence_tests {
    use super::*;
    use crate::db::{database, run_migrations};
    use podfetch_domain::device::kind as device_kind;

    mod seed_schema {
        diesel::table! {
            users (id) { id -> Text, username -> Text, role -> Text, }
        }
    }

    #[derive(diesel::Insertable)]
    #[diesel(table_name = seed_schema::users)]
    struct SeedUser { id: String, username: String, role: String }

    fn seed_user() -> Uuid {
        use seed_schema::users;
        let owner = podfetch_domain::ids::new_id();
        let mut conn = database().connection().expect("db connection");
        diesel::insert_into(users::table)
            .values(SeedUser {
                id: owner.to_string(),
                username: format!("mopidy-test-{owner}"),
                role: "user".to_string(),
            })
            .execute(&mut conn)
            .expect("seed user");
        owner
    }

    fn mopidy_device(owner: Uuid, kind_str: &str, url: &str) -> Device {
        Device {
            id: None,
            deviceid: url.to_string(),
            kind: kind_str.to_string(),
            name: "Living Room".to_string(),
            user_id: owner,
            chromecast_uuid: Some(podfetch_domain::ids::new_id().to_string()),
            agent_id: None,
            last_seen_at: None,
            ip: None,
            base_url: Some(url.to_string()),
        }
    }

    #[test]
    fn create_persists_base_url_and_list_castable_includes_shared_mopidy() {
        run_migrations();
        let repo = DieselDeviceRepository::new(database());
        let owner = seed_user();
        let viewer = seed_user();

        let created = repo
            .create(mopidy_device(owner, device_kind::MOPIDY_SHARED, "http://m.local:6680"))
            .expect("create mopidy device");
        assert_eq!(created.base_url.as_deref(), Some("http://m.local:6680"));

        let castable = repo.list_castable_for_user(viewer).expect("list castable");
        assert!(castable.iter().any(|d| d.id == created.id));

        let found = repo.find_by_id(created.id.unwrap()).expect("find").expect("present");
        assert_eq!(found.base_url, created.base_url);
        assert_eq!(repo.delete_by_id(created.id.unwrap()).expect("delete"), 1);
        assert!(repo.find_by_id(created.id.unwrap()).expect("find again").is_none());
    }

    #[test]
    fn list_castable_hides_personal_mopidy_from_other_user() {
        run_migrations();
        let repo = DieselDeviceRepository::new(database());
        let owner = seed_user();
        let other = seed_user();

        let created = repo
            .create(mopidy_device(owner, device_kind::MOPIDY_PERSONAL, "http://personal.local:6680"))
            .expect("create personal mopidy device");

        // Owner A sees their own personal device.
        let castable_owner = repo.list_castable_for_user(owner).expect("list castable owner");
        assert!(
            castable_owner.iter().any(|d| d.id == created.id),
            "owner should see their own personal device"
        );

        // A different user B must NOT see A's personal device.
        let castable_other = repo.list_castable_for_user(other).expect("list castable other");
        assert!(
            !castable_other.iter().any(|d| d.id == created.id),
            "personal device must not be visible to a different user"
        );

        repo.delete_by_id(created.id.unwrap()).expect("delete");
    }
}
