use crate::role::STANDARD_USER_ID;
use crate::user_admin::{ManagedUser, UserAdminApplicationService};
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::error::CustomError;
use podfetch_domain::user::{User, UserWithoutPassword};
use podfetch_domain::user_admin::UserAdminRepository;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserAdminService {
    repository: Arc<dyn UserAdminRepository<Error = CustomError>>,
    environment: Arc<EnvironmentService>,
}

impl UserAdminService {
    pub fn new(
        repository: Arc<dyn UserAdminRepository<Error = CustomError>>,
        environment: Arc<EnvironmentService>,
    ) -> Self {
        Self {
            repository,
            environment,
        }
    }

    pub fn read_only_admin_id(&self) -> i32 {
        STANDARD_USER_ID
    }

    pub fn oidc_configured(&self) -> bool {
        self.environment.oidc_configured
    }

    pub fn create_user(&self, user: User) -> Result<User, CustomError> {
        self.repository.create(user.into()).map(Into::into)
    }

    pub fn find_user_by_username(&self, username: &str) -> Result<Option<User>, CustomError> {
        self.repository
            .find_by_username(username)
            .map(|user| user.map(Into::into))
    }

    pub fn list_users(&self) -> Result<Vec<UserWithoutPassword>, CustomError> {
        self.repository
            .find_all()
            .map(|users| users.into_iter().map(|user| user.to_summary()).collect())
    }

    pub fn update_user(&self, user: User) -> Result<User, CustomError> {
        self.repository.update(user.into()).map(Into::into)
    }

    pub fn delete_user_by_username(&self, username: &str) -> Result<(), CustomError> {
        self.repository.delete_by_username(username)
    }
}

pub fn map_requester(user: &User) -> ManagedUser {
    user.clone().into()
}

impl UserAdminApplicationService for UserAdminService {
    type Error = CustomError;

    fn find_by_username(&self, username: &str) -> Result<Option<ManagedUser>, Self::Error> {
        self.repository
            .find_by_username(username)
            .map(|user| user.map(Into::into))
    }

    fn find_all(&self) -> Result<Vec<ManagedUser>, Self::Error> {
        self.repository
            .find_all()
            .map(|users| users.into_iter().map(Into::into).collect())
    }

    fn update(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        self.repository.update(user.into()).map(Into::into)
    }

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error> {
        self.repository.delete_by_username(username)
    }
}
