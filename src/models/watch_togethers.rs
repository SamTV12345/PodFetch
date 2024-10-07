use diesel::{AsChangeset, Insertable, OptionalExtension, Queryable, RunQueryDsl};
use utoipa::ToSchema;
use crate::controllers::watch_together_dto::WatchTogetherDto;
use crate::dbconfig::DBType;
use crate::utils::error::{map_db_error, CustomError};
use crate::dbconfig::schema::watch_togethers;

#[derive(
    Queryable, Insertable, Clone, ToSchema, PartialEq, Debug, AsChangeset,
)]
pub struct WatchTogether {
    #[diesel(sql_type = Integer)]
    id : i32,
    #[diesel(sql_type = Text)]
    room_id: String,
    #[diesel(sql_type = Text)]
    admin: String,
    #[diesel(sql_type = Text)]
    room_name: String
}

impl Into<WatchTogetherDto> for WatchTogether {
    fn into(self) -> WatchTogetherDto {
        WatchTogetherDto {
            room_id: self.room_id,
            admin: self.admin,
            room_name: self.room_name
        }
    }
}

impl WatchTogether {
    pub fn new(id: i32, room_id: &String, admin: String, room_name: String) -> Self {
        WatchTogether {
            id,
            room_id: room_id.to_string(),
            admin,
            room_name
        }
    }

    pub fn save_watch_together(&self, connection: &mut DBType) -> Result<(), CustomError> {
        use crate::dbconfig::schema::watch_togethers;

        diesel::insert_into(watch_togethers::table)
            .values(self)
            .execute(connection)
            .map_err(map_db_error)
            .map(|_| ())
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

    pub fn get_watch_together_by_id(room_code_to_search: String, connection: &mut DBType) ->
                                                                     Result<Option<WatchTogether>, CustomError> {
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