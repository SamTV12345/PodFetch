use std::fmt::Display;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub enum LoginDecision {
    Authenticated,
    WrongUserOrPassword,
    Forbidden,
}

pub trait LoginApplicationService {
    type Error;

    fn verify_login(&self, username: &str, password: &str) -> Result<LoginDecision, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum LoginControllerError<E: Display> {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("{0}")]
    Service(E),
}

pub fn login<S>(service: &S, request: &LoginRequest) -> Result<(), LoginControllerError<S::Error>>
where
    S: LoginApplicationService,
    S::Error: Display,
{
    match service
        .verify_login(&request.username, &request.password)
        .map_err(LoginControllerError::Service)?
    {
        LoginDecision::Authenticated => Ok(()),
        LoginDecision::WrongUserOrPassword => Err(LoginControllerError::Unauthorized),
        LoginDecision::Forbidden => Err(LoginControllerError::Forbidden),
    }
}
