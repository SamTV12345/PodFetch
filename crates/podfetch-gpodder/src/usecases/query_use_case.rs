use crate::domain::device::Device;
use common_infrastructure::error::CustomError;

#[allow(dead_code)]
pub trait QueryUseCase {
    fn query_by_username(username: &str) -> Result<Vec<Device>, CustomError>;
}
