//! Refreshing a podcast from its feed: new episodes are added, known ones
//! updated, the podcast's metadata and image taken over.
//!
//! The catalog doesn't call other contexts. What they need from the feed —
//! the transcript tags of the items, a cover the podcast didn't have — is
//! in the [`SyncReport`]; the caller hands it on.

use super::read::{self, FeedTranscript};
use crate::domain::podcast::{Podcast, PodcastRepository};
use crate::domain::podcast_episode::{NewPodcastEpisode, PodcastEpisode, PodcastEpisodeRepository};
use crate::persistence::podcast::DieselPodcastRepository;
use crate::persistence::podcast_episode::DieselPodcastEpisodeRepository;
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::db::Database;
use common_infrastructure::error::{
    CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error,
};
use common_infrastructure::http::{COMMON_USER_AGENT, get_sync_client};
use common_infrastructure::runtime::is_default_image_url;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use rss::{Channel, Guid, Item};
use std::sync::Arc;
use uuid::Uuid;

/// A feed as fetched: its final URL (after redirects) and its content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedFeed {
    pub url: String,
    pub content: String,
}

/// Port to the network: fetches a feed.
pub trait FeedFetcher: Send + Sync {
    fn fetch(&self, feed_url: &str) -> Result<FetchedFeed, CustomError>;
}

/// Fetches feeds over HTTP, following redirects.
pub struct HttpFeedFetcher {
    environment: Arc<EnvironmentService>,
}

impl HttpFeedFetcher {
    pub fn new(environment: Arc<EnvironmentService>) -> Self {
        Self { environment }
    }
}

impl FeedFetcher for HttpFeedFetcher {
    fn fetch(&self, feed_url: &str) -> Result<FetchedFeed, CustomError> {
        // Follows every redirect, as the feed refresh always did.
        let client = get_sync_client(&self.environment)
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                attempt.follow()
            }))
            .build()
            .map_err(map_reqwest_error)?;
        let mut headers = HeaderMap::new();
        headers.append(
            ACCEPT,
            HeaderValue::from_static("application/rss+xml,application/xml"),
        );
        headers.append(USER_AGENT, HeaderValue::from_static(COMMON_USER_AGENT));
        let response = client
            .get(feed_url)
            .headers(headers)
            .send()
            .map_err(map_reqwest_error)?;
        let url = response.url().to_string();
        let content = response.text().map_err(map_reqwest_error)?;
        Ok(FetchedFeed { url, content })
    }
}

/// What a refresh found that other contexts may need.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncReport {
    /// The episodes new in the catalog.
    pub inserted: Vec<PodcastEpisode>,
    /// The `<podcast:transcript>` tags of every item with tags, new or
    /// known, by catalog episode id, in feed order.
    pub transcripts: Vec<(Uuid, Vec<FeedTranscript>)>,
    /// The feed's image, when the podcast had only the default cover so far.
    pub new_cover: Option<String>,
}

pub struct FeedSync {
    podcasts: DieselPodcastRepository,
    episodes: DieselPodcastEpisodeRepository,
    fetcher: Arc<dyn FeedFetcher>,
}

fn invalid_feed(name: &str, error: impl std::fmt::Debug) -> CustomError {
    CustomErrorInner::BadRequest(
        format!("Error parsing podcast {name} with cause {error:?}"),
        ErrorSeverity::Error,
    )
    .into()
}

impl FeedSync {
    pub fn new(database: Database, fetcher: Arc<dyn FeedFetcher>) -> Self {
        Self {
            podcasts: DieselPodcastRepository::new(database.clone()),
            episodes: DieselPodcastEpisodeRepository::new(database),
            fetcher,
        }
    }

    /// Refreshes the podcast from its feed.
    #[tracing::instrument(skip_all, fields(podcast_id = %podcast.id, podcast_name = %podcast.name))]
    pub fn sync(&self, podcast: &Podcast) -> Result<SyncReport, CustomError> {
        let fetched = self.fetcher.fetch(&podcast.rssfeed)?;
        let channel = Channel::read_from(fetched.content.as_bytes()).map_err(|e| {
            tracing::info!(
                "Error parsing podcast {} {:?} with cause {:?}",
                podcast.name,
                fetched.content,
                e
            );
            invalid_feed(&podcast.name, e)
        })?;

        self.follow_new_feed_url(podcast, &channel)?;
        if let Some(update) = read::metadata(&channel, podcast.id) {
            self.podcasts.update_metadata(update)?;
        }
        let mut report = SyncReport {
            new_cover: self.take_over_image(podcast, &channel)?,
            ..SyncReport::default()
        };

        for item in &channel.items {
            let Some(enclosure) = &item.enclosure else {
                tracing::info!(
                    "Skipping podcast episode {:?} as it has no enclosure",
                    item.title
                );
                continue;
            };
            let known = match &item.guid {
                Some(guid) => self.episodes.find_by_guid(&guid.value)?,
                None => self
                    .episodes
                    .find_by_url(&enclosure.url, Some(podcast.id))?,
            };
            let episode_id = match known {
                Some(episode) => {
                    self.update_known(podcast, item, &enclosure.url, &episode)?;
                    episode.id
                }
                None => {
                    let inserted = self.insert(podcast, item, &enclosure.url)?;
                    let id = inserted.id;
                    report.inserted.push(inserted);
                    id
                }
            };
            let transcripts = read::transcripts(item);
            if !transcripts.is_empty() {
                report.transcripts.push((episode_id, transcripts));
            }
        }
        Ok(report)
    }

