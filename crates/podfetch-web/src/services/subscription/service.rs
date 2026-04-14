use crate::subscription::{
    GPodderAvailablePodcast, SubscriptionApplicationService, SubscriptionChangesToClient,
    SubscriptionModelChanges, SubscriptionUpdateRequest,
};
use chrono::DateTime;
use common_infrastructure::error::CustomError;
use podfetch_domain::subscription::SubscriptionRepository;
use std::sync::Arc;

#[derive(Clone)]
pub struct SubscriptionService {
    repository: Arc<dyn SubscriptionRepository<Error = CustomError>>,
}

impl SubscriptionService {
    pub fn new(repository: Arc<dyn SubscriptionRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn delete_by_user_id(&self, user_id: i32) -> Result<(), CustomError> {
        self.repository.delete_by_user_id(user_id)
    }

    pub fn get_device_subscriptions(
        &self,
        device_id: &str,
        user_id: i32,
        since: i32,
    ) -> Result<SubscriptionChangesToClient, CustomError> {
        let timestamp = common_infrastructure::time::get_current_timestamp();
        let since = Self::parse_since(since);
        self.repository
            .get_device_subscriptions(device_id, user_id, since, timestamp)
            .map(Into::into)
            .map(crate::subscription::to_client_changes)
    }

    pub fn get_user_subscriptions(
        &self,
        user_id: i32,
        since: i32,
    ) -> Result<SubscriptionModelChanges, CustomError> {
        let timestamp = common_infrastructure::time::get_current_timestamp();
        let since = Self::parse_since(since);
        self.repository
            .get_user_subscriptions(user_id, since, timestamp)
            .map(Into::into)
    }

    pub fn update_subscriptions(
        &self,
        device_id: &str,
        user_id: i32,
        request: SubscriptionUpdateRequest,
    ) -> Result<Vec<Vec<String>>, CustomError> {
        self.repository
            .update_subscriptions(device_id, user_id, &request.add, &request.remove)
    }

    pub fn get_active_device_podcast_urls(
        &self,
        device_id: &str,
        user_id: i32,
    ) -> Result<Vec<String>, CustomError> {
        self.repository
            .get_active_device_podcast_urls(device_id, user_id)
    }

    pub fn get_available_gpodder_podcasts(
        &self,
    ) -> Result<Vec<GPodderAvailablePodcast>, CustomError> {
        self.repository
            .get_available_gpodder_podcasts()
            .map(|podcasts| podcasts.into_iter().map(Into::into).collect())
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
        user_id: i32,
        since: i32,
    ) -> Result<SubscriptionChangesToClient, Self::Error> {
        self.get_device_subscriptions(device_id, user_id, since)
    }

    fn get_user_subscriptions(
        &self,
        user_id: i32,
        since: i32,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.get_user_subscriptions(user_id, since)
    }

    fn update_subscriptions(
        &self,
        device_id: &str,
        user_id: i32,
        request: SubscriptionUpdateRequest,
    ) -> Result<Vec<Vec<String>>, Self::Error> {
        self.update_subscriptions(device_id, user_id, request)
    }

    fn get_available_gpodder_podcasts(&self) -> Result<Vec<GPodderAvailablePodcast>, Self::Error> {
        self.get_available_gpodder_podcasts()
    }
}
