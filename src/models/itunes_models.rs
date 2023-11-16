use utoipa::ToSchema;

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
