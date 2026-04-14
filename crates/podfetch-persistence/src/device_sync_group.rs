use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable, QueryableByName};
use diesel::sql_types::{Integer, Text};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_domain::device_sync_group::{DeviceSyncGroup, DeviceSyncGroupRepository};

diesel::table! {
    device_sync_groups (id) {
        id -> Integer,
        user_id -> Integer,
        group_id -> Integer,
        device_id -> Text,
    }
}

#[derive(Debug, Clone, Queryable, QueryableByName, Insertable)]
#[diesel(table_name = device_sync_groups)]
struct DeviceSyncGroupEntity {
    #[diesel(sql_type = Integer)]
    id: i32,
    #[diesel(sql_type = Integer)]
    user_id: i32,
    #[diesel(sql_type = Integer)]
    group_id: i32,
    #[diesel(sql_type = Text)]
    device_id: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = device_sync_groups)]
struct NewDeviceSyncGroupEntity {
    user_id: i32,
    group_id: i32,
    device_id: String,
}

impl From<DeviceSyncGroupEntity> for DeviceSyncGroup {
    fn from(value: DeviceSyncGroupEntity) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            group_id: value.group_id,
            device_id: value.device_id,
        }
    }
}

pub struct DieselDeviceSyncGroupRepository {
    database: Database,
}

impl DieselDeviceSyncGroupRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl DeviceSyncGroupRepository for DieselDeviceSyncGroupRepository {
    type Error = PersistenceError;

    fn get_by_user_id(&self, user_id_to_find: i32) -> Result<Vec<DeviceSyncGroup>, Self::Error> {
        use self::device_sync_groups::dsl as dsg_dsl;

        dsg_dsl::device_sync_groups
            .filter(dsg_dsl::user_id.eq(user_id_to_find))
            .load::<DeviceSyncGroupEntity>(&mut self.database.connection()?)
            .map(|groups| groups.into_iter().map(DeviceSyncGroup::from).collect())
            .map_err(Into::into)
    }

    fn replace_all(
        &self,
        user_id_to_replace: i32,
        groups: Vec<DeviceSyncGroup>,
    ) -> Result<(), Self::Error> {
        use self::device_sync_groups::dsl as dsg_dsl;

        let mut connection = self.database.connection()?;

        diesel::delete(dsg_dsl::device_sync_groups.filter(dsg_dsl::user_id.eq(user_id_to_replace)))
            .execute(&mut connection)
            .map_err(PersistenceError::from)?;

        for group in groups {
            diesel::insert_into(dsg_dsl::device_sync_groups)
                .values(NewDeviceSyncGroupEntity {
                    user_id: group.user_id,
                    group_id: group.group_id,
                    device_id: group.device_id,
                })
                .execute(&mut connection)
                .map_err(PersistenceError::from)?;
        }

        Ok(())
    }
}
