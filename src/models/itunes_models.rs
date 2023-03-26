use diesel::prelude::{Insertable, Queryable};
use crate::schema::*;
use utoipa::{ToSchema};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize,Clone, ToSchema)]
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

#[derive(Serialize, Deserialize, Queryable, Insertable, Clone, ToSchema)]
pub struct Podcast {
    #[diesel(sql_type = Integer)]
    pub(crate) id: i32,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = Text)]
    pub directory: String,
    #[diesel(sql_type = Text)]
    pub(crate) rssfeed: String,
    #[diesel(sql_type = Text)]
    pub image_url: String,
    #[diesel(sql_type = Text)]
    pub favored: i32,
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
    pub active: bool
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Clone, Debug, ToSchema)]
pub struct PodcastEpisode {
    pub(crate) id: i32,
    pub(crate) podcast_id: i32,
    pub(crate) episode_id: String,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) date_of_recording: String,
    pub image_url: String,
    pub total_time: i32,
    pub(crate) local_url: String,
    pub(crate) local_image_url: String,
    pub (crate) description: String,
    pub (crate) status: String,
    pub (crate) download_time: Option<NaiveDateTime>,
}