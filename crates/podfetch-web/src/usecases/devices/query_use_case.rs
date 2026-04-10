use common_infrastructure::error::CustomError;
use podfetch_domain::device::Device;

#[allow(dead_code)]
pub trait QueryUseCase {
    fn query_by_username(username: &str) -> Result<Vec<Device>, CustomError>;
}
