use podfetch_domain::tag::Tag;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::{IntoParams, ToSchema};

use crate::url_rewriting::UrlRewriter;

#[derive(Debug, Serialize, Deserialize, Clone, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct PodcastSearchModelUtoipa {
    pub order: Option<String>,
    pub title: Option<String>,
    pub order_option: Option<String>,
    pub favored_only: bool,
    pub tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PodcastUpdateNameRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DeletePodcast {
    pub delete_files: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastAddModel {
    pub track_id: i32,
    pub user_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PodcastRSSAddModel {
    #[serde(rename = "rssFeedUrl")]
    pub rss_feed_url: String,
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct OpmlModel {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PodcastFavorUpdateModel {
    pub id: i32,
    pub favored: bool,
}

#[derive(Debug, Clone)]
pub struct PodcastInsertModel {
    pub title: String,
    pub id: i32,
    pub feed_url: String,
    pub image_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PodcastDto {
    pub id: i32,
    pub name: String,
    pub directory_id: String,
    pub directory_name: String,
    pub podfetch_feed: String,
    pub rssfeed: String,
    pub image_url: String,
    pub summary: Option<String>,
    pub language: Option<String>,
    pub explicit: Option<String>,
    pub keywords: Option<String>,
    pub last_build_date: Option<String>,
    pub author: Option<String>,
    pub active: bool,
    pub original_image_url: String,
    pub favorites: bool,
    pub tags: Vec<Tag>,
}

impl PodcastDto {
    /// Rewrites internal URLs (image_url, podfetch_feed) to use the resolved server URL.
    ///
    /// This is useful when the server is behind a reverse proxy and clients need URLs
    /// that point to the external-facing URL rather than the internal one.
    pub fn rewrite_urls(&mut self, rewriter: &UrlRewriter) {
        rewriter.rewrite_in_place(&mut self.image_url);
        rewriter.rewrite_in_place(&mut self.podfetch_feed);
    }

    /// Returns a new PodcastDto with URLs rewritten using the given rewriter.
    pub fn with_rewritten_urls(mut self, rewriter: &UrlRewriter) -> Self {
        self.rewrite_urls(rewriter);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProxyPodcastParams {
    pub episode_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    ITunes,
    Podindex,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ItunesWrapper {
    result_count: i32,
    results: Vec<ItunesModel>,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodindexResponse {
    pub status: String,
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

impl TryFrom<i32> for SearchType {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == SearchType::Podindex as i32 => Ok(SearchType::Podindex),
            x if x == SearchType::ITunes as i32 => Ok(SearchType::ITunes),
            _ => Err(()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PodcastControllerError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("{0}")]
    Service(E),
}

pub fn require_privileged<Err: Display>(
    is_privileged: bool,
) -> Result<(), PodcastControllerError<Err>> {
    if is_privileged {
        Ok(())
    } else {
        Err(PodcastControllerError::Forbidden)
    }
}

pub fn require_admin<Err: Display>(is_admin: bool) -> Result<(), PodcastControllerError<Err>> {
    if is_admin {
        Ok(())
    } else {
        Err(PodcastControllerError::Forbidden)
    }
}

pub fn parse_podcast_id<Err: Display>(id: &str) -> Result<i32, PodcastControllerError<Err>> {
    id.parse::<i32>().map_err(|_| {
        PodcastControllerError::BadRequest("podcast id must be an integer".to_string())
    })
}

pub fn parse_search_type<Err: Display>(
    type_of: i32,
) -> Result<SearchType, PodcastControllerError<Err>> {
    SearchType::try_from(type_of)
        .map_err(|_| PodcastControllerError::BadRequest("Invalid search type".to_string()))
}

pub fn ensure_podindex_configured<Err: Display>(
    configured: bool,
) -> Result<(), PodcastControllerError<Err>> {
    if configured {
        Ok(())
    } else {
        Err(PodcastControllerError::BadRequest(
            "Podindex is not configured".to_string(),
        ))
    }
}
