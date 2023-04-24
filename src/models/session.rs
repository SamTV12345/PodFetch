use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use utoipa::ToSchema;
use crate::schema::sessions;

#[derive(Queryable, Insertable, Clone, ToSchema, PartialEq)]
pub struct Session{
    pub username: String,
    pub session_id: String,
    pub expires: NaiveDateTime
}