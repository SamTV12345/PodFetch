use chrono::NaiveDateTime;

#[derive(Clone, Debug, Default)]
pub struct PodcastEpisode {
    pub(crate) id: i32,
    pub(crate) podcast_id: i32,
    pub(crate) episode_id: String,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) date_of_recording: String,
    pub image_url: String,
    pub total_time: i32,
    pub(crate) local_url: String,
    pub(crate) local_image_url: String,
    pub(crate) description: String,
    pub(crate) status: String,
    pub(crate) download_time: Option<NaiveDateTime>,
    pub(crate) guid: String,
    pub(crate) deleted: bool,
    pub(crate) file_episode_path: Option<String>,
    pub(crate) file_image_path: Option<String>,
    pub (crate) episode_numbering_processed : bool,
}