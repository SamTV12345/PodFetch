use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable, QueryableByName};
use diesel::sql_types::{Integer, Nullable, Text};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::gpodder_setting::{GpodderSetting, GpodderSettingRepository};

diesel::table! {
    gpodder_settings (id) {
        id -> Integer,
        username -> Text,
        scope -> Text,
        scope_id -> Nullable<Text>,
        data -> Text,
    }
}

#[derive(Debug, Clone, Queryable, QueryableByName, Insertable)]
#[diesel(table_name = gpodder_settings)]
struct GpodderSettingEntity {
    #[diesel(sql_type = Integer)]
    id: i32,
    #[diesel(sql_type = Text)]
    username: String,
    #[diesel(sql_type = Text)]
    scope: String,
    #[diesel(sql_type = Nullable<Text>)]
    scope_id: Option<String>,
    #[diesel(sql_type = Text)]
    data: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = gpodder_settings)]
struct NewGpodderSettingEntity {
    username: String,
    scope: String,
    scope_id: Option<String>,
    data: String,
}

impl From<GpodderSettingEntity> for GpodderSetting {
    fn from(value: GpodderSettingEntity) -> Self {
        Self {
            id: value.id,
            username: value.username,
            scope: value.scope,
            scope_id: value.scope_id,
            data: value.data,
        }
    }
}

pub struct DieselGpodderSettingRepository {
    database: Database,
}

impl DieselGpodderSettingRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl GpodderSettingRepository for DieselGpodderSettingRepository {
    type Error = PersistenceError;

    fn get_setting(
        &self,
        username: &str,
        scope: &str,
        scope_id: Option<&str>,
    ) -> Result<Option<GpodderSetting>, Self::Error> {
        use self::gpodder_settings::dsl as gs_dsl;

        let mut query = gs_dsl::gpodder_settings
            .filter(gs_dsl::username.eq(username))
            .filter(gs_dsl::scope.eq(scope))
            .into_boxed();

        match scope_id {
            Some(sid) => {
                query = query.filter(gs_dsl::scope_id.eq(sid));
            }
            None => {
                query = query.filter(gs_dsl::scope_id.is_null());
            }
        }

        query
            .first::<GpodderSettingEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(GpodderSetting::from))
            .map_err(Into::into)
    }

    fn save_setting(&self, setting: GpodderSetting) -> Result<GpodderSetting, Self::Error> {
        use self::gpodder_settings::dsl as gs_dsl;

        let mut connection = self.database.connection()?;

        let mut query = gs_dsl::gpodder_settings
            .filter(gs_dsl::username.eq(&setting.username))
            .filter(gs_dsl::scope.eq(&setting.scope))
            .into_boxed();

        match &setting.scope_id {
            Some(sid) => {
                query = query.filter(gs_dsl::scope_id.eq(sid));
            }
            None => {
                query = query.filter(gs_dsl::scope_id.is_null());
            }
        }

        let existing = query
            .first::<GpodderSettingEntity>(&mut connection)
            .optional()
            .map_err(PersistenceError::from)?;

        match existing {
            Some(existing) => {
                diesel::update(gs_dsl::gpodder_settings.filter(gs_dsl::id.eq(existing.id)))
                    .set(gs_dsl::data.eq(&setting.data))
                    .execute(&mut connection)
                    .map_err(PersistenceError::from)?;

                Ok(GpodderSetting {
                    id: existing.id,
                    username: setting.username,
                    scope: setting.scope,
                    scope_id: setting.scope_id,
                    data: setting.data,
                })
            }
            None => {
                diesel::insert_into(gs_dsl::gpodder_settings)
                    .values(NewGpodderSettingEntity {
                        username: setting.username.clone(),
                        scope: setting.scope.clone(),
                        scope_id: setting.scope_id.clone(),
                        data: setting.data.clone(),
                    })
                    .execute(&mut connection)
                    .map_err(PersistenceError::from)?;

                Ok(setting)
            }
        }
    }
}
