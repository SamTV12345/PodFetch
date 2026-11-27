use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodcastEpisodeChapter {
    pub id: String,
    pub episode_id: Uuid,
    pub title: String,
    pub start_time: i32,
    pub end_time: i32,
    pub href: Option<String>,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertPodcastEpisodeChapter {
    pub episode_id: Uuid,
    pub title: String,
    pub start_time: i32,
    pub end_time: i32,
    pub href: Option<String>,
    pub image: Option<String>,
}

pub trait PodcastEpisodeChapterRepository: Send + Sync {
    type Error;

    fn upsert(&self, chapter: UpsertPodcastEpisodeChapter) -> Result<(), Self::Error>;

    fn get_by_episode_id(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<PodcastEpisodeChapter>, Self::Error>;
    /// Removes the rows of the episodes (they leave the catalog).
    fn delete_by_episode_ids(&self, episode_ids: &[Uuid]) -> Result<(), Self::Error>;
}
