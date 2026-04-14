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
