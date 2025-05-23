use crate::domain::models::device::model::Device;
use crate::utils::error::CustomError;

pub trait CreateUseCase {
    fn create(device_to_safe: Device) -> Result<Device, CustomError>;
}
