use crate::models::user::User;
use crate::utils::error::CustomError;
use podfetch_web::sys::{LoginApplicationService, LoginDecision};
use sha256::digest;
use std::sync::Arc;

use crate::service::environment_service::EnvironmentService;

#[derive(Clone)]
pub struct LoginService {
    environment: Arc<EnvironmentService>,
}

impl LoginService {
    pub fn new(environment: Arc<EnvironmentService>) -> Self {
        Self { environment }
    }
}

impl LoginApplicationService for LoginService {
    type Error = CustomError;

    fn verify_login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LoginDecision, Self::Error> {
        let digested_password = digest(password);

        if let Some(admin_username) = &self.environment.username
            && admin_username == username
            && let Some(admin_password) = &self.environment.password
            && admin_password == &digested_password
        {
            return Ok(LoginDecision::Authenticated);
        }

        match User::find_by_username(username) {
            Ok(user) => {
                if let Some(stored_password) = user.password {
                    if stored_password == digested_password {
                        Ok(LoginDecision::Authenticated)
                    } else {
                        Ok(LoginDecision::Forbidden)
                    }
                } else {
                    Ok(LoginDecision::Forbidden)
                }
            }
            Err(err) => {
                if matches!(err.inner, crate::utils::error::CustomErrorInner::NotFound(_)) {
                    Ok(LoginDecision::WrongUserOrPassword)
                } else {
                    Err(err)
                }
            }
        }
    }
}
