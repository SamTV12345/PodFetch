use common_infrastructure::config::FileHandlerType;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use http::HeaderMap;
use podfetch_domain::ordering::{OrderCriteria, OrderOption};
use podfetch_domain::podcast::Podcast;
use podfetch_domain::user::User;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::thread;
use utoipa::{IntoParams, ToSchema};

use crate::filter::Filter;
use crate::tags::Tag;
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

#[derive(Debug, Clone)]
pub struct PodcastSearchPlan {
    pub order: OrderCriteria,
    pub order_option: OrderOption,
    pub title: Option<String>,
    pub tag: Option<String>,
    pub favored_only: bool,
    pub user_id: i32,
    pub filter: Filter,
}

pub fn build_podcast_search_plan(
    query: PodcastSearchModelUtoipa,
    user_id: i32,
    existing_filter: Option<Filter>,
) -> PodcastSearchPlan {
    let order = query.order.map(|o| o.into()).unwrap_or(OrderCriteria::Asc);
    let order_option = query
        .order_option
        .map(OrderOption::from_string)
        .unwrap_or(OrderOption::Title);
    let only_favored = existing_filter
        .map(|filter| filter.only_favored)
        .unwrap_or(true);
    let filter = Filter::new(
        user_id,
        query.title.clone(),
        order.to_bool(),
        Some(order_option.to_string()),
        only_favored,
    );

    PodcastSearchPlan {
        order,
        order_option,
        title: query.title,
        tag: query.tag,
        favored_only: query.favored_only,
        user_id,
        filter,
    }
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

pub fn map_podcast_to_dto(value: Podcast) -> PodcastDto {
    let image_url = format!(
        "{}{}",
        ENVIRONMENT_SERVICE.get_server_url(),
        value.image_url
    );
    let keywords = dedupe_keywords(value.keywords.clone());
    let podfetch_rss_feed = build_podfetch_feed(value.id, None);

    PodcastDto {
        id: value.id,
        name: value.name.clone(),
        directory_id: value.directory_id.clone(),
        rssfeed: value.rssfeed.clone(),
        image_url,
        language: value.language.clone(),
        keywords,
        podfetch_feed: podfetch_rss_feed,
        summary: value.summary.clone(),
        explicit: value.explicit.clone(),
        last_build_date: value.last_build_date.clone(),
        author: value.author.clone(),
        active: value.active,
        original_image_url: value.original_image_url.clone(),
        directory_name: value.directory_name.clone(),
        tags: vec![],
        favorites: false,
    }
}

pub fn map_podcast_with_context_to_dto(
    value: Podcast,
    favorite: Option<bool>,
    tags: Vec<Tag>,
    user: &User,
) -> PodcastDto {
    let image_url = match resolve_file_handler_type(value.download_location.clone()) {
        FileHandlerType::Local => {
            format!(
                "{}{}",
                ENVIRONMENT_SERVICE.get_server_url(),
                value.image_url
            )
        }
        FileHandlerType::S3 => {
            format!(
                "{}/{}",
                ENVIRONMENT_SERVICE.s3_config.endpoint.clone(),
                &value.image_url
            )
        }
    };

    PodcastDto {
        id: value.id,
        name: value.name.clone(),
        directory_id: value.directory_id.clone(),
        rssfeed: value.rssfeed.clone(),
        image_url,
        podfetch_feed: build_podfetch_feed(value.id, user.api_key.as_deref()),
        language: value.language.clone(),
        keywords: dedupe_keywords(value.keywords.clone()),
        summary: value.summary.clone(),
        explicit: value.explicit.clone(),
        last_build_date: value.last_build_date.clone(),
        author: value.author.clone(),
        active: value.active,
        original_image_url: value.original_image_url.clone(),
        directory_name: value.directory_name.clone(),
        tags,
        favorites: favorite.unwrap_or(false),
    }
}

fn resolve_file_handler_type(value: Option<String>) -> FileHandlerType {
    match value {
        Some(val) => FileHandlerType::from(val.as_str()),
        None => ENVIRONMENT_SERVICE.default_file_handler.clone(),
    }
}

fn dedupe_keywords(keywords: Option<String>) -> Option<String> {
    keywords.map(|k| {
        k.split(',')
            .map(|keyword| keyword.trim().to_string())
            .collect::<HashSet<String>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(",")
    })
}

fn build_podfetch_feed(podcast_id: i32, api_key: Option<&str>) -> String {
    let mut podfetch_rss_feed = ENVIRONMENT_SERVICE.build_url_to_rss_feed();
    podfetch_rss_feed
        .join(&format!("/{}", podcast_id))
        .expect("safe string join for podcast rss feed");

    if let Some(api_key) = api_key {
        podfetch_rss_feed
            .query_pairs_mut()
            .append_pair("apiKey", api_key);
    }

    podfetch_rss_feed.to_string()
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
    pub id: Option<i32>,
    pub podcast_guid: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub original_url: Option<String>,
    pub link: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub owner_name: Option<String>,
    pub image: Option<String>,
    pub artwork: Option<String>,
    pub last_update_time: Option<i32>,
    pub last_crawl_time: Option<i32>,
    pub last_parse_time: Option<i32>,
    pub last_good_http_status_time: Option<i32>,
    pub explicit: Option<bool>,
    #[serde(default)]
    pub categories: Option<HashMap<String, String>>,
    pub language: Option<String>,
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
    #[error("quota exceeded: {0}")]
    QuotaExceeded(String),
    #[error("{0}")]
    Service(E),
}

pub fn map_podcast_error(error: PodcastControllerError<CustomError>) -> CustomError {
    match error {
        PodcastControllerError::Forbidden => {
            CustomErrorInner::Forbidden(ErrorSeverity::Warning).into()
        }
        PodcastControllerError::NotFound => CustomErrorInner::NotFound(ErrorSeverity::Debug).into(),
        PodcastControllerError::BadRequest(message) => {
            CustomErrorInner::BadRequest(message, ErrorSeverity::Info).into()
        }
        PodcastControllerError::QuotaExceeded(message) => {
            CustomErrorInner::BadRequest(message, ErrorSeverity::Warning).into()
        }
        PodcastControllerError::Service(error) => error,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProxyPodcastError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Service(E),
}

pub fn ensure_proxy_api_access<Err: Display, F>(
    auth_enabled: bool,
    api_key: Option<String>,
    is_api_key_valid: F,
) -> Result<(), ProxyPodcastError<Err>>
where
    F: FnOnce(&str) -> bool,
{
    if !auth_enabled {
        return Ok(());
    }

    let api_key = api_key.ok_or(ProxyPodcastError::Forbidden)?;
    if is_api_key_valid(&api_key) {
        Ok(())
    } else {
        Err(ProxyPodcastError::Forbidden)
    }
}

pub fn require_proxy_episode<T, Err: Display>(
    episode: Option<T>,
) -> Result<T, ProxyPodcastError<Err>> {
    episode.ok_or(ProxyPodcastError::NotFound)
}

pub fn sanitize_proxy_request_headers(headers: &mut HeaderMap) {
    for header in ["host", "referer", "sec-fetch-site"] {
        headers.remove(header);
    }
}

pub fn map_proxy_podcast_error(error: ProxyPodcastError<CustomError>) -> CustomError {
    match error {
        ProxyPodcastError::Forbidden => CustomErrorInner::Forbidden(ErrorSeverity::Debug).into(),
        ProxyPodcastError::NotFound => CustomErrorInner::NotFound(ErrorSeverity::Debug).into(),
        ProxyPodcastError::Service(error) => error,
    }
}

pub fn spawn_podindex_download<Err, F>(track_id: i32, download: F)
where
    F: FnOnce(i32) -> Result<(), Err> + Send + 'static,
    Err: Display + Send + 'static,
{
    thread::spawn(move || {
        if let Err(error) = download(track_id) {
            log::error!("Error downloading podindex podcast: {error}");
        }
    });
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

pub fn check_podcast_add_permission<Err: Display>(
    is_privileged: bool,
    user_podcast_limit: u32,
    current_count: i64,
    adding_count: u32,
) -> Result<(), PodcastControllerError<Err>> {
    if is_privileged {
        return Ok(());
    }

    if user_podcast_limit == 0 {
        return Err(PodcastControllerError::Forbidden);
    }

    let new_total = current_count + adding_count as i64;
    if new_total > user_podcast_limit as i64 {
        return Err(PodcastControllerError::QuotaExceeded(format!(
            "Adding {} podcast(s) would exceed your limit of {}. You currently have {}.",
            adding_count, user_podcast_limit, current_count
        )));
    }

    Ok(())
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
