//! Media library services for code that has no injected dependencies yet,
//! and the ports the media library needs from this crate.
//!
//! Transitional: downloads and rescans are still associated functions.
//! Once they move into `podfetch_media` (migration step 5.2b), they get the
//! SponsorBlock service injected and the accessors here go away.

use crate::services::settings::service::SettingsService;
use common_infrastructure::error::CustomError;
use common_infrastructure::http::get_http_client;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_media::persistence::DieselSponsorblockRepository;
use podfetch_media::sponsorblock::{
    EpisodeToCheck, SponsorblockClient, SponsorblockService, SponsorblockSwitch,
};
use podfetch_persistence::db::database;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity;
use std::sync::Arc;

/// SponsorBlock is on unless the admin turned it off in the settings.
pub struct SettingsSponsorblockSwitch(pub Arc<SettingsService>);

impl SponsorblockSwitch for SettingsSponsorblockSwitch {
    fn enabled(&self) -> Result<bool, CustomError> {
        Ok(self
            .0
            .get_settings()?
            .is_none_or(|settings| settings.sponsorblock_enabled))
    }
}

pub fn sponsorblock() -> SponsorblockService {
    SponsorblockService::new(
        Arc::new(DieselSponsorblockRepository::new(database())),
        SponsorblockClient::new(get_http_client(&ENVIRONMENT_SERVICE)),
        Arc::new(SettingsSponsorblockSwitch(Arc::new(
            SettingsService::shared(),
        ))),
    )
}

/// Fetches and stores the SponsorBlock segments of a catalog episode from
/// synchronous code (the downloader and rescan), whatever async context the
/// caller runs in. Returns the number of segments stored.
pub fn fetch_sponsor_segments_blocking(
    episode: &PodcastEpisodeEntity,
) -> Result<usize, CustomError> {
    let episode_id = episode.id.clone();
    let youtube_video_id = episode.youtube_video_id.clone();
    let duration_secs = i64::from(episode.total_time);
    crate::background::run_isolated("sponsorblock-fetch", move || async move {
        sponsorblock()
            .fetch_and_store(EpisodeToCheck {
                episode_id: &episode_id,
                youtube_video_id: youtube_video_id.as_deref(),
                duration_secs,
            })
            .await
    })
}
