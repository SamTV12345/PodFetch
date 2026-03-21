use chrono::NaiveDateTime;

/// A podcast episode - technology-agnostic domain entity.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PodcastEpisode {
    pub id: i32,
    pub podcast_id: i32,
    pub episode_id: String,
    pub name: String,
    pub url: String,
    pub date_of_recording: String,
    pub image_url: String,
    pub total_time: i32,
    pub description: String,
    pub download_time: Option<NaiveDateTime>,
    pub guid: String,
    pub deleted: bool,
    pub file_episode_path: Option<String>,
    pub file_image_path: Option<String>,
    pub episode_numbering_processed: bool,
    pub download_location: Option<String>,
}

impl PodcastEpisode {
    pub fn is_downloaded(&self) -> bool {
        self.download_location.is_some()
    }
}

/// Data for creating a new podcast episode.
#[derive(Debug, Clone)]
pub struct NewPodcastEpisode {
    pub podcast_id: i32,
    pub episode_id: String,
    pub name: String,
    pub url: String,
    pub date_of_recording: String,
    pub image_url: String,
    pub total_time: i32,
    pub description: String,
    pub guid: String,
}

/// Repository trait for PodcastEpisode persistence operations.
pub trait PodcastEpisodeRepository: Send + Sync {
    type Error;

    fn create(&self, episode: NewPodcastEpisode) -> Result<PodcastEpisode, Self::Error>;
    fn find_by_id(&self, id: i32) -> Result<Option<PodcastEpisode>, Self::Error>;
    fn find_by_episode_id(&self, episode_id: &str) -> Result<Option<PodcastEpisode>, Self::Error>;
    fn find_by_url(
        &self,
        url: &str,
        podcast_id: Option<i32>,
    ) -> Result<Option<PodcastEpisode>, Self::Error>;
    fn find_by_guid(&self, guid: &str) -> Result<Option<PodcastEpisode>, Self::Error>;
    fn find_by_podcast_id(&self, podcast_id: i32) -> Result<Vec<PodcastEpisode>, Self::Error>;
    fn find_by_file_path(&self, path: &str) -> Result<Option<PodcastEpisode>, Self::Error>;
    fn update(&self, episode: &PodcastEpisode) -> Result<(), Self::Error>;
    fn delete(&self, id: i32) -> Result<(), Self::Error>;
    fn delete_by_podcast_id(&self, podcast_id: i32) -> Result<(), Self::Error>;
}
