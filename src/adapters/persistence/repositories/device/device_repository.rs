use common_infrastructure::error::CustomError;
use podfetch_domain::device::{Device, DeviceRepository};
use podfetch_persistence::db::Database;
use podfetch_persistence::device::DieselDeviceRepository;

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

    fn get_devices_of_user(&self, username_to_find: &str) -> Result<Vec<Device>, CustomError> {
        self.inner
            .get_devices_of_user(username_to_find)
            .map_err(Into::into)
    }

    fn delete_by_username(&self, username1: &str) -> Result<(), CustomError> {
        self.inner.delete_by_username(username1).map_err(Into::into)
    }
}

