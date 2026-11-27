//! Reading the YouTube video id of a feed item. The catalog stores it with
//! the episode; the media library's SponsorBlock lookup is keyed by it.
//! Fetching and storing SponsorBlock data lives in `podfetch_media`.

pub mod video_id;
