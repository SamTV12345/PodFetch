use chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListeningEvent {
    pub id: i32,
    pub user_id: i32,
    pub device: String,
    pub podcast_episode_id: String,
    pub podcast_id: i32,
    pub podcast_episode_db_id: i32,
    pub delta_seconds: i32,
    pub start_position: i32,
    pub end_position: i32,
    pub listened_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewListeningEvent {
    pub user_id: i32,
    pub device: String,
    pub podcast_episode_id: String,
    pub podcast_id: i32,
    pub podcast_episode_db_id: i32,
    pub delta_seconds: i32,
    pub start_position: i32,
    pub end_position: i32,
    pub listened_at: NaiveDateTime,
}

pub trait ListeningEventRepository: Send + Sync {
    type Error;

    fn create(&self, event: NewListeningEvent) -> Result<ListeningEvent, Self::Error>;
    fn get_by_user_and_range(
        &self,
        user_id: i32,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
    ) -> Result<Vec<ListeningEvent>, Self::Error>;
    fn delete_by_user_id(&self, user_id: i32) -> Result<usize, Self::Error>;
    fn delete_by_podcast_id(&self, podcast_id: i32) -> Result<usize, Self::Error>;
}
