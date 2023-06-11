use crate::dbconfig::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::{Queryable, Identifiable, Selectable, QueryableByName};
use diesel::{RunQueryDsl};
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text, Nullable, Bool, Timestamp};
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use crate::DbConnection;

#[derive(Serialize, Deserialize)]
pub struct PodcastDto {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub directory_id: String,
    pub(crate) rssfeed: String,
    pub image_url: String,
    pub summary: Option<String>,
    pub language: Option<String>,
    pub explicit: Option<String>,
    pub keywords: Option<String>,
    pub last_build_date: Option<String>,
    pub author: Option<String>,
    pub active: bool,
    pub original_image_url: String,
    pub favorites: bool
}