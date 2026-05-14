use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct MediaProgress {
    pub id: String,
    pub user_id: i32,
    pub library_item_id: String,
    pub episode_id: Option<String>,
    pub media_type: String,
    pub duration: f64,
    pub current_time: f64,
    pub progress: f64,
    pub is_finished: bool,
    pub hide_from_continue_listening: bool,
    pub last_update: NaiveDateTime,
    pub started_at: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
}

impl MediaProgress {
    pub fn compose_id(library_item_id: &str, episode_id: Option<&str>) -> String {
        match episode_id {
            Some(ep) => format!("{library_item_id}-{ep}"),
            None => library_item_id.to_string(),
        }
    }
}

pub trait MediaProgressRepository: Send + Sync {
    type Error;

    fn list_for_user(&self, user_id: i32) -> Result<Vec<MediaProgress>, Self::Error>;
    fn find(
        &self,
        user_id: i32,
        library_item_id: &str,
        episode_id: Option<&str>,
    ) -> Result<Option<MediaProgress>, Self::Error>;
    fn upsert(&self, progress: MediaProgress) -> Result<MediaProgress, Self::Error>;
    fn delete(&self, id: &str) -> Result<usize, Self::Error>;
}
