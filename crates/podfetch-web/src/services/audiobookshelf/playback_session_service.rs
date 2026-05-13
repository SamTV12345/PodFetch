use common_infrastructure::error::CustomError;
use podfetch_domain::audiobookshelf::playback_session::{
    PlaybackSession, PlaybackSessionRepository,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AudiobookshelfPlaybackSessionService {
    repository: Arc<dyn PlaybackSessionRepository<Error = CustomError>>,
}

impl AudiobookshelfPlaybackSessionService {
    pub fn new(repository: Arc<dyn PlaybackSessionRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn create(&self, session: PlaybackSession) -> Result<PlaybackSession, CustomError> {
        self.repository.create(session)
    }

    pub fn find_by_id(&self, id: &str) -> Result<Option<PlaybackSession>, CustomError> {
        self.repository.find_by_id(id)
    }

    pub fn update(&self, session: PlaybackSession) -> Result<PlaybackSession, CustomError> {
        self.repository.update(session)
    }

    pub fn delete(&self, id: &str) -> Result<usize, CustomError> {
        self.repository.delete(id)
    }
}
