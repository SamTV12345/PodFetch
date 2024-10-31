use crate::dbconfig::schema::watch_together_users;
use crate::dbconfig::DBType;
use crate::utils::error::{map_db_error, CustomError};
use diesel::{AsChangeset, Insertable, OptionalExtension, Queryable, RunQueryDsl};
use utoipa::ToSchema;

#[derive(Queryable, Insertable, Clone, ToSchema, PartialEq, Debug, AsChangeset)]
pub struct WatchTogetherUser {
   #[diesel(sql_type=Text)]
   pub subject: String,
    #[diesel(sql_type=Text)]
    pub name: Option<String>
}

impl WatchTogetherUser {
    pub fn new(
        subject: String,
        name: Option<String>
    ) -> Self {
        WatchTogetherUser {
            subject,
            name,
        }
    }

    pub fn save_watch_together_users(&self, connection: &mut DBType) -> Result<(), CustomError> {
        use crate::dbconfig::schema::watch_together_users;

        diesel::insert_into(watch_together_users::table)
            .values(self)
            .execute(connection)
            .map_err(map_db_error)
            .map(|_| ())
    }

    pub fn get_watch_together_users_by_id(
        subject_to_find: String,
        connection: &mut DBType,
    ) -> Result<Option<WatchTogetherUser>, CustomError> {
        use crate::dbconfig::schema::watch_together_users::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        watch_together_users
            .filter(subject.eq(subject_to_find))
            .first(connection)
            .optional()
            .map_err(map_db_error)
    }
}
