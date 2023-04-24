use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use diesel::{Insertable, Queryable, QueryableByName};
use crate::schema::subscriptions;
use chrono::NaiveDateTime;
use diesel::sql_types::{Integer, Text, Nullable, Timestamp};
#[derive(Debug, Serialize, Deserialize,QueryableByName, Queryable,Insertable, Clone, ToSchema)]
pub struct Subscription{
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub device:String,
    #[diesel(sql_type = Integer)]
    pub podcast_id: i32,
    #[diesel(sql_type = Timestamp)]
    pub created: NaiveDateTime,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub deleted: Option<NaiveDateTime>
}

