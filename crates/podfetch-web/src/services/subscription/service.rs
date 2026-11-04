use crate::subscription::{
    GPodderAvailablePodcast, SubscriptionApplicationService, SubscriptionChangesToClient,
    SubscriptionModelChanges, SubscriptionUpdateRequest,
};
use chrono::{DateTime, NaiveDateTime};
use common_infrastructure::error::CustomError;
use common_infrastructure::runtime::DEFAULT_DEVICE;
use podfetch_domain::device_sync_group::DeviceSyncGroupRepository;
use podfetch_domain::subscription::{Subscription, SubscriptionRepository};
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct SubscriptionService {
    repository: Arc<dyn SubscriptionRepository<Error = CustomError>>,
    sync_group_repository: Arc<dyn DeviceSyncGroupRepository<Error = CustomError>>,
}

impl SubscriptionService {
    pub fn new(
        repository: Arc<dyn SubscriptionRepository<Error = CustomError>>,
        sync_group_repository: Arc<dyn DeviceSyncGroupRepository<Error = CustomError>>,
    ) -> Self {
        Self {
            repository,
            sync_group_repository,
        }
    }

    /// All devices whose subscriptions are shared with `device_id`, i.e. the
    /// members of its GPodder sync group (`POST /api/2/sync-devices`).
    /// The requested device is always the first entry; a device that is not
    /// part of any group only yields itself.
    fn synced_devices(&self, device_id: &str, user_id: Uuid) -> Result<Vec<String>, CustomError> {
        let groups = self.sync_group_repository.get_by_user_id(user_id)?;
        let mut devices = vec![device_id.to_string()];
        if let Some(group_id) = groups
            .iter()
            .find(|g| g.device_id == device_id)
            .map(|g| g.group_id)
        {
            for group in groups.iter().filter(|g| g.group_id == group_id) {
                if !devices.contains(&group.device_id) {
                    devices.push(group.device_id.clone());
                }
            }
        }
        Ok(devices)
    }

    /// Merge the subscription rows of several synchronized devices into one
    /// change set: a podcast is subscribed if it is active on any device of
    /// the group, and it is only reported when it changed after `since`.
    fn merge_group_subscriptions(
        rows: Vec<Subscription>,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> podfetch_domain::subscription::SubscriptionModelChanges {
        let mut by_podcast: BTreeMap<String, Vec<Subscription>> = BTreeMap::new();
        for row in rows {
            by_podcast.entry(row.podcast.clone()).or_default().push(row);
        }

        let mut add = vec![];
        let mut remove = vec![];
        for (_, rows) in by_podcast {
            let last_change = rows
                .iter()
                .map(|r| r.deleted.map_or(r.created, |d| d.max(r.created)))
                .max()
                .unwrap_or_default();
            if last_change <= since {
                continue;
            }
            match rows.iter().find(|r| r.deleted.is_none()) {
                Some(active) => add.push(active.clone()),
                None => {
                    if let Some(latest) = rows.into_iter().max_by_key(|r| r.deleted) {
                        remove.push(latest);
                    }
                }
            }
        }

        podfetch_domain::subscription::SubscriptionModelChanges {
            add,
            remove,
            timestamp,
        }
    }

    pub fn delete_by_user_id(&self, user_id: Uuid) -> Result<(), CustomError> {
        self.repository.delete_by_user_id(user_id)
    }

    pub fn get_device_subscriptions(
        &self,
        device_id: &str,
        user_id: Uuid,
        since: i32,
    ) -> Result<SubscriptionChangesToClient, CustomError> {
        let timestamp = common_infrastructure::time::get_current_timestamp();
        let since = Self::parse_since(since);
        let devices = self.synced_devices(device_id, user_id)?;
        if devices.len() == 1 {
            return self
                .repository
                .get_device_subscriptions(device_id, user_id, since, timestamp)
                .map(Into::into)
                .map(crate::subscription::to_client_changes);
        }

        let mut rows = vec![];
        for device in &devices {
            let changes = self.repository.get_device_subscriptions(
                device,
                user_id,
                NaiveDateTime::default(),
                timestamp,
            )?;
            rows.extend(changes.add);
            rows.extend(changes.remove);
        }
        Ok(crate::subscription::to_client_changes(
            Self::merge_group_subscriptions(rows, since, timestamp).into(),
        ))
    }

    pub fn get_user_subscriptions(
        &self,
        user_id: Uuid,
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
        user_id: Uuid,
        request: SubscriptionUpdateRequest,
    ) -> Result<Vec<Vec<String>>, CustomError> {
        let devices = self.synced_devices(device_id, user_id)?;
        let mut update_urls = vec![];
        for (index, device) in devices.iter().enumerate() {
            let rewritten = self.repository.update_subscriptions(
                device,
                user_id,
                &request.add,
                &request.remove,
            )?;
            if index == 0 {
                update_urls = rewritten;
            }
        }

        // A device that shares its subscriptions with the web UI also keeps
        // the web favorites in sync.
        if device_id != DEFAULT_DEVICE && devices.iter().any(|d| d == DEFAULT_DEVICE) {
            Self::sync_favorites(user_id, &request.add, &request.remove);
        }
        Ok(update_urls)
    }

    /// Best effort: a failing favorite update must not fail the client's sync.
    pub fn sync_favorites(user_id: Uuid, add: &[String], remove: &[String]) {
        if let Err(err) =
            crate::services::podcast::service::PodcastService::apply_gpodder_subscriptions_to_favorites(
                user_id, add, remove,
            )
        {
            tracing::warn!("Could not update favorites from GPodder subscriptions: {err:?}");
        }
    }

    pub fn get_active_device_podcast_urls(
        &self,
        device_id: &str,
        user_id: Uuid,
    ) -> Result<Vec<String>, CustomError> {
        let mut urls = vec![];
        for device in self.synced_devices(device_id, user_id)? {
            for url in self
                .repository
                .get_active_device_podcast_urls(&device, user_id)?
            {
                if !urls.contains(&url) {
                    urls.push(url);
                }
            }
        }
        Ok(urls)
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
        user_id: Uuid,
        since: i32,
    ) -> Result<SubscriptionChangesToClient, Self::Error> {
        self.get_device_subscriptions(device_id, user_id, since)
    }

    fn get_user_subscriptions(
        &self,
        user_id: Uuid,
        since: i32,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.get_user_subscriptions(user_id, since)
    }

    fn update_subscriptions(
        &self,
        device_id: &str,
        user_id: Uuid,
        request: SubscriptionUpdateRequest,
    ) -> Result<Vec<Vec<String>>, Self::Error> {
        self.update_subscriptions(device_id, user_id, request)
    }

    fn get_available_gpodder_podcasts(&self) -> Result<Vec<GPodderAvailablePodcast>, Self::Error> {
        self.get_available_gpodder_podcasts()
    }
}
