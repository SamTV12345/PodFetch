use chrono::{NaiveDateTime, Utc};
use diesel::{AsChangeset, Insertable, JoinOnDsl, OptionalExtension, Queryable, QueryableByName, RunQueryDsl, Table};
use utoipa::ToSchema;
use crate::utils::error::{CustomError, map_db_error};
use diesel::sql_types::{Text,Nullable, Timestamp };
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::tags;
use crate::execute_with_conn;

pub struct Tag {
    pub(crate) id: String,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub color: String,
}

impl Tag {
    pub fn new(name: String, description: Option<String>, color: String, username: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            username,
            description,
            created_at: Utc::now().naive_utc(),
            color,
        }
    }
}
