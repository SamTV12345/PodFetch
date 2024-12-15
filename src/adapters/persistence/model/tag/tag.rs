use chrono::NaiveDateTime;
use diesel::{AsChangeset, Insertable, Queryable, QueryableByName};
use utoipa::ToSchema;

#[derive(
    Debug,
    QueryableByName,
    Queryable,
    AsChangeset,
    Insertable,
    Clone,
)]
#[diesel(treat_none_as_null = true)]
pub struct TagEntity {
    #[diesel(sql_type = Text)]
    pub(crate) id: String,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = Timestamp)]
    pub created_at: NaiveDateTime,
    #[diesel(sql_type = Text)]
    pub color: String,
}


