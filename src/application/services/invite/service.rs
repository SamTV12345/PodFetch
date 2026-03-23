use crate::constants::inner_constants::Role;
use crate::utils::error::ErrorSeverity::{Debug, Error, Warning};
use crate::utils::error::{CustomError, CustomErrorInner};
use common_infrastructure::config::EnvironmentService;
use podfetch_domain::invite::InviteRepository;
use podfetch_web::invite::{Invite, InviteApplicationService};
use std::sync::Arc;

#[derive(Clone)]
pub struct InviteService {
    repository: Arc<dyn InviteRepository<Error = CustomError>>,
    environment: Arc<EnvironmentService>,
}

impl InviteService {
    pub fn new(
        repository: Arc<dyn InviteRepository<Error = CustomError>>,
        environment: Arc<EnvironmentService>,
    ) -> Self {
        Self {
            repository,
            environment,
        }
    }

    pub fn create_invite(
        &self,
        role: String,
        explicit_consent: bool,
    ) -> Result<Invite, CustomError> {
        let role = Role::try_from(role)?;
        self.repository
            .create(&role.to_string(), explicit_consent)
            .map(Into::into)
    }

    pub fn find_optional_invite(&self, invite_id: &str) -> Result<Option<Invite>, CustomError> {
        self.repository
            .find_by_id(invite_id)
            .map(|invite| invite.map(Into::into))
    }

    pub fn get_invite(&self, invite_id: &str) -> Result<Invite, CustomError> {
        let invite = self
            .repository
            .find_by_id(invite_id)?
            .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;

        if invite.accepted_at.is_some() {
            return Err(
                CustomErrorInner::Conflict("Invite already accepted".to_string(), Warning).into(),
            );
        }

        Ok(invite.into())
    }

    pub fn get_invites(&self) -> Result<Vec<Invite>, CustomError> {
        self.repository
            .find_all()
            .map(|invites| invites.into_iter().map(Into::into).collect())
    }

    pub fn invalidate_invite(&self, invite_id: &str) -> Result<(), CustomError> {
        self.repository.invalidate(invite_id)
    }

    pub fn get_invite_link(&self, invite_id: &str) -> Result<String, CustomError> {
        let invite = self
            .repository
            .find_by_id(invite_id)?
            .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Error)))?;
        Ok(format!(
            "{}{}{}",
            self.environment.server_url, "ui/invite/", invite.id
        ))
    }

    pub fn delete_invite(&self, invite_id: &str) -> Result<(), CustomError> {
        self.repository
            .find_by_id(invite_id)?
            .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
        self.repository.delete(invite_id)
    }
}

impl InviteApplicationService for InviteService {
    type Error = CustomError;

    fn create_invite(&self, role: String, explicit_consent: bool) -> Result<Invite, Self::Error> {
        self.create_invite(role, explicit_consent)
    }

    fn get_invites(&self) -> Result<Vec<Invite>, Self::Error> {
        self.get_invites()
    }

    fn get_invite(&self, invite_id: &str) -> Result<Invite, Self::Error> {
        self.get_invite(invite_id)
    }

    fn get_invite_link(&self, invite_id: &str) -> Result<String, Self::Error> {
        self.get_invite_link(invite_id)
    }

    fn delete_invite(&self, invite_id: &str) -> Result<(), Self::Error> {
        self.delete_invite(invite_id)
    }
}
