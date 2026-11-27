//! Subscribing the server to podcasts: by feed URL, from a podcast
//! directory (iTunes, Podindex) or from an OPML file.
//!
//! The catalog decides what a subscription is and stores it. The podcast's
//! directory and cover belong to the media library; the catalog reaches
//! them through [`PodcastStorage`]. The first refresh is the caller's: it
//! runs the [`FeedSync`](crate::feed::FeedSync) like any other refresh.

use crate::domain::podcast::{NewPodcast, Podcast, PodcastRepository};
use crate::feed::{FeedFetcher, channel_image_url};
use crate::persistence::podcast::DieselPodcastRepository;
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::db::Database;
use common_infrastructure::error::{
    CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error,
};
use common_infrastructure::http::{COMMON_USER_AGENT, get_sync_client};
use common_infrastructure::runtime::is_default_image_url;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use rss::Channel;
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::sync::Arc;
use uuid::Uuid;

/// What to subscribe to. Everything but the feed URL is taken from the
/// feed when it isn't given.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubscribeRequest {
    pub feed_url: String,
    pub title: Option<String>,
    pub image_url: Option<String>,
    /// The podcast's id in a directory (iTunes collection id, Podindex
    /// feed id). Feeds from elsewhere get a random one.
    pub directory_id: Option<i32>,
    pub added_by: Option<Uuid>,
}

/// Port to the media library: where a podcast's files live.
pub trait PodcastStorage: Send + Sync {
    /// Makes the directory for a podcast about to be added and returns its
    /// name. `directory_id_known`: a podcast with the same directory id is
    /// in the catalog already, so a directory of the same name is its own.
    fn create_directory(
        &self,
        channel: &Channel,
        directory_id_known: bool,
    ) -> Result<String, CustomError>;

    /// Stores the cover of a podcast just added, from its `image_url`.
    fn store_cover(&self, podcast: &Podcast) -> Result<(), CustomError>;
}

/// Why a user may not add podcasts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddRefused {
    /// Only admins and uploaders may add podcasts (USER_PODCAST_LIMIT = 0).
    NotAllowed,
    QuotaExceeded {
        limit: u32,
        current: i64,
        adding: u32,
    },
}

impl From<AddRefused> for CustomError {
    fn from(refused: AddRefused) -> Self {
        match refused {
            AddRefused::NotAllowed => CustomErrorInner::Forbidden(ErrorSeverity::Warning).into(),
            AddRefused::QuotaExceeded {
                limit,
                current,
                adding,
            } => CustomErrorInner::BadRequest(
                format!(
                    "Adding {adding} podcast(s) would exceed your limit of {limit}. You currently have {current}."
                ),
                ErrorSeverity::Warning,
            )
            .into(),
        }
    }
}

/// Whether a user may add `adding` more podcasts: admins and uploaders
/// always, other users only within the limit (0 = not at all).
pub fn check_add_permission(
    privileged: bool,
    limit: u32,
    current: i64,
    adding: u32,
) -> Result<(), AddRefused> {
    if privileged {
        return Ok(());
    }
    if limit == 0 {
        return Err(AddRefused::NotAllowed);
    }
    if current + i64::from(adding) > i64::from(limit) {
        return Err(AddRefused::QuotaExceeded {
            limit,
            current,
            adding,
        });
    }
    Ok(())
}

/// The catalog's subscriptions.
pub struct Subscriptions {
    podcasts: DieselPodcastRepository,
    feeds: Arc<dyn FeedFetcher>,
    storage: Arc<dyn PodcastStorage>,
    user_podcast_limit: u32,
}

impl Subscriptions {
    pub fn new(
        database: Database,
        feeds: Arc<dyn FeedFetcher>,
        storage: Arc<dyn PodcastStorage>,
        user_podcast_limit: u32,
    ) -> Self {
        Self {
            podcasts: DieselPodcastRepository::new(database),
            feeds,
            storage,
            user_podcast_limit,
        }
    }

    /// Whether `user_id` may add `adding` more podcasts. Every way of
    /// subscribing - native API and compatibility layers alike - checks it.
    pub fn ensure_can_add(
        &self,
        user_id: Uuid,
        privileged: bool,
        adding: u32,
    ) -> Result<(), CustomError> {
        let current = self.podcasts.count_by_added_by(user_id)?;
        check_add_permission(privileged, self.user_podcast_limit, current, adding)?;
        Ok(())
    }

