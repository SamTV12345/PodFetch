use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ItunesWrapper {
    result_count: i32,
    results: Vec<ItunesModel>,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodindexResponse {
    pub status: bool,
    pub feeds: Vec<Feed>,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum PodcastSearchReturn {
    Itunes(ItunesWrapper),
    Podindex(PodindexResponse),
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    id: Option<i32>,
    podcast_guid: Option<String>,
    title: Option<String>,
    url: Option<String>,
    original_url: Option<String>,
    link: Option<String>,
    description: Option<String>,
    author: Option<String>,
    owner_name: Option<String>,
    image: Option<String>,
    artwork: Option<String>,
    last_update_time: Option<i32>,
    last_crawl_time: Option<i32>,
    last_parse_time: Option<i32>,
    last_good_http_status_time: Option<i32>,
    explicit: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItunesModel {
    pub artist_id: Option<i64>,
    pub description: Option<String>,
    pub artist_view_url: Option<String>,
    pub kind: Option<String>,
    pub wrapper_type: Option<String>,
    pub collection_id: i64,
    pub track_id: Option<i64>,
    pub collection_censored_name: Option<String>,
    pub track_censored_name: Option<String>,
    pub artwork_url30: Option<String>,
    pub artwork_url60: Option<String>,
    pub artwork_url600: Option<String>,
    pub collection_price: Option<f64>,
    pub track_price: Option<f64>,
    pub release_date: Option<String>,
    pub collection_explicitness: Option<String>,
    pub track_explicitness: Option<String>,
    pub track_count: Option<i32>,
    pub country: Option<String>,
    pub currency: Option<String>,
    pub primary_genre_name: Option<String>,
    pub content_advisory_rating: Option<String>,
    pub feed_url: Option<String>,
    pub collection_view_url: Option<String>,
    pub collection_hd_price: Option<f64>,
    pub artist_name: Option<String>,
    pub track_name: Option<String>,
    pub collection_name: Option<String>,
    pub artwork_url_100: Option<String>,
    pub preview_url: Option<String>,
    pub track_view_url: String,
    pub track_time_millis: Option<i64>,
    pub genre_ids: Vec<String>,
    pub genres: Vec<String>,
}
