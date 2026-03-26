use common_infrastructure::error::CustomError;
use podfetch_domain::device::Device;

#[allow(dead_code)]
pub trait CreateUseCase {
    fn create(device_to_safe: Device) -> Result<Device, CustomError>;
}

