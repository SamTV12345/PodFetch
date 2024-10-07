use diesel::{AsChangeset, Insertable, OptionalExtension, Queryable, RunQueryDsl};
use utoipa::ToSchema;
use crate::dbconfig::DBType;
use crate::utils::error::{map_db_error, CustomError};
use crate::dbconfig::schema::watch_together_users;

#[derive(
    Queryable, Insertable, Clone, ToSchema, PartialEq, Debug, AsChangeset,
)]
pub struct WatchTogetherUser {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Text)]
    pub room_id: String,
    #[diesel(sql_type = Text)]
    pub user: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub username: Option<String>,
    #[diesel(sql_type = Text)]
    pub status: String
}

impl WatchTogetherUser {
    pub fn new(id: i32, room_id: String, user: String, status: String, username: Option<String>) ->
                                                                                            Self {
        WatchTogetherUser {
            id,
            room_id,
            user,
            username,
            status
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

    pub fn get_watch_together_users_by_id(room_code_to_search: String, connection: &mut DBType) ->
                                                                     Result<Option<WatchTogetherUser>, CustomError> {
        use crate::dbconfig::schema::watch_together_users::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        watch_together_users
            .filter(room_id.eq(room_code_to_search))
            .first(connection)
            .optional()
            .map_err(map_db_error)
    }

    pub fn get_watch_together_users_by_token(token: String, connection: &mut DBType) ->
                                                                     Result<Option<WatchTogetherUser>, CustomError> {
        use crate::dbconfig::schema::watch_together_users::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        watch_together_users
            .filter(user.eq(token))
            .first(connection)
            .optional()
            .map_err(map_db_error)
    }

    pub fn get_watch_together_by_username(username_to_search: &str, connection: &mut DBType) ->
                                                                     Result<Option<WatchTogetherUser>, CustomError> {
        use crate::dbconfig::schema::watch_together_users::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;
        watch_together_users
            .filter(username.eq(username_to_search))
            .first(connection)
            .optional()
            .map_err(map_db_error)
    }

    pub fn update_watch_together_by_username(&self, connection: &mut DBType) -> Result<(), CustomError> {
        use crate::dbconfig::schema::watch_together_users::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        diesel::update(watch_together_users.filter(id.eq(self.id.clone())))
            .set((status.eq(self.status.clone()),
                            user.eq(self.user.clone())))
            .execute(connection)
            .map_err(map_db_error)
            .map(|_| ())
    }
}