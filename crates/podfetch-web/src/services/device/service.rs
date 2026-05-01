use crate::device::DeviceApplicationService;
use common_infrastructure::error::CustomError;
use podfetch_domain::device::{Device, DeviceRepository};
use std::sync::Arc;

pub struct DeviceService {
    repository: Arc<dyn DeviceRepository<Error = CustomError>>,
}

impl DeviceService {
    pub fn new(repository: Arc<dyn DeviceRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn create(&self, device_to_safe: Device) -> Result<Device, CustomError> {
        self.repository.create(device_to_safe)
    }

    pub fn query_by_user_id(&self, user_id: i32) -> Result<Vec<Device>, CustomError> {
        self.repository.get_devices_of_user(user_id)
    }

    pub fn delete_by_user_id(&self, user_id: i32) -> Result<(), CustomError> {
        self.repository.delete_by_user_id(user_id)
    }

    pub fn list_castable_for_user(&self, user_id: i32) -> Result<Vec<Device>, CustomError> {
        self.repository.list_castable_for_user(user_id)
    }

    pub fn find_by_chromecast_uuid(
        &self,
        chromecast_uuid: &str,
    ) -> Result<Option<Device>, CustomError> {
        self.repository.find_by_chromecast_uuid(chromecast_uuid)
    }

    pub fn upsert_chromecast_from_agent(
        &self,
        chromecast_uuid: &str,
        agent_id: &str,
        owner_user_id: i32,
        name: &str,
        ip: Option<&str>,
        last_seen_at: chrono::NaiveDateTime,
    ) -> Result<Device, CustomError> {
        self.repository.upsert_chromecast_from_agent(
            chromecast_uuid,
            agent_id,
            owner_user_id,
            name,
            ip,
            last_seen_at,
        )
    }
}

impl DeviceApplicationService for DeviceService {
    type Error = CustomError;

    fn create(&self, device: Device) -> Result<Device, Self::Error> {
        self.create(device)
    }

    fn query_by_user_id(&self, user_id: i32) -> Result<Vec<Device>, Self::Error> {
        self.query_by_user_id(user_id)
    }
}
