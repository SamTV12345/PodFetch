use chrono::NaiveDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMethod {
    Direct,
    HlsTranscode,
}

impl PlayMethod {
    pub fn as_i32(&self) -> i32 {
        match self {
            PlayMethod::Direct => 0,
            PlayMethod::HlsTranscode => 1,
        }
    }

    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => PlayMethod::HlsTranscode,
            _ => PlayMethod::Direct,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaybackSession {
    pub id: String,
    pub user_id: i32,
    pub library_id: Option<String>,
    pub library_item_id: String,
    pub episode_id: Option<String>,
    pub media_type: String,
    pub play_method: PlayMethod,
    pub current_time: f64,
    pub duration: f64,
    pub started_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
    pub time_listening_total: f64,
    pub display_title: Option<String>,
    pub display_author: Option<String>,
    pub cover_path: Option<String>,
    pub media_metadata_json: Option<String>,
    pub device_info_json: Option<String>,
}

pub trait PlaybackSessionRepository: Send + Sync {
    type Error;

    fn create(&self, session: PlaybackSession) -> Result<PlaybackSession, Self::Error>;
    fn find_by_id(&self, id: &str) -> Result<Option<PlaybackSession>, Self::Error>;
    fn update(&self, session: PlaybackSession) -> Result<PlaybackSession, Self::Error>;
    fn delete(&self, id: &str) -> Result<usize, Self::Error>;
}
