use crate::utils::error::CustomError;

pub trait EditUseCase {
    fn delete_by_username(username: &str) -> Result<(), CustomError>;
}
