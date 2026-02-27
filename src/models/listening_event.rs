use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::listening_events;
use crate::utils::error::ErrorSeverity::Critical;
use crate::utils::error::{CustomError, map_db_error};
use chrono::NaiveDateTime;
use diesel::ExpressionMethods;
use diesel::Insertable;
use diesel::QueryDsl;
use diesel::Queryable;
use diesel::QueryableByName;
use diesel::RunQueryDsl;
use diesel::Selectable;
use diesel::sql_types::{Integer, Text, Timestamp};
use utoipa::ToSchema;

#[derive(
    Serialize, Deserialize, Clone, Debug, Queryable, QueryableByName, Selectable, ToSchema,
)]
#[diesel(table_name = listening_events)]
pub struct ListeningEvent {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub device: String,
    #[diesel(sql_type = Text)]
    pub podcast_episode_id: String,
    #[diesel(sql_type = Integer)]
    pub podcast_id: i32,
    #[diesel(sql_type = Integer)]
    pub podcast_episode_db_id: i32,
    #[diesel(sql_type = Integer)]
    pub delta_seconds: i32,
    #[diesel(sql_type = Integer)]
    pub start_position: i32,
    #[diesel(sql_type = Integer)]
    pub end_position: i32,
    #[diesel(sql_type = Timestamp)]
    pub listened_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Insertable)]
#[diesel(table_name = listening_events)]
pub struct NewListeningEvent {
    pub username: String,
    pub device: String,
    pub podcast_episode_id: String,
    pub podcast_id: i32,
    pub podcast_episode_db_id: i32,
    pub delta_seconds: i32,
    pub start_position: i32,
    pub end_position: i32,
    pub listened_at: NaiveDateTime,
}

impl ListeningEvent {
    pub fn insert_event(event: NewListeningEvent) -> Result<ListeningEvent, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::listening_events::dsl::listening_events;
        diesel::insert_into(listening_events)
            .values(event)
            .get_result::<ListeningEvent>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))
    }

    pub fn get_by_user_and_range(
        username_to_search: &str,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
    ) -> Result<Vec<ListeningEvent>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::listening_events::dsl as le_dsl;
        use crate::adapters::persistence::dbconfig::schema::listening_events::table as le_table;

        let mut query = le_table
            .filter(le_dsl::username.eq(username_to_search))
            .into_boxed();

        if let Some(from) = from {
            query = query.filter(le_dsl::listened_at.ge(from));
        }

        if let Some(to) = to {
            query = query.filter(le_dsl::listened_at.le(to));
        }

        query
            .order_by(le_dsl::listened_at.asc())
            .load::<ListeningEvent>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))
    }
}
