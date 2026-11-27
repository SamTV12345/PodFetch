use common_infrastructure::db::Database;
use common_infrastructure::error::CustomError;
use uuid::Uuid;

// ── ListeningEvent ──────────────────────────────────────────────────────────

use crate::domain::listening_event::{ListeningEvent, ListeningEventRepository, NewListeningEvent};
use crate::persistence::listening_event::DieselListeningEventRepository;
use chrono::NaiveDateTime;

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
        user_id: Uuid,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
    ) -> Result<Vec<ListeningEvent>, Self::Error> {
        self.inner
            .get_by_user_and_range(user_id, from, to)
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id: Uuid) -> Result<usize, Self::Error> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }

    fn delete_by_podcast_id(&self, podcast_id: Uuid) -> Result<usize, Self::Error> {
        self.inner
            .delete_by_podcast_id(podcast_id)
            .map_err(Into::into)
    }
}
