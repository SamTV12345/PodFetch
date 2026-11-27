use chrono::NaiveDateTime;
use common_infrastructure::db::Database;
use common_infrastructure::error::CustomError;
use uuid::Uuid;

// ── DeviceSyncGroup ─────────────────────────────────────────────────────────

use crate::domain::device_sync_group::{DeviceSyncGroup, DeviceSyncGroupRepository};
use crate::persistence::device_sync_group::DieselDeviceSyncGroupRepository;

pub struct DeviceSyncGroupRepositoryImpl {
    inner: DieselDeviceSyncGroupRepository,
}

impl DeviceSyncGroupRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselDeviceSyncGroupRepository::new(database),
        }
    }
}

impl DeviceSyncGroupRepository for DeviceSyncGroupRepositoryImpl {
    type Error = CustomError;

    fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<DeviceSyncGroup>, Self::Error> {
        self.inner.get_by_user_id(user_id).map_err(Into::into)
    }

    fn replace_all(&self, user_id: Uuid, groups: Vec<DeviceSyncGroup>) -> Result<(), Self::Error> {
        self.inner.replace_all(user_id, groups).map_err(Into::into)
    }
}

// ── Device ──────────────────────────────────────────────────────────────────

use crate::domain::device::{Device, DeviceRepository};
use crate::persistence::device::DieselDeviceRepository;

pub struct DeviceRepositoryImpl {
    inner: DieselDeviceRepository,
}

impl DeviceRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselDeviceRepository::new(database),
        }
    }
}

impl DeviceRepository for DeviceRepositoryImpl {
    type Error = CustomError;

    fn create(&self, device: Device) -> Result<Device, CustomError> {
        self.inner.create(device).map_err(Into::into)
    }

    fn get_devices_of_user(&self, user_id_to_find: Uuid) -> Result<Vec<Device>, CustomError> {
        self.inner
            .get_devices_of_user(user_id_to_find)
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id: Uuid) -> Result<(), CustomError> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }
}

// ── GpodderSetting ──────────────────────────────────────────────────────────

use crate::domain::gpodder_setting::{GpodderSetting, GpodderSettingRepository};
use crate::persistence::gpodder_setting::DieselGpodderSettingRepository;

pub struct GpodderSettingRepositoryImpl {
    inner: DieselGpodderSettingRepository,
}

impl GpodderSettingRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselGpodderSettingRepository::new(database),
        }
    }
}

impl GpodderSettingRepository for GpodderSettingRepositoryImpl {
    type Error = CustomError;

    fn get_setting(
        &self,
        user_id: Uuid,
        scope: &str,
        scope_id: Option<&str>,
    ) -> Result<Option<GpodderSetting>, Self::Error> {
        self.inner
            .get_setting(user_id, scope, scope_id)
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id: Uuid) -> Result<usize, Self::Error> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }

    fn save_setting(&self, setting: GpodderSetting) -> Result<GpodderSetting, Self::Error> {
        self.inner.save_setting(setting).map_err(Into::into)
    }
}

// ── Subscription ────────────────────────────────────────────────────────────

use crate::domain::subscription::{
    GPodderAvailablePodcast, SubscriptionModelChanges, SubscriptionRepository,
};
use crate::persistence::subscription::DieselSubscriptionRepository;

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

    fn delete_by_user_id(&self, user_id: Uuid) -> Result<(), Self::Error> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }

    fn get_device_subscriptions(
        &self,
        device_id: &str,
        user_id: Uuid,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.inner
            .get_device_subscriptions(device_id, user_id, since, timestamp)
            .map_err(Into::into)
    }

    fn get_user_subscriptions(
        &self,
        user_id: Uuid,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.inner
            .get_user_subscriptions(user_id, since, timestamp)
            .map_err(Into::into)
    }

    fn update_subscriptions(
        &self,
        device_id: &str,
        user_id: Uuid,
        add: &[String],
        remove: &[String],
    ) -> Result<Vec<Vec<String>>, Self::Error> {
        self.inner
            .update_subscriptions(device_id, user_id, add, remove)
            .map_err(Into::into)
    }

    fn get_subscribed_feeds(&self) -> Result<Vec<GPodderAvailablePodcast>, Self::Error> {
        self.inner.get_subscribed_feeds().map_err(Into::into)
    }

    fn get_active_device_podcast_urls(
        &self,
        device_id: &str,
        user_id: Uuid,
    ) -> Result<Vec<String>, Self::Error> {
        self.inner
            .get_active_device_podcast_urls(device_id, user_id)
            .map_err(Into::into)
    }
}
