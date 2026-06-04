use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable, QueryableByName};
use diesel::sql_types::{Nullable, Text};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::gpodder_setting::{GpodderSetting, GpodderSettingRepository};
use uuid::Uuid;

diesel::table! {
    gpodder_settings (id) {
        id -> Text,
        user_id -> Text,
        scope -> Text,
        scope_id -> Nullable<Text>,
        data -> Text,
    }
}

#[derive(Debug, Clone, Queryable, QueryableByName, Insertable)]
#[diesel(table_name = gpodder_settings)]
struct GpodderSettingEntity {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    user_id: String,
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
    id: String,
    user_id: String,
    scope: String,
    scope_id: Option<String>,
    data: String,
}

impl From<GpodderSettingEntity> for GpodderSetting {
    fn from(value: GpodderSettingEntity) -> Self {
        Self {
            id: Uuid::parse_str(&value.id).expect("valid uuid in db"),
            user_id: Uuid::parse_str(&value.user_id).expect("valid uuid in db"),
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
        user_id_to_find: Uuid,
        scope: &str,
        scope_id: Option<&str>,
    ) -> Result<Option<GpodderSetting>, Self::Error> {
        use self::gpodder_settings::dsl as gs_dsl;

        let mut query = gs_dsl::gpodder_settings
            .filter(gs_dsl::user_id.eq(user_id_to_find.to_string()))
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
            .filter(gs_dsl::user_id.eq(setting.user_id.to_string()))
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
                diesel::update(gs_dsl::gpodder_settings.filter(gs_dsl::id.eq(existing.id.clone())))
                    .set(gs_dsl::data.eq(&setting.data))
                    .execute(&mut connection)
                    .map_err(PersistenceError::from)?;

                Ok(GpodderSetting {
                    id: Uuid::parse_str(&existing.id).expect("valid uuid in db"),
                    user_id: setting.user_id,
                    scope: setting.scope,
                    scope_id: setting.scope_id,
                    data: setting.data,
                })
            }
            None => {
                diesel::insert_into(gs_dsl::gpodder_settings)
                    .values(NewGpodderSettingEntity {
                        id: podfetch_domain::ids::new_id().to_string(),
                        user_id: setting.user_id.to_string(),
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
