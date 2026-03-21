use crate::utils::error::CustomError;
use chrono::DateTime;
use podfetch_domain::subscription::{
    GPodderAvailablePodcast, SubscriptionChangesToClient, SubscriptionModelChanges,
    SubscriptionRepository,
};
use podfetch_web::subscription::{SubscriptionApplicationService, SubscriptionUpdateRequest};
use std::sync::Arc;

#[derive(Clone)]
pub struct SubscriptionService {
    repository: Arc<dyn SubscriptionRepository<Error = CustomError>>,
}

impl SubscriptionService {
    pub fn new(repository: Arc<dyn SubscriptionRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn delete_by_username(&self, username: &str) -> Result<(), CustomError> {
        self.repository.delete_by_username(username)
    }

    pub fn get_device_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        since: i32,
    ) -> Result<SubscriptionChangesToClient, CustomError> {
        let timestamp = crate::utils::time::get_current_timestamp();
        let since = Self::parse_since(since);
        self.repository
            .get_device_subscriptions(device_id, username, since, timestamp)
            .map(Into::into)
    }

    pub fn get_user_subscriptions(
        &self,
        username: &str,
        since: i32,
    ) -> Result<SubscriptionModelChanges, CustomError> {
        let timestamp = crate::utils::time::get_current_timestamp();
        let since = Self::parse_since(since);
        self.repository
            .get_user_subscriptions(username, since, timestamp)
    }

    pub fn update_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        request: SubscriptionUpdateRequest,
    ) -> Result<Vec<Vec<String>>, CustomError> {
        self.repository
            .update_subscriptions(device_id, username, &request.add, &request.remove)
    }

    pub fn get_available_gpodder_podcasts(
        &self,
    ) -> Result<Vec<GPodderAvailablePodcast>, CustomError> {
        self.repository.get_available_gpodder_podcasts()
    }

    fn parse_since(since: i32) -> chrono::NaiveDateTime {
        DateTime::from_timestamp(since as i64, 0)
            .map(|value| value.naive_utc())
            .unwrap_or_default()
    }
}

impl SubscriptionApplicationService for SubscriptionService {
    type Error = CustomError;

    fn get_device_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        since: i32,
    ) -> Result<SubscriptionChangesToClient, Self::Error> {
        self.get_device_subscriptions(device_id, username, since)
    }

    fn get_user_subscriptions(
        &self,
        username: &str,
        since: i32,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.get_user_subscriptions(username, since)
    }

    fn update_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        request: SubscriptionUpdateRequest,
    ) -> Result<Vec<Vec<String>>, Self::Error> {
        self.update_subscriptions(device_id, username, request)
    }
}
