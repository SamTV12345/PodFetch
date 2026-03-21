use crate::utils::error::CustomError;
use podfetch_domain::subscription::{
    GPodderAvailablePodcast, SubscriptionModelChanges, SubscriptionRepository,
};
use podfetch_persistence::db::Database;
use podfetch_persistence::subscription::DieselSubscriptionRepository;

pub struct SubscriptionRepositoryImpl {
    inner: DieselSubscriptionRepository,
}

impl SubscriptionRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSubscriptionRepository::new(database),
        }
    }
}

impl SubscriptionRepository for SubscriptionRepositoryImpl {
    type Error = CustomError;

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error> {
        self.inner.delete_by_username(username).map_err(Into::into)
    }

    fn get_device_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        since: chrono::NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.inner
            .get_device_subscriptions(device_id, username, since, timestamp)
            .map_err(Into::into)
    }

    fn get_user_subscriptions(
        &self,
        username: &str,
        since: chrono::NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.inner
            .get_user_subscriptions(username, since, timestamp)
            .map_err(Into::into)
    }

    fn update_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        add: &[String],
        remove: &[String],
    ) -> Result<Vec<Vec<String>>, Self::Error> {
        self.inner
            .update_subscriptions(device_id, username, add, remove)
            .map_err(Into::into)
    }

    fn get_available_gpodder_podcasts(&self) -> Result<Vec<GPodderAvailablePodcast>, Self::Error> {
        self.inner
            .get_available_gpodder_podcasts()
            .map_err(Into::into)
    }
}
