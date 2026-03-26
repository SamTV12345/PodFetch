use podfetch_persistence::db::database;
use podfetch_persistence::adapters::FilterRepositoryImpl;
use common_infrastructure::error::CustomError;
use podfetch_domain::filter::FilterRepository;
use crate::filter::Filter;
use std::sync::Arc;

#[derive(Clone)]
pub struct FilterService {
    repository: Arc<dyn FilterRepository<Error = CustomError>>,
}

impl FilterService {
    pub fn new(repository: Arc<dyn FilterRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn default_service() -> Self {
        Self::new(Arc::new(FilterRepositoryImpl::new(database())))
    }

    pub fn get_filter_by_username(&self, username: &str) -> Result<Option<Filter>, CustomError> {
        self.repository
            .get_by_username(username)
            .map(|filter| filter.map(Into::into))
    }

    pub fn save_filter(&self, filter: Filter) -> Result<(), CustomError> {
        self.repository.save(filter.into())
    }

    pub fn save_timeline_decision(
        &self,
        username: &str,
        only_favored: bool,
    ) -> Result<(), CustomError> {
        self.repository
            .save_timeline_decision(username, only_favored)
    }
}

