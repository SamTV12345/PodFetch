use common_infrastructure::error::CustomError;
use podfetch_domain::filter::{Filter, FilterRepository};
use podfetch_persistence::db::Database;
use podfetch_persistence::filter::DieselFilterRepository;

pub struct FilterRepositoryImpl {
    inner: DieselFilterRepository,
}

impl FilterRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselFilterRepository::new(database),
        }
    }
}

impl FilterRepository for FilterRepositoryImpl {
    type Error = CustomError;

    fn get_by_username(&self, username: &str) -> Result<Option<Filter>, Self::Error> {
        self.inner.get_by_username(username).map_err(Into::into)
    }

    fn save(&self, filter: Filter) -> Result<(), Self::Error> {
        self.inner.save(filter).map_err(Into::into)
    }

    fn save_timeline_decision(
        &self,
        username: &str,
        only_favored: bool,
    ) -> Result<(), Self::Error> {
        self.inner
            .save_timeline_decision(username, only_favored)
            .map_err(Into::into)
    }
}

