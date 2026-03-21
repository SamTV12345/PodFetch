use crate::utils::error::CustomError;
use podfetch_domain::device::{Device, DeviceRepository};
use podfetch_web::device::DeviceApplicationService;
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

    pub fn query_by_username(&self, username: &str) -> Result<Vec<Device>, CustomError> {
        self.repository.get_devices_of_user(username)
    }

    pub fn delete_by_username(&self, username: &str) -> Result<(), CustomError> {
        self.repository.delete_by_username(username)
    }
}

impl DeviceApplicationService for DeviceService {
    type Error = CustomError;

    fn create(&self, device: Device) -> Result<Device, Self::Error> {
        self.create(device)
    }

    fn query_by_username(&self, username: &str) -> Result<Vec<Device>, Self::Error> {
        self.query_by_username(username)
    }
}
