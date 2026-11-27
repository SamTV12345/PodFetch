use chrono::NaiveDateTime;
use common_infrastructure::error::CustomError;

/// What the media library knows about the file of one catalog episode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpisodeFile {
    pub episode_id: String,
    pub download_time: Option<NaiveDateTime>,
    pub file_episode_path: Option<String>,
    pub file_image_path: Option<String>,
    /// The storage the file lives on (local or S3).
    pub download_location: Option<String>,
    /// The user deleted the file; it isn't downloaded again on its own.
    pub deleted: bool,
}

impl EpisodeFile {
    /// The state of an episode that was never downloaded.
    pub fn none(episode_id: &str) -> Self {
        Self {
            episode_id: episode_id.to_string(),
            ..Self::default()
        }
    }

    /// Nothing is known about the file: not downloaded, not deleted.
    pub fn is_empty(&self) -> bool {
        self.download_time.is_none()
            && self.file_episode_path.is_none()
            && self.file_image_path.is_none()
            && self.download_location.is_none()
            && !self.deleted
    }
}

pub trait EpisodeFileRepository: Send + Sync {
    fn find(&self, episode_id: &str) -> Result<Option<EpisodeFile>, CustomError>;

    /// Applies `change` to the episode's file state in one transaction. A
    /// state left empty is not stored.
    fn modify(
        &self,
        episode_id: &str,
        change: &mut dyn FnMut(&mut EpisodeFile),
    ) -> Result<(), CustomError>;

    /// Forgets the file state of the episodes (they leave the catalog).
    fn forget(&self, episode_ids: &[String]) -> Result<(), CustomError>;
}
