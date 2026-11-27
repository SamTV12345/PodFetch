//! The podcast feeds: reading them and refreshing the catalog from them.

pub mod read;
pub mod sync;

pub use read::{FeedTranscript, channel_image_url};
pub use sync::{FeedFetcher, FeedSync, FetchedFeed, HttpFeedFetcher, SyncReport};
