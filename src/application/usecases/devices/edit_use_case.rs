use crate::utils::error::CustomError;

#[allow(dead_code)]
pub trait EditUseCase {
    fn delete_by_username(username: &str) -> Result<(), CustomError>;
}
