use chrono::NaiveDateTime;
use diesel::{AsChangeset, Insertable, Queryable};
use utoipa::ToSchema;

#[derive(
   Queryable, Insertable, Clone, ToSchema, PartialEq, Debug, AsChangeset,
)]
pub struct UserEntity {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub password: Option<String>,
    pub explicit_consent: bool,
    pub created_at: NaiveDateTime,
    pub api_key: Option<String>,
}