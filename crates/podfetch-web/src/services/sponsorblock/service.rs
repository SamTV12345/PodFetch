//! Orchestrates fetching SponsorBlock data for a downloaded episode and storing
//! it. All failures are the caller's to swallow — see download/service.rs.

use crate::services::sponsorblock::client::{self, FetchedSegment};
use common_infrastructure::error::CustomError;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_persistence::sponsorblock::{SponsorblockRepository, SponsorSegmentEntity};

/// Tolerance for declaring a duration mismatch: the larger of 2 seconds or 1%.
fn durations_mismatch(episode_secs: i64, sb_secs: f64) -> bool {
    // If either side is unknown (<= 0) we cannot verify alignment, so we do NOT
    // flag — better to skip than to disable the feature for feeds without a
    // duration. Only a confident, sizeable divergence flags as a mismatch.
    if episode_secs <= 0 || sb_secs <= 0.0 {
        return false;
    }
    let diff = (episode_secs as f64 - sb_secs).abs();
    let tolerance = (sb_secs * 0.01).max(2.0);
    diff > tolerance
}

/// Fetch + store SponsorBlock segments for one episode. Returns the number of
/// segments stored. Caller is responsible for non-fatal error handling.
pub async fn fetch_and_store(episode: &PodcastEpisode) -> Result<usize, CustomError> {
    // Guard 1: global toggle.
    let settings = crate::services::settings::service::SettingsService::shared().get_settings()?;
    let enabled = settings.map(|s| s.sponsorblock_enabled).unwrap_or(true);
    if !enabled {
        return Ok(0);
    }

    // Guard 2: must be a YouTube episode.
    let Some(video_id) = episode.youtube_video_id.clone() else {
        return Ok(0);
    };

    let fetched: Vec<FetchedSegment> = client::fetch_segments(&video_id).await?;

    let now = chrono::Utc::now().naive_utc();
    let episode_secs = episode.total_time as i64;

    let rows: Vec<SponsorSegmentEntity> = fetched
        .into_iter()
        .map(|seg| SponsorSegmentEntity {
            id: uuid::Uuid::new_v4().to_string(),
            episode_id: episode.id.clone(),
            uuid: seg.uuid,
            category: seg.category,
            action_type: seg.action_type,
            start_ms: seg.start_ms,
            end_ms: seg.end_ms,
            votes: seg.votes,
            locked: seg.locked,
            duration_mismatch: durations_mismatch(episode_secs, seg.video_duration_secs),
            fetched_at: now,
        })
        .collect();

    let count = rows.len();
    SponsorblockRepository::replace_segments_for_episode(&episode.id, rows)?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::durations_mismatch;

    #[test]
    fn unknown_durations_never_mismatch() {
        assert!(!durations_mismatch(0, 600.0));
        assert!(!durations_mismatch(600, 0.0));
    }

    #[test]
    fn close_durations_do_not_mismatch() {
        // 600s vs 601s -> diff 1s, tolerance max(6, 2)=6 -> ok.
        assert!(!durations_mismatch(600, 601.0));
    }

    #[test]
    fn large_divergence_flags_mismatch() {
        // 600s vs 540s -> diff 60s, tolerance 6 -> mismatch.
        assert!(durations_mismatch(600, 540.0));
    }

    #[test]
    fn small_videos_use_two_second_floor() {
        // 100s vs 103s -> diff 3s, tolerance max(1.03, 2)=2 -> mismatch.
        assert!(durations_mismatch(100, 103.0));
        // 100s vs 101s -> diff 1s -> ok.
        assert!(!durations_mismatch(100, 101.0));
    }
}
