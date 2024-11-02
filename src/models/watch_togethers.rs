use crate::controllers::watch_together_dto::WatchTogetherDto;
use crate::dbconfig::schema::watch_togethers;
use crate::dbconfig::DBType;
use crate::models::user::User;
use crate::utils::error::{map_db_error, CustomError};
use diesel::{AsChangeset, ExpressionMethods, Insertable, JoinOnDsl, OptionalExtension, QueryDsl, Queryable, RunQueryDsl, Table};
use utoipa::ToSchema;
use crate::dbconfig::schema::watch_together_users_to_room_mappings::dsl::watch_together_users_to_room_mappings;
use crate::models::watch_together_users_to_room_mappings::{WatchTogetherStatus, WatchTogetherUsersToRoomMapping};

#[derive(Queryable, Insertable, Clone, ToSchema, PartialEq, Debug, AsChangeset)]
pub struct WatchTogether {
    #[diesel(sql_type = Integer,deserialize_as = i32)]
    pub id: Option<i32>,
    #[diesel(sql_type = Text)]
    pub room_id: String,
    #[diesel(sql_type = Text)]
    pub room_name: String,
}

impl Into<WatchTogetherDto> for WatchTogether {
    fn into(self) -> WatchTogetherDto {
        WatchTogetherDto {
            room_id: self.room_id,
            room_name: self.room_name,
        }
    }
}

impl WatchTogether {
    pub fn new(id: Option<i32>, room_id: &String, room_name: String) -> Self {
        WatchTogether {
            id,
            room_id: room_id.to_string(),
            room_name,
        }
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
        watch_together_room_id_to_search: &str,
        connection: &mut DBType,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::watch_togethers::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        let watch_together = Self::get_watch_together_by_id(&watch_together_room_id_to_search,
                                                     connection)?;
        if watch_together.is_none() {
            return Ok(());
        }

        // Delete mappings
        WatchTogetherUsersToRoomMapping::delete_mappings_by_room_id(watch_together.unwrap().id.unwrap(), connection)?;

        diesel::delete(
            watch_togethers.filter(
                room_id
                    .eq(watch_together_room_id_to_search)
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


    pub fn get_watch_together_by_admin(
        admin: i32,
        connection: &mut DBType,
    ) -> Result<Vec<WatchTogether>, CustomError> {
        use crate::dbconfig::schema::watch_togethers::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use crate::dbconfig::schema::watch_togethers::room_id as w_t_room_id;
        use crate::dbconfig::schema::watch_together_users_to_room_mappings::room_id as
        w_t_m_room_id;
        use crate::dbconfig::schema::watch_together_users_to_room_mappings::status as w_t_u_mapping_status;


        watch_togethers
            .inner_join(watch_together_users_to_room_mappings.on(w_t_room_id.eq(w_t_m_room_id)))
            .filter(room_name.eq(admin))
            .filter(w_t_u_mapping_status.eq(WatchTogetherStatus::Admin.to_string()))
            .select(watch_togethers::all_columns())
            .load(connection)
            .map_err(map_db_error)
    }
}