    /// Adds the podcast behind a feed. The feed must be valid; a feed the
    /// catalog has already is a conflict. Its episodes come with the first
    /// refresh.
    pub fn subscribe(&self, request: SubscribeRequest) -> Result<Podcast, CustomError> {
        if self.podcasts.find_by_rss_feed(&request.feed_url)?.is_some() {
            return Err(CustomErrorInner::Conflict(
                format!("Podcast with feed url {} already exists", request.feed_url),
                ErrorSeverity::Warning,
            )
            .into());
        }
        let fetched = self.feeds.fetch(&request.feed_url)?;
        let channel = Channel::read_from(fetched.content.as_bytes()).map_err(|e| {
            CustomError::from(CustomErrorInner::BadRequest(
                format!("Invalid podcast feed: {e}"),
                ErrorSeverity::Info,
            ))
        })?;

        let directory_id = request
            .directory_id
            .unwrap_or_else(|| rand::random_range(100..10_000_000))
            .to_string();
        let title = request.title.unwrap_or_else(|| channel.title().to_string());
        // An empty image URL makes the default image apply.
        let image_url = request
            .image_url
            .or_else(|| channel_image_url(&channel))
            .unwrap_or_default();

        let known = self.podcasts.find_by_directory_id(&directory_id)?.is_some();
        let directory_name = self.storage.create_directory(&channel, known)?;
        let podcast = self.podcasts.create(NewPodcast {
            name: title,
            directory_id,
            rssfeed: request.feed_url,
            image_url: image_url.clone(),
            directory_name,
            added_by: request.added_by,
        })?;
        // Remember which image the cover comes from, so the first refresh
        // doesn't treat the podcast as having none.
        if !is_default_image_url(&image_url) {
            self.podcasts
                .update_original_image_url(podcast.id, &image_url)?;
        }
        self.storage.store_cover(&podcast)?;
        self.podcasts
            .find_by_id(podcast.id)?
            .ok_or_else(|| CustomErrorInner::NotFound(ErrorSeverity::Error).into())
    }
}

/// The feed URLs of all leaf outlines of an OPML file, depth-first.
pub fn opml_feed_urls(content: &str) -> Result<Vec<String>, CustomError> {
    fn leaves(outlines: &[opml::Outline]) -> Vec<String> {
        outlines
            .iter()
            .flat_map(|outline| {
                if outline.outlines.is_empty() {
                    outline.xml_url.clone().into_iter().collect()
                } else {
                    leaves(&outline.outlines)
                }
            })
            .collect()
    }
    let document = opml::OPML::from_str(content).map_err(|e| {
        CustomError::from(CustomErrorInner::BadRequest(
            format!("Invalid OPML file: {e}"),
            ErrorSeverity::Info,
        ))
    })?;
    Ok(leaves(&document.body.outlines))
}

/// Podcast directories a podcast can be subscribed from.
pub struct Directories {
    environment: Arc<EnvironmentService>,
}

const ITUNES_LOOKUP_URL: &str = "https://itunes.apple.com/lookup";
const PODINDEX_BY_FEED_ID_URL: &str = "https://api.podcastindex.org/api/1.0/podcasts/byfeedid";

impl Directories {
    pub fn new(environment: Arc<EnvironmentService>) -> Self {
        Self { environment }
    }

    /// Whether Podindex can be asked: its API key and secret are set.
    pub fn podindex_configured(&self) -> bool {
        !self.environment.podindex_api_key.is_empty()
            && !self.environment.podindex_api_secret.is_empty()
    }

    /// The subscription for an iTunes podcast.
    pub fn itunes(&self, collection_id: i32) -> Result<SubscribeRequest, CustomError> {
        let response = get_sync_client(&self.environment)
            .build()
            .map_err(map_reqwest_error)?
            .get(ITUNES_LOOKUP_URL)
            .query(&[
                ("id", collection_id.to_string()),
                ("entity", "podcast".to_string()),
            ])
            .send()
            .map_err(map_reqwest_error)?
            .json::<Value>()
            .map_err(map_reqwest_error)?;
        itunes_entry(&response["results"][0]).ok_or_else(|| {
            CustomErrorInner::BadRequest(
                format!("iTunes has no podcast with id {collection_id}"),
                ErrorSeverity::Info,
            )
            .into()
        })
    }

