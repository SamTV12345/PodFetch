use crate::DbPool;
use crate::domain::models::device::model::Device;
use crate::utils::error::CustomError;

pub trait QueryUseCase {
    fn query_by_username(username: String, pool: &DbPool) -> Result<Vec<Device>, CustomError>;
}