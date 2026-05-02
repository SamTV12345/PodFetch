use crate::services::sponsorblock::client::{SponsorBlockClient, SponsorBlockError};
use chrono::Utc;
use common_infrastructure::error::CustomError;
use podfetch_domain::podcast_episode::PodcastEpisodeRepository;
use podfetch_domain::podcast_episode_chapter::{
    PodcastEpisodeChapterRepository, UpsertPodcastEpisodeChapter,
};
use podfetch_domain::podcast_settings::PodcastSettingsRepository;
use podfetch_domain::settings::SettingRepository;
use podfetch_domain::sponsorblock::{
    SponsorBlockCategory, SponsorBlockSegment, extract_youtube_id,
};
use podfetch_persistence::adapters::PodcastEpisodeChapterRepositoryImpl;
use podfetch_persistence::db::database;
use podfetch_persistence::podcast_episode::DieselPodcastEpisodeRepository;
use podfetch_persistence::podcast_settings::DieselPodcastSettingsRepository;
use podfetch_persistence::settings::DieselSettingsRepository;

/// Orchestrates SponsorBlock lookups: extract video ID, look up effective
/// settings, query the API, persist segments as chapters.
///
/// All errors are logged but never propagated — the caller (feed-refresh path)
/// must not be blocked by SponsorBlock outages.
pub struct SponsorBlockSyncService {
    client: SponsorBlockClient,
}

impl SponsorBlockSyncService {
    pub fn new(client: SponsorBlockClient) -> Self {
        Self { client }
    }

    pub fn default_service() -> Self {
        Self::new(SponsorBlockClient::new())
    }

    /// Try to sync SponsorBlock data for a single episode. No-op if:
    /// - SponsorBlock is disabled (globally or per-podcast),
    /// - the URL/GUID is not a recognised YouTube reference,
    /// - the episode already has a `sponsorblock_fetched_at` timestamp.
    ///
    /// Errors during the HTTP call are logged and swallowed.
    pub fn maybe_sync(
        &self,
        episode_id: i32,
        podcast_id: i32,
        url: &str,
        guid: &str,
    ) -> Result<(), CustomError> {
        let categories = match resolve_categories(podcast_id)? {
            Some(c) => c,
            None => return Ok(()),
        };

        let video_id = match extract_youtube_id(url, Some(guid)) {
            Some(id) => id,
            None => return Ok(()),
        };

        let episode_repo = DieselPodcastEpisodeRepository::new(database());
        if let Some(episode) = episode_repo.find_by_id(episode_id)?
            && episode.sponsorblock_fetched_at.is_some()
        {
            return Ok(());
        }

        let segments = match self.client.fetch_segments(&video_id, &categories) {
            Ok(s) => s,
            Err(SponsorBlockError::NotFound) => Vec::new(),
            Err(e) => {
                tracing::warn!(
                    "SponsorBlock fetch failed for episode {episode_id} (video {video_id}): {e}"
                );
                return Ok(());
            }
        };

        persist_segments(episode_id, &segments)?;
        episode_repo
            .set_sponsorblock_fetched_at(episode_id, Some(Utc::now().naive_utc()))?;
        Ok(())
    }

    /// Reset the fetched-at marker for every episode of a podcast and refetch.
    /// Returns the number of episodes that were reset.
    pub fn force_resync_podcast(&self, podcast_id: i32) -> Result<usize, CustomError> {
        let episode_repo = DieselPodcastEpisodeRepository::new(database());
        let cleared = episode_repo.clear_sponsorblock_fetched_at_for_podcast(podcast_id)?;
        let episodes = episode_repo.find_by_podcast_id(podcast_id)?;
        for episode in episodes {
            self.maybe_sync(episode.id, podcast_id, &episode.url, &episode.guid)?;
        }
        Ok(cleared)
    }
}

impl Default for SponsorBlockSyncService {
    fn default() -> Self {
        Self::default_service()
    }
}

/// Returns the effective categories for the podcast, or `None` if SponsorBlock
/// is disabled in the effective configuration.
fn resolve_categories(podcast_id: i32) -> Result<Option<Vec<SponsorBlockCategory>>, CustomError> {
    let podcast_settings_repo = DieselPodcastSettingsRepository::new(database());
    if let Some(per_podcast) = podcast_settings_repo.get_settings(podcast_id)? {
        if !per_podcast.sponsorblock_enabled {
            return Ok(None);
        }
        return Ok(Some(per_podcast.sponsorblock_categories));
    }

    let global_repo = DieselSettingsRepository::new(database());
    let global = match global_repo.get_settings()? {
        Some(s) => s,
        None => return Ok(None),
    };
    if !global.sponsorblock_enabled {
        return Ok(None);
    }
    Ok(Some(global.sponsorblock_categories))
}

fn persist_segments(
    episode_id: i32,
    segments: &[SponsorBlockSegment],
) -> Result<(), CustomError> {
    if segments.is_empty() {
        return Ok(());
    }
    let chapter_repo = PodcastEpisodeChapterRepositoryImpl::new(database());
    for segment in segments {
        let upsert = UpsertPodcastEpisodeChapter {
            episode_id,
            title: segment.uuid.clone(),
            start_time: segment.start_seconds.round() as i32,
            end_time: segment.end_seconds.round() as i32,
            href: None,
            image: None,
            chapter_type: segment.category.as_str().to_string(),
        };
        chapter_repo.upsert(upsert)?;
    }
    Ok(())
}