    /// The subscription for a Podindex feed.
    pub fn podindex(&self, feed_id: i32) -> Result<SubscribeRequest, CustomError> {
        let response = get_sync_client(&self.environment)
            .build()
            .map_err(map_reqwest_error)?
            .get(PODINDEX_BY_FEED_ID_URL)
            .query(&[("id", feed_id.to_string())])
            .headers(podindex_headers(&self.environment))
            .send()
            .map_err(map_reqwest_error)?
            .json::<Value>()
            .map_err(map_reqwest_error)?;
        podindex_entry(feed_id, &response["feed"]).ok_or_else(|| {
            CustomErrorInner::BadRequest(
                format!("Podindex has no feed with id {feed_id}"),
                ErrorSeverity::Info,
            )
            .into()
        })
    }
}

fn text(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Reads an entry of an iTunes lookup.
fn itunes_entry(result: &Value) -> Option<SubscribeRequest> {
    Some(SubscribeRequest {
        feed_url: text(&result["feedUrl"])?,
        title: text(&result["collectionName"]),
        image_url: text(&result["artworkUrl600"]),
        directory_id: Some(
            result["collectionId"]
                .as_i64()
                .and_then(|id| i32::try_from(id).ok())?,
        ),
        added_by: None,
    })
}

/// Reads the feed of a Podindex lookup.
fn podindex_entry(feed_id: i32, feed: &Value) -> Option<SubscribeRequest> {
    Some(SubscribeRequest {
        feed_url: text(&feed["url"])?,
        title: text(&feed["title"]),
        image_url: text(&feed["image"]),
        directory_id: Some(feed_id),
        added_by: None,
    })
}

/// The authentication headers Podindex asks for with every request.
pub fn podindex_headers(environment: &EnvironmentService) -> HeaderMap {
    let seconds = chrono::Utc::now().timestamp();
    let api_key = &environment.podindex_api_key;
    let digest = Sha1::digest(format!(
        "{api_key}{}{seconds}",
        environment.podindex_api_secret
    ));
    let hashed: String = digest.iter().map(|b| format!("{b:02x}")).collect();

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(COMMON_USER_AGENT));
    headers.insert("X-Auth-Date", HeaderValue::from(seconds));
    // A hex digest is always a valid header value; only a misconfigured
    // API key can be rejected.
    if let Ok(hashed) = HeaderValue::from_str(&hashed) {
        headers.insert(AUTHORIZATION, hashed);
    }
    match HeaderValue::from_str(api_key) {
        Ok(key) => {
            headers.insert("X-Auth-Key", key);
        }
        Err(_) => tracing::error!("PODINDEX_API_KEY contains characters invalid in headers"),
    }
    headers
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn privileged_users_add_without_limit() {
        assert_eq!(check_add_permission(true, 0, 100, 5), Ok(()));
    }

    #[test]
    fn a_limit_of_zero_forbids_adding() {
        assert_eq!(
            check_add_permission(false, 0, 0, 1),
            Err(AddRefused::NotAllowed)
        );
    }

    #[test]
    fn users_add_up_to_their_limit() {
        assert_eq!(check_add_permission(false, 3, 2, 1), Ok(()));
        assert_eq!(
            check_add_permission(false, 3, 2, 2),
            Err(AddRefused::QuotaExceeded {
                limit: 3,
                current: 2,
                adding: 2
            })
        );
    }

    #[test]
    fn opml_yields_the_feeds_of_nested_outlines() {
        let urls = opml_feed_urls(
            r#"<opml version="2.0"><head><title>t</title></head><body>
                <outline text="a" xmlUrl="https://a.example/feed"/>
                <outline text="group">
                    <outline text="b" xmlUrl="https://b.example/feed"/>
                    <outline text="no feed"/>
                </outline>
            </body></opml>"#,
        )
        .unwrap();
        assert_eq!(urls, ["https://a.example/feed", "https://b.example/feed"]);
    }

    #[test]
    fn invalid_opml_is_a_bad_request() {
        assert!(opml_feed_urls("not opml").is_err());
    }

    #[test]
    fn directory_entries_are_read() {
        let itunes = itunes_entry(&json!({
            "collectionId": 42, "feedUrl": "https://f.example/rss",
            "collectionName": "Show", "artworkUrl600": ""
        }))
        .unwrap();
        assert_eq!(itunes.directory_id, Some(42));
        assert_eq!(itunes.title.as_deref(), Some("Show"));
        assert_eq!(itunes.image_url, None);

        let podindex =
            podindex_entry(7, &json!({"url": "https://p.example/rss", "title": "P"})).unwrap();
        assert_eq!(podindex.feed_url, "https://p.example/rss");
        assert_eq!(podindex.directory_id, Some(7));

        assert_eq!(itunes_entry(&json!({"collectionId": 1})), None);
        assert_eq!(podindex_entry(1, &Value::Null), None);
    }
}
