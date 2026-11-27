use chrono::NaiveDateTime;
use common_infrastructure::error::CustomError;
use podfetch_kernel::UserId;

/// A stretch of an episode SponsorBlock marks as skippable.
#[derive(Debug, Clone, PartialEq)]
pub struct SponsorSegment {
    /// SponsorBlock's id of the segment.
    pub uuid: String,
    pub category: String,
    pub action_type: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub votes: i32,
    pub locked: bool,
    /// The episode's length differs from the video SponsorBlock measured,
    /// so the segment may be off.
    pub duration_mismatch: bool,
    pub fetched_at: NaiveDateTime,
}

/// Which segments a user wants the player to skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipPreferences {
    pub enabled: bool,
    pub skip_sponsor: bool,
    pub skip_selfpromo: bool,
    pub skip_interaction: bool,
    pub skip_intro: bool,
    pub skip_outro: bool,
    pub skip_preview: bool,
    pub skip_filler: bool,
    pub skip_music_offtopic: bool,
}

impl Default for SkipPreferences {
    /// What a user gets before saving preferences. Matches the column
    /// defaults of the `2026-06-05-120000_sponsorblock` migration: enabled,
    /// skipping sponsors and self-promotion only.
    fn default() -> Self {
        Self {
            enabled: true,
            skip_sponsor: true,
            skip_selfpromo: true,
            skip_interaction: false,
            skip_intro: false,
            skip_outro: false,
            skip_preview: false,
            skip_filler: false,
            skip_music_offtopic: false,
        }
    }
}

pub trait SponsorblockRepository: Send + Sync {
    /// Replaces all stored segments of the episode with `segments`.
    fn replace_segments(
        &self,
        episode_id: &str,
        segments: &[SponsorSegment],
    ) -> Result<(), CustomError>;
    /// The episode's segments, ordered by start.
    fn segments(&self, episode_id: &str) -> Result<Vec<SponsorSegment>, CustomError>;
    fn preferences(&self, user_id: UserId) -> Result<Option<SkipPreferences>, CustomError>;
    fn save_preferences(
        &self,
        user_id: UserId,
        preferences: &SkipPreferences,
    ) -> Result<(), CustomError>;
    fn delete_preferences(&self, user_id: UserId) -> Result<(), CustomError>;
}
