use crate::domain::device::Device;
use common_infrastructure::error::CustomError;

#[allow(dead_code)]
pub trait CreateUseCase {
    fn create(device_to_safe: Device) -> Result<Device, CustomError>;
}
