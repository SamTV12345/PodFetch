use crate::domain::filter::FilterRepository;
use crate::dto::filter::Filter;
use common_infrastructure::error::CustomError;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct FilterService {
    repository: Arc<dyn FilterRepository<Error = CustomError>>,
}

impl FilterService {
    pub fn new(repository: Arc<dyn FilterRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn get_filter_by_user_id(&self, user_id: Uuid) -> Result<Option<Filter>, CustomError> {
        self.repository
            .get_by_user_id(user_id)
            .map(|filter| filter.map(Into::into))
    }

    pub fn save_filter(&self, filter: Filter) -> Result<(), CustomError> {
        self.repository.save(filter.into())
    }

    pub fn save_timeline_decision(
        &self,
        user_id: Uuid,
        only_favored: bool,
    ) -> Result<(), CustomError> {
        self.repository
            .save_timeline_decision(user_id, only_favored)
    }
}
