use crate::controllers::watch_together_dto::WatchTogetherDto;
use crate::dbconfig::schema::watch_together_users::user;
use crate::dbconfig::schema::watch_togethers;
use crate::dbconfig::DBType;
use crate::models::user::User;
use crate::utils::error::{map_db_error, CustomError};
use diesel::{
    AsChangeset, BoolExpressionMethods, ExpressionMethods, Insertable, OptionalExtension, QueryDsl,
    Queryable, RunQueryDsl,
};
use utoipa::ToSchema;
use crate::models::watch_together_users::WatchTogetherUser;

#[derive(Queryable, Insertable, Clone, ToSchema, PartialEq, Debug, AsChangeset)]
pub struct WatchTogether {
    #[diesel(sql_type = Integer,deserialize_as = i32)]
    pub id: Option<i32>,
    #[diesel(sql_type = Text)]
    pub room_id: String,
    #[diesel(sql_type = Text)]
    pub admin: String,
    #[diesel(sql_type = Text)]
    pub room_name: String,
}

impl Into<WatchTogetherDto> for WatchTogether {
    fn into(self) -> WatchTogetherDto {
        WatchTogetherDto {
            room_id: self.room_id,
            admin: self.admin,
            room_name: self.room_name,
        }
    }
}

impl WatchTogether {
    pub fn new(id: Option<i32>, room_id: &String, admin: String, room_name: String) -> Self {
        WatchTogether {
            id,
            room_id: room_id.to_string(),
            admin,
            room_name,
        }
    }

    pub(crate) fn get_watch_together_by_admin(
        admin_to_search: String,
        conn: &mut DBType,
    ) -> Result<Vec<WatchTogether>, CustomError> {
        use crate::dbconfig::schema::watch_togethers::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        watch_togethers
            .filter(admin.eq(admin_to_search))
            .load::<WatchTogether>(conn)
            .map_err(map_db_error)
    }

    pub fn save_watch_together(
        &self,
        connection: &mut DBType,
    ) -> Result<WatchTogether, CustomError> {
        use crate::dbconfig::schema::watch_togethers;

        diesel::insert_into(watch_togethers::table)
            .values(self)
            .get_result::<WatchTogether>(connection)
            .map_err(map_db_error)
    }

    pub fn random_room_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut room_id = "#".to_string();
        for _ in 0..10 {
            let room_id_seq = rng.gen_range(0..10);
            room_id.push_str(&room_id_seq.to_string());
        }

        room_id
    }

    pub fn delete_watch_together(
        watch_together_user_id: i32,
        watch_together_room_id_to_search: String,
        connection: &mut DBType,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::watch_togethers::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        let user_found = User::get_user_by_userid(watch_together_user_id, connection)?;
        let watch_together = Self::get_watch_together_by_id(&watch_together_room_id_to_search,
                                                     connection)?;
        if watch_together.is_none() {
            return Ok(());
        }

        WatchTogetherUser::delete_by_room_id(watch_together.unwrap().id.unwrap(), connection)?;

        diesel::delete(
            watch_togethers.filter(
                room_id
                    .eq(watch_together_room_id_to_search)
                    .and(admin.eq(user_found.username)),
            ),
        )
        .execute(connection)
        .map_err(map_db_error)
        .map(|_| ())
    }

    pub fn get_watch_together_by_id(
        room_code_to_search: &str,
        connection: &mut DBType,
    ) -> Result<Option<WatchTogether>, CustomError> {
        use crate::dbconfig::schema::watch_togethers::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        watch_togethers
            .filter(room_id.eq(room_code_to_search))
            .first(connection)
            .optional()
            .map_err(map_db_error)
    }
}
