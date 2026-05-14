//! `POST /api/podcasts/feed` — fetch + parse an RSS feed without inserting,
//! preview shape for the audiobookshelf "Add Podcast" form.
//!
//! `POST /api/podcasts` — actually subscribe to a podcast. Mobile apps submit
//! this after the user picks an iTunes result or pastes a feed URL.
//!
//! Both reuse PodFetch's existing podcast pipeline
//! (`PodcastService::handle_insert_of_podcast`) under the hood; we only map
//! input/output between audiobookshelf-shape JSON and PodFetch's model.

use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::mapping::podcast::map_podcast;
use crate::podcast::PodcastInsertModel;
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::Json;
use axum::extract::State;
use chrono::DateTime;
use common_infrastructure::error::ErrorSeverity::Error as ErrSeverityError;
use common_infrastructure::error::{CustomError, CustomErrorInner, map_reqwest_error};
use common_infrastructure::http::{COMMON_USER_AGENT, get_http_client};
use common_infrastructure::request::add_basic_auth_headers_conditionally;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use rand::RngExt;
use reqwest::header::HeaderMap as ReqwestHeaderMap;
use rss::Channel;
use serde::Deserialize;
use serde_json::{Value, json};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetPodcastFeedRequest {
    pub rss_feed: String,
}

