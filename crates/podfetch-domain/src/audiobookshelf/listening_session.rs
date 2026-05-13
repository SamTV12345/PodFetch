use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct ListeningSession {
    pub id: String,
    pub user_id: i32,
    pub library_id: Option<String>,
    pub library_item_id: String,
    pub episode_id: Option<String>,
    pub media_type: String,
    pub duration: f64,
    pub current_time: f64,
    pub time_listening: f64,
    pub play_method: i32,
    pub started_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub display_title: Option<String>,
    pub display_author: Option<String>,
    pub cover_path: Option<String>,
}

pub trait ListeningSessionRepository: Send + Sync {
    type Error;

    fn create(&self, session: ListeningSession) -> Result<ListeningSession, Self::Error>;
    fn list_for_user(&self, user_id: i32, limit: i64) -> Result<Vec<ListeningSession>, Self::Error>;
}
