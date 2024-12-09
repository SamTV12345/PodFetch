use crate::dbconfig::DBType;
use crate::domain::models::device::model::Device;
use crate::utils::error::CustomError;

pub trait DeviceRepository {
    fn create(device: Device, conn: &mut DBType) -> Result<Device, CustomError>;
    fn get_devices_of_user(username: String, conn: &mut DBType) -> Result<Vec<Device>, CustomError>;
    fn delete_by_username(username: &str, conn: &mut DBType) -> Result<(), CustomError>;
}