#[utoipa::path(
    post,
    path = "/api/podcasts/feed",
    request_body = GetPodcastFeedRequest,
    responses((status = 200, description = "Parsed podcast feed preview")),
    tag = "audiobookshelf"
)]
pub async fn get_podcast_feed(
    State(_state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Json(body): Json<GetPodcastFeedRequest>,
) -> Result<Json<Value>, CustomError> {
    let feed_url = body.rss_feed.trim().to_string();
    if feed_url.is_empty()
        || (!feed_url.starts_with("http://") && !feed_url.starts_with("https://"))
    {
        return Err(CustomErrorInner::BadRequest(
            "rssFeed must be a valid http(s) URL".to_string(),
            ErrSeverityError,
        )
        .into());
    }

    let channel = fetch_and_parse_feed(&feed_url).await?;
    Ok(Json(json!({ "podcast": shape_feed(&channel, &feed_url) })))
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePodcastRequest {
    pub media: CreatePodcastMedia,
    #[allow(dead_code)]
    pub library_id: Option<String>,
    #[allow(dead_code)]
    pub folder_id: Option<String>,
    #[allow(dead_code)]
    pub path: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePodcastMedia {
    pub metadata: CreatePodcastMetadata,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePodcastMetadata {
    pub feed_url: String,
    #[allow(dead_code)]
    pub title: Option<String>,
    #[allow(dead_code)]
    pub image_url: Option<String>,
    #[allow(dead_code)]
    pub itunes_id: Option<i64>,
}

#[utoipa::path(
    post,
    path = "/api/podcasts",
    request_body = CreatePodcastRequest,
    responses((status = 200, description = "Subscribed podcast (libraryItem shape)")),
    tag = "audiobookshelf"
)]
pub async fn create_podcast(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(body): Json<CreatePodcastRequest>,
) -> Result<Json<Value>, CustomError> {
    let feed_url = body.media.metadata.feed_url.trim().to_string();
    if feed_url.is_empty() {
        return Err(CustomErrorInner::BadRequest(
            "media.metadata.feedUrl must be set".to_string(),
            ErrSeverityError,
        )
        .into());
    }

    let channel = fetch_and_parse_feed(&feed_url).await?;
    let title = body
        .media
        .metadata
        .title
        .clone()
        .unwrap_or_else(|| channel.title().to_string());
    let image_url = body
        .media
        .metadata
        .image_url
        .clone()
        .or_else(|| channel.image().map(|i| i.url.clone()))
        .unwrap_or_else(|| {
            ENVIRONMENT_SERVICE.server_url.clone()
                + common_infrastructure::runtime::DEFAULT_IMAGE_URL
        });
    let id = body
        .media
        .metadata
        .itunes_id
        .map(|v| v as i32)
        .unwrap_or_else(|| rand::rng().random_range(100..10_000_000));

    let inserted = PodcastService::handle_insert_of_podcast(
        PodcastInsertModel {
            feed_url: feed_url.clone(),
            title,
            id,
            image_url,
        },
        Some(channel),
        Some(user.id),
    )
    .await?;

    let library = state
        .audiobookshelf_library_service
        .find_default_podcasts_library()?
        .ok_or_else(|| {
            CustomError::from(CustomErrorInner::NotFound(
                common_infrastructure::error::ErrorSeverity::Debug,
            ))
        })?;
    let podcast: podfetch_domain::podcast::Podcast = inserted.into();
    let episodes: Vec<podfetch_domain::podcast_episode::PodcastEpisode> =
        PodcastEpisodeService::get_episodes_by_podcast_id(podcast.id)?
            .into_iter()
            .map(Into::into)
            .collect();
    Ok(Json(map_podcast(&podcast, &episodes, &library.id)))
}

async fn fetch_and_parse_feed(feed_url: &str) -> Result<Channel, CustomError> {
    let mut headers = ReqwestHeaderMap::new();
    headers.insert("User-Agent", COMMON_USER_AGENT.parse().unwrap());
    add_basic_auth_headers_conditionally(feed_url.to_string(), &mut headers);
    let response = get_http_client(&ENVIRONMENT_SERVICE)
        .get(feed_url)
        .headers(headers)
        .send()
        .await
        .map_err(map_reqwest_error)?;
    if !response.status().is_success() {
        return Err(CustomErrorInner::BadRequest(
            format!("Feed responded with {}", response.status()),
            ErrSeverityError,
        )
        .into());
    }
    let body = response.text().await.map_err(map_reqwest_error)?;
    Channel::read_from(body.as_bytes()).map_err(|e| {
        CustomError::from(CustomErrorInner::BadRequest(
            format!("Invalid RSS feed: {e}"),
            ErrSeverityError,
        ))
    })
}

/// Builds the upstream-shaped `podcast` body for /api/podcasts/feed. Field
/// names mirror `server/utils/podcastUtils.js::cleanPodcastJson` so the
/// mobile-app's "New Podcast" form can prefill itself from the response.
fn shape_feed(channel: &Channel, feed_url: &str) -> Value {
    let itunes = channel.itunes_ext();
    let image = channel.image().map(|i| i.url.clone()).unwrap_or_default();
    let categories: Vec<Value> = itunes
        .map(|i| i.categories.clone())
        .map(|cats| {
            cats.into_iter()
                .map(|c| Value::from(c.text))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let metadata = json!({
        "image": image,
        "categories": categories,
        "feedUrl": feed_url,
        "title": channel.title(),
        "language": channel.language().unwrap_or_default(),
        "description": channel.description(),
        "descriptionPlain": channel.description(),
        "explicit": itunes.and_then(|i| i.explicit.clone()).unwrap_or_default(),
        "author": itunes.and_then(|i| i.author.clone()).unwrap_or_default(),
        "pubDate": channel.pub_date().unwrap_or_default(),
        "link": channel.link(),
        "type": itunes
            .and_then(|i| i.r#type.clone())
            .unwrap_or_else(|| "episodic".to_string()),
    });

    let episodes: Vec<Value> = channel
        .items()
        .iter()
        .filter_map(|item| {
            let enclosure = item.enclosure().map(|e| {
                json!({
                    "url": e.url(),
                    "type": e.mime_type(),
                    "length": e.length(),
                })
            })?;
            let pub_date_str = item.pub_date().unwrap_or_default();
            let published_at_ms = DateTime::parse_from_rfc2822(pub_date_str)
                .ok()
                .map(|dt| dt.timestamp_millis());
            let it_ext = item.itunes_ext();
            let duration_str = it_ext.and_then(|i| i.duration.clone()).unwrap_or_default();
            let duration_seconds = parse_duration_seconds(&duration_str);
            Some(json!({
                "title": item.title().unwrap_or_default(),
                "subtitle": it_ext.and_then(|i| i.subtitle.clone()).unwrap_or_default(),
                "description": item.description().unwrap_or_default(),
                "descriptionPlain": item.description().unwrap_or_default(),
                "pubDate": pub_date_str,
                "episodeType": it_ext.and_then(|i| i.episode_type.clone()).unwrap_or_default(),
                "season": it_ext.and_then(|i| i.season.clone()).unwrap_or_default(),
                "episode": it_ext.and_then(|i| i.episode.clone()).unwrap_or_default(),
                "author": it_ext.and_then(|i| i.author.clone()).unwrap_or_default(),
                "duration": duration_str,
                "durationSeconds": duration_seconds,
                "explicit": it_ext.and_then(|i| i.explicit.clone()).unwrap_or_default(),
                "publishedAt": published_at_ms,
                "enclosure": enclosure,
                "guid": item.guid().map(|g| g.value().to_string()),
                "chaptersUrl": Value::Null,
                "chaptersType": Value::Null,
                "chapters": Value::Array(Vec::new()),
            }))
        })
        .collect();

    json!({
        "metadata": metadata,
        "episodes": episodes,
        "numEpisodes": channel.items().len(),
    })
}

/// Parses iTunes-style duration strings ("HH:MM:SS", "MM:SS", or plain
/// seconds) the same way `podcastUtils.timestampToSeconds()` does. Returns
/// `null` (None) on unparseable input — mobile app accepts null here.
fn parse_duration_seconds(s: &str) -> Option<i64> {
    if s.is_empty() {
        return None;
    }
    if let Ok(n) = s.parse::<i64>() {
        return Some(n);
    }
    let parts: Vec<&str> = s.split(':').collect();
    let nums: Option<Vec<i64>> = parts.iter().map(|p| p.parse::<i64>().ok()).collect();
    let nums = nums?;
    match nums.len() {
        2 => Some(nums[0] * 60 + nums[1]),
        3 => Some(nums[0] * 3600 + nums[1] * 60 + nums[2]),
        _ => None,
    }
}

pub fn get_podcasts_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_podcast_feed))
        .routes(routes!(create_podcast))
}

#[cfg(test)]
pub fn parse_duration_seconds_for_test(s: &str) -> Option<i64> {
    parse_duration_seconds(s)
}
