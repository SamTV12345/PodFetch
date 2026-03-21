use crate::service::invite_service::InviteService;
use crate::utils::error::ErrorSeverity::{Debug, Warning};
use crate::utils::error::{CustomError, CustomErrorInner};
use podfetch_domain::user_admin::{ManagedUser, UserAdminRepository, UserSummary};
use podfetch_web::user_onboarding::{UserOnboardingApplicationService, UserOnboardingModel};
use sha256::digest;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserOnboardingService {
    invite_service: Arc<InviteService>,
    repository: Arc<dyn UserAdminRepository<Error = CustomError>>,
}

impl UserOnboardingService {
    pub fn new(
        invite_service: Arc<InviteService>,
        repository: Arc<dyn UserAdminRepository<Error = CustomError>>,
    ) -> Self {
        Self {
            invite_service,
            repository,
        }
    }

    fn is_valid_password(password: &str) -> bool {
        let mut has_whitespace = false;
        let mut has_upper = false;
        let mut has_lower = false;
        let mut has_digit = false;

        for c in password.chars() {
            has_whitespace |= c.is_whitespace();
            has_lower |= c.is_lowercase();
            has_upper |= c.is_uppercase();
            has_digit |= c.is_ascii_digit();
        }

        !has_whitespace && has_upper && has_lower && has_digit && password.len() >= 8
    }

    pub fn register_user(
        &self,
        username: String,
        password: String,
        invite_id: String,
    ) -> Result<UserSummary, CustomError> {
        if !Self::is_valid_password(&password) {
            return Err(
                CustomErrorInner::Conflict("Password is not valid".to_string(), Warning).into(),
            );
        }

        if self.repository.find_by_username(&username)?.is_some() {
            return Err(
                CustomErrorInner::Conflict("Username already taken".to_string(), Warning).into(),
            );
        }

        let invite = self
            .invite_service
            .find_optional_invite(&invite_id)?
            .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;

        if invite.accepted_at.is_some() {
            return Err(
                CustomErrorInner::Conflict("Invite already accepted".to_string(), Warning).into(),
            );
        }

        let user = self.repository.create(ManagedUser {
            id: 0,
            username,
            role: invite.role,
            password: Some(digest(password)),
            explicit_consent: invite.explicit_consent,
            created_at: chrono::Utc::now().naive_utc(),
            api_key: None,
        })?;

        self.invite_service.invalidate_invite(&invite_id)?;
        Ok(user.to_summary())
    }
}

impl UserOnboardingApplicationService for UserOnboardingService {
    type Error = CustomError;

    fn onboard_user(&self, request: UserOnboardingModel) -> Result<UserSummary, Self::Error> {
        self.register_user(request.username, request.password, request.invite_id)
    }
}
