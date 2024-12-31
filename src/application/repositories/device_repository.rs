use crate::domain::models::device::model::Device;
use crate::utils::error::CustomError;

pub trait DeviceRepository {
    fn create(device: Device) -> Result<Device, CustomError>;
    fn get_devices_of_user(username: &str) -> Result<Vec<Device>, CustomError>;
    fn delete_by_username(username: &str) -> Result<(), CustomError>;
}
