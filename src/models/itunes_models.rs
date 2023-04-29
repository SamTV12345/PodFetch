use std::ops::Deref;
use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::{Queryable, Identifiable, Selectable, QueryableByName};
use diesel::{RunQueryDsl, SqliteConnection};
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text, Nullable, Bool, Timestamp};
use diesel::QueryDsl;
use diesel::ExpressionMethods;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItunesModel {
    pub artist_id: Option<i64>,
    pub description: Option<String>,
    pub artist_view_url: Option<String>,
    pub kind: Option<String>,
    pub wrapper_type: String,
    pub collection_id: i64,
    pub track_id: Option<i64>,
    pub collection_censored_name: String,
    pub track_censored_name: Option<String>,
    pub artwork_url30: String,
    pub artwork_url60: String,
    pub artwork_url600: String,
    pub collection_price: f64,
    pub track_price: f64,
    pub release_date: String,
    pub collection_explicitness: String,
    pub track_explicitness: String,
    pub track_count: i32,
    pub country: String,
    pub currency: String,
    pub primary_genre_name: String,
    pub content_advisory_rating: String,
    pub feed_url: String,
    pub collection_view_url: String,
    pub collection_hd_price: f64,
    pub artist_name: String,
    pub track_name: String,
    pub collection_name: String,
    pub artwork_url_100: String,
    pub preview_url: Option<String>,
    pub track_view_url: String,
    pub track_time_millis: i64,
    pub genre_ids: Vec<String>,
    pub genres: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResponseModel {
    pub result_count: i32,
    pub results: Vec<ItunesModel>,
}

#[derive(Queryable, Identifiable,QueryableByName, Selectable, Debug, PartialEq, Clone, ToSchema,
Serialize, Deserialize,)]
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
    pub fn get_by_rss_feed(rssfeed_i: &str, conn: &mut SqliteConnection) -> Result<Podcast,
        diesel::result::Error> {
        use crate::schema::podcasts::dsl::*;
        podcasts
            .filter(rssfeed.eq(rssfeed_i))
            .first::<Podcast>(conn)
    }
}

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


#[derive(Queryable, Identifiable,QueryableByName, Selectable, Debug, PartialEq, Clone, ToSchema,
Serialize, Deserialize)]
pub struct PodcastEpisode {
    #[diesel(sql_type = Integer)]
    pub(crate) id: i32,
    #[diesel(sql_type = Integer)]
    pub(crate) podcast_id: i32,
    #[diesel(sql_type = Text)]
    pub(crate) episode_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = Text)]
    pub(crate) url: String,
    #[diesel(sql_type = Text)]
    pub(crate) date_of_recording: String,
    #[diesel(sql_type = Text)]
    pub image_url: String,
    #[diesel(sql_type = Integer)]
    pub total_time: i32,
    #[diesel(sql_type = Text)]
    pub(crate) local_url: String,
    #[diesel(sql_type = Text)]
    pub(crate) local_image_url: String,
    #[diesel(sql_type = Text)]
    pub(crate) description: String,
    #[diesel(sql_type = Text)]
    pub(crate) status: String,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub(crate) download_time: Option<NaiveDateTime>
}
