use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct ItunesWrapper {
    result_count: i32,
    results: Vec<ItunesModel>,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodindexResponse {
    pub status: bool,
    pub feeds: Vec<Feed>
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub enum PodcastSearchReturn {
    Itunes(ItunesWrapper),
    Podindex(PodindexResponse),
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    id: i32,
    podcast_guid: String,
    title: String,
    url: String,
    original_url: String,
    link: String,
    description: String,
    author: String,
    owner_name: String,
    image: String,
    artwork: String,
    last_update_time: i32,
    last_crawl_time: i32,
    last_parse_time: i32,
    last_good_http_status_time: i32,
    explicit: bool
}


#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
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
