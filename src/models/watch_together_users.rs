use crate::dbconfig::schema::watch_together_users;
use crate::dbconfig::DBType;
use crate::models::watch_togethers::WatchTogether;
use crate::utils::error::{map_db_error, CustomError};
use diesel::{AsChangeset, Insertable, OptionalExtension, Queryable, RunQueryDsl};
use utoipa::ToSchema;

#[derive(Queryable, Insertable, Clone, ToSchema, PartialEq, Debug, AsChangeset)]
pub struct WatchTogetherUser {
    #[diesel(sql_type = Integer,deserialize_as = i32)]
    pub id: Option<i32>,
    #[diesel(sql_type = Integer)]
    pub room_id: i32,
    #[diesel(sql_type = Text)]
    pub user: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub username: Option<String>,
    #[diesel(sql_type = Text)]
    pub status: String,
}

impl WatchTogetherUser {
    pub fn new(
        id: Option<i32>,
        room_id: i32,
        user: String,
        status: String,
        username: Option<String>,
    ) -> Self {
        WatchTogetherUser {
            id,
            room_id,
            user,
            username,
            status,
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
        room_code_to_search: String,
        connection: &mut DBType,
    ) -> Result<Option<WatchTogetherUser>, CustomError> {
        use crate::dbconfig::schema::watch_together_users::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        let watch_together_opt =
            WatchTogether::get_watch_together_by_id(room_code_to_search, connection)?;

        if watch_together_opt.is_none() {
            return Ok(None);
        }

        watch_together_users
            .filter(room_id.eq(watch_together_opt.unwrap().id.unwrap()))
            .first(connection)
            .optional()
            .map_err(map_db_error)
    }

    pub fn get_watch_together_users_by_token(
        token: String,
        connection: &mut DBType,
    ) -> Result<Option<WatchTogetherUser>, CustomError> {
        use crate::dbconfig::schema::watch_together_users::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        watch_together_users
            .filter(user.eq(token))
            .first(connection)
            .optional()
            .map_err(map_db_error)
    }

    pub fn get_watch_together_by_username(
        username_to_search: &str,
        connection: &mut DBType,
    ) -> Result<Option<WatchTogetherUser>, CustomError> {
        use crate::dbconfig::schema::watch_together_users::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        watch_together_users
            .filter(username.eq(username_to_search))
            .first(connection)
            .optional()
            .map_err(map_db_error)
    }

    pub fn update_watch_together_by_username(
        &self,
        connection: &mut DBType,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::watch_together_users::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        diesel::update(watch_together_users.filter(id.eq(self.id.clone().unwrap())))
            .set((status.eq(self.status.clone()), user.eq(self.user.clone())))
            .execute(connection)
            .map_err(map_db_error)
            .map(|_| ())
    }
}
