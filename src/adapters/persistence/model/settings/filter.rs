use crate::utils::error::{map_db_error, CustomError};
use crate::execute_with_conn;
use diesel::AsChangeset;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::Queryable;
use diesel::{Insertable, OptionalExtension, RunQueryDsl};
use utoipa::ToSchema;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::filters;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset, Queryable, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FilterEntity {
    pub username: String,
    pub title: Option<String>,
    pub ascending: bool,
    pub filter: Option<String>,
    pub only_favored: bool,
}


