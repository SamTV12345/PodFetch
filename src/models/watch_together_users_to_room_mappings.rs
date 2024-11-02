use crate::dbconfig::schema::watch_together_users_to_room_mappings::dsl::watch_together_users_to_room_mappings;
use std::fmt::{Display, Formatter};
use diesel::{AsChangeset, Insertable, OptionalExtension, Queryable, RunQueryDsl};
use diesel::associations::HasTable;
use utoipa::ToSchema;
use crate::dbconfig::DBType;
use crate::utils::error::{map_db_error, CustomError};

#[derive(Queryable, Insertable, Clone, ToSchema, PartialEq, Debug, AsChangeset)]
pub struct WatchTogetherUsersToRoomMapping {
    #[diesel(sql_type = Integer,deserialize_as = i32)]
    pub room_id: i32,
    #[diesel(sql_type=String)]
    pub subject: String,
    #[diesel(sql_type=String)]
    pub status: String,
    #[diesel(sql_type=String)]
    pub role: String
}


impl WatchTogetherUsersToRoomMapping {

    pub(crate) fn get_by_user_and_room_id(subject_to_find: &str, room_id_to_search: &str, conn:
    &mut DBType)
        ->
                                                                             Result<Option<WatchTogetherUsersToRoomMapping>, CustomError> {
        use crate::dbconfig::schema::watch_together_users_to_room_mappings::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;


        watch_together_users_to_room_mappings
            .filter(subject.eq(subject_to_find))
            .filter(room_id.eq(room_id_to_search))
            .first::<WatchTogetherUsersToRoomMapping>(conn)
            .optional()
            .map_err(map_db_error)
    }

    fn insert_watch_together_user_to_room_mapping(&self, conn: &mut DBType) -> Result<(), CustomError> {
        use crate::dbconfig::schema::watch_together_users_to_room_mappings;

        diesel::insert_into(watch_together_users_to_room_mappings::table)
            .values(self)
            .execute(conn)
            .map_err(map_db_error)
            .map(|_| ())
    }

    fn update_watch_together_user_to_room_mapping(&self, conn: &mut DBType) -> Result<(), CustomError> {
        use crate::dbconfig::schema::watch_together_users_to_room_mappings::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        diesel::update(watch_together_users_to_room_mappings::table)
            .filter(subject.eq(&self.subject))
            .filter(room_id.eq(&self.room_id))
            .set(status.eq(&self.status))
            .execute(conn)
            .map_err(map_db_error)
            .map(|_| ())
    }

    pub(crate) fn delete_mappings_by_room_id(room_id_to_search: i32, conn: &mut DBType) -> Result<(),
        CustomError> {
        use crate::dbconfig::schema::watch_together_users_to_room_mappings::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        diesel::delete(watch_together_users_to_room_mappings::table)
            .filter(room_id.eq(room_id_to_search))
            .execute(conn)
            .map_err(map_db_error)
            .map(|_| ())
    }
}


pub enum WatchTogetherStatus {
    Pending,
    Accepted,
    Rejected,
    User,
    Admin
}


impl Display for WatchTogetherStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WatchTogetherStatus::Pending => write!(f, "Pending"),
            WatchTogetherStatus::Accepted => write!(f, "Accepted"),
            WatchTogetherStatus::Rejected => write!(f, "Rejected"),
            WatchTogetherStatus::User => write!(f, "User"),
            WatchTogetherStatus::Admin => write!(f, "Admin"),
        }
    }
}


impl WatchTogetherStatus {
    fn from_string(status: &str) -> WatchTogetherStatus {
        match status {
            "Pending" => WatchTogetherStatus::Pending,
            "Accepted" => WatchTogetherStatus::Accepted,
            "Rejected" => WatchTogetherStatus::Rejected,
            "User" => WatchTogetherStatus::User,
            "Admin" => WatchTogetherStatus::Admin,
            _ => WatchTogetherStatus::Pending
        }
    }
}