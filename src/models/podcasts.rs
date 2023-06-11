use crate::dbconfig::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::{Queryable, Identifiable, Selectable, QueryableByName};
use diesel::{RunQueryDsl};
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text, Nullable, Bool, Timestamp};
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use crate::DbConnection;



#[derive(Queryable, Identifiable,QueryableByName, Selectable, Debug, PartialEq, Clone, ToSchema,
Serialize, Deserialize,Default)]
pub struct Podcast {
    #[diesel(sql_type = Integer)]
    pub(crate) id: i32,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = Text)]
    pub directory_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) rssfeed: String,
    #[diesel(sql_type = Text)]
    pub image_url: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub summary: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub language: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub explicit: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub keywords: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub last_build_date: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub author: Option<String>,
    #[diesel(sql_type = Bool)]
    pub active: bool,
    #[diesel(sql_type = Text)]
    pub original_image_url: String,
    #[diesel(sql_type = Text)]
    pub directory_name:String
}

impl Podcast{
    pub fn get_by_rss_feed(rssfeed_i: &str, conn: &mut DbConnection) -> Result<Podcast,
        diesel::result::Error> {
        use crate::dbconfig::schema::podcasts::dsl::*;
        podcasts
            .filter(rssfeed.eq(rssfeed_i))
            .first::<Podcast>(conn)
    }
}