    /// Takes over a changed title, enclosure, description or image.
    fn update_known(
        &self,
        podcast: &Podcast,
        item: &Item,
        enclosure_url: &str,
        episode: &PodcastEpisode,
    ) -> Result<(), CustomError> {
        let mut updated = episode.clone();
        if let Some(title) = &item.title {
            updated.name.clone_from(title);
        }
        updated.url = enclosure_url.to_owned();
        if let Some(description) = &item.description {
            updated.description.clone_from(description);
        }
        // Backfill missing episode artwork with the parent podcast's image so
        // older rows that stored the default fallback get repaired on the next
        // feed refresh.
        let itunes_image = item.itunes_ext.as_ref().and_then(|ext| ext.image.clone());
        if let Some(itunes_image) = itunes_image {
            updated.image_url = itunes_image;
        } else if is_default_image_url(&updated.image_url)
            && !is_default_image_url(&podcast.original_image_url)
        {
            updated.image_url.clone_from(&podcast.original_image_url);
        }

        if updated.name != episode.name
            || updated.url != episode.url
            || updated.description != episode.description
            || updated.image_url != episode.image_url
        {
            self.episodes.update(&updated)?;
        }
        Ok(())
    }

    fn insert(
        &self,
        podcast: &Podcast,
        item: &Item,
        enclosure_url: &str,
    ) -> Result<PodcastEpisode, CustomError> {
        let mut duration = 0;
        // Fall back to the parent podcast's image (the one that wraps all
        // episodes) when an episode carries no image of its own.
        let mut image_url = podcast.original_image_url.clone();
        if let Some(itunes_ext) = &item.itunes_ext {
            if let Some(itunes_duration) = &itunes_ext.duration {
                duration = read::duration(itunes_duration);
            }
            if let Some(itunes_image) = &itunes_ext.image {
                image_url.clone_from(itunes_image);
            }
        }
        let guid = item
            .guid
            .clone()
            .unwrap_or(Guid {
                value: Uuid::new_v4().to_string(),
                ..Default::default()
            })
            .value;
        let youtube_video_id = common_infrastructure::youtube::extract_youtube_video_id(
            item.link.as_deref(),
            item.guid.as_ref().map(|g| g.value.as_str()),
            item.enclosure.as_ref().map(|e| e.url.as_str()),
        );
        Ok(self.episodes.create(NewPodcastEpisode {
            podcast_id: podcast.id,
            episode_id: Uuid::new_v4().to_string(),
            name: item
                .title
                .clone()
                .unwrap_or_else(|| "No title given".to_string()),
            url: enclosure_url.to_owned(),
            date_of_recording: read::recording_date(item),
            image_url,
            total_time: duration as i32,
            description: item.description.clone().unwrap_or_default(),
            guid,
            youtube_video_id,
        })?)
    }

    /// Stores the feed's image as the podcast's original image. Returns it
    /// when the podcast had only the default cover so far.
    fn take_over_image(
        &self,
        podcast: &Podcast,
        channel: &Channel,
    ) -> Result<Option<String>, CustomError> {
        let image_url = read::channel_image_url(channel);
        self.podcasts
            .update_original_image_url(podcast.id, image_url.as_deref().unwrap_or(""))?;
        // Podcasts that were added without a usable image (e.g. a feed that
        // only had <itunes:image>, which older versions ignored) got the
        // default cover; the real one can be fetched now.
        Ok(image_url.filter(|_| is_default_image_url(&podcast.original_image_url)))
    }

    /// Follows `<itunes:new-feed-url>`: stores the new feed URL and takes
    /// over the enclosure URLs of the episodes from the new feed.
    fn follow_new_feed_url(&self, podcast: &Podcast, channel: &Channel) -> Result<(), CustomError> {
        let Some(new_url) = channel
            .itunes_ext
            .as_ref()
            .and_then(|ext| ext.new_feed_url.as_ref())
        else {
            return Ok(());
        };
        self.podcasts.update_rss_feed(podcast.id, new_url)?;

        // As before: the refetch goes to the feed URL the podcast was loaded
        // with, not to the new one.
        let fetched = self.fetcher.fetch(&podcast.rssfeed)?;
        let channel = Channel::read_from(fetched.content.as_bytes()).map_err(|e| {
            CustomError::from(CustomErrorInner::BadRequest(
                format!("Invalid podcast feed after redirect to {new_url}: {e}"),
                ErrorSeverity::Warning,
            ))
        })?;
        for item in channel.items() {
            let (Some(guid), Some(enclosure)) = (&item.guid, &item.enclosure) else {
                tracing::debug!(
                    "Skipping redirected item {:?} without guid/enclosure",
                    item.title
                );
                continue;
            };
            if let Some(mut episode) = self.episodes.find_by_guid(&guid.value)? {
                episode.url.clone_from(&enclosure.url);
                self.episodes.update(&episode)?;
            }
        }
        Ok(())
    }
}
