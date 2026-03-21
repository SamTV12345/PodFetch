use crate::utils::error::CustomError;
use chrono::NaiveDateTime;
use podfetch_domain::listening_event::{
    ListeningEvent, ListeningEventRepository, NewListeningEvent,
};
use podfetch_persistence::db::Database;
use podfetch_persistence::listening_event::DieselListeningEventRepository;

pub struct ListeningEventRepositoryImpl {
    inner: DieselListeningEventRepository,
}

impl ListeningEventRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselListeningEventRepository::new(database),
        }
    }
}

impl ListeningEventRepository for ListeningEventRepositoryImpl {
    type Error = CustomError;

    fn create(&self, event: NewListeningEvent) -> Result<ListeningEvent, Self::Error> {
        self.inner.create(event).map_err(Into::into)
    }

    fn get_by_user_and_range(
        &self,
        username: &str,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
    ) -> Result<Vec<ListeningEvent>, Self::Error> {
        self.inner
            .get_by_user_and_range(username, from, to)
            .map_err(Into::into)
    }

    fn delete_by_username(&self, username: &str) -> Result<usize, Self::Error> {
        self.inner.delete_by_username(username).map_err(Into::into)
    }

    fn delete_by_podcast_id(&self, podcast_id: i32) -> Result<usize, Self::Error> {
        self.inner
            .delete_by_podcast_id(podcast_id)
            .map_err(Into::into)
    }
}
