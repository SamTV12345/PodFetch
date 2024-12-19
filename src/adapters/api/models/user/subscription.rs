use chrono::NaiveDateTime;
use diesel::{AsChangeset, Insertable, Queryable, QueryableByName};
use utoipa::ToSchema;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    QueryableByName,
    Queryable,
    AsChangeset,
    Insertable,
    Clone,
    ToSchema,
)]
#[diesel(treat_none_as_null = true)]
pub struct Subscription {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub device: String,
    #[diesel(sql_type = Text)]
    pub podcast: String,
    #[diesel(sql_type = Timestamp)]
    pub created: NaiveDateTime,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub deleted: Option<NaiveDateTime>,
}