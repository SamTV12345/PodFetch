use crate::application::services::user_auth::service::UserAuthService;
use crate::utils::error::CustomError;
use common_infrastructure::config::EnvironmentService;
use podfetch_web::sys::{LoginApplicationService, LoginDecision};
use sha256::digest;
use std::sync::Arc;

#[derive(Clone)]
pub struct LoginService {
    environment: Arc<EnvironmentService>,
    user_auth_service: Arc<UserAuthService>,
}

impl LoginService {
    pub fn new(
        environment: Arc<EnvironmentService>,
        user_auth_service: Arc<UserAuthService>,
    ) -> Self {
        Self {
            environment,
            user_auth_service,
        }
    }
}

impl LoginApplicationService for LoginService {
    type Error = CustomError;

    fn verify_login(&self, username: &str, password: &str) -> Result<LoginDecision, Self::Error> {
        let digested_password = digest(password);

        if let Some(admin_username) = &self.environment.username
            && admin_username == username
            && let Some(admin_password) = &self.environment.password
            && admin_password == &digested_password
        {
            return Ok(LoginDecision::Authenticated);
        }

        match self.user_auth_service.find_by_username(username) {
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
                if matches!(
                    err.inner,
                    crate::utils::error::CustomErrorInner::NotFound(_)
                ) {
                    Ok(LoginDecision::WrongUserOrPassword)
                } else {
                    Err(err)
                }
            }
        }
    }
}
