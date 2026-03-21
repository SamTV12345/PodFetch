use crate::constants::inner_constants::STANDARD_USER_ID;
use crate::models::user::{User, UserWithAPiKey, UserWithoutPassword};
use crate::service::environment_service::EnvironmentService;
use crate::utils::error::CustomError;
use podfetch_domain::user_admin::{ManagedUser, UserAdminRepository};
use podfetch_web::user_admin::UserAdminApplicationService;
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
}

pub fn map_requester(user: &User) -> ManagedUser {
    ManagedUser {
        id: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
        password: user.password.clone(),
        explicit_consent: user.explicit_consent,
        created_at: user.created_at,
        api_key: user.api_key.clone(),
    }
}

impl From<podfetch_domain::user_admin::UserSummary> for UserWithoutPassword {
    fn from(value: podfetch_domain::user_admin::UserSummary) -> Self {
        Self {
            id: value.id,
            username: value.username,
            role: value.role,
            created_at: value.created_at,
            explicit_consent: value.explicit_consent,
        }
    }
}

impl From<podfetch_domain::user_admin::UserWithApiKey> for UserWithAPiKey {
    fn from(value: podfetch_domain::user_admin::UserWithApiKey) -> Self {
        Self {
            id: value.id,
            username: value.username,
            role: value.role,
            created_at: value.created_at,
            explicit_consent: value.explicit_consent,
            api_key: value.api_key,
            read_only: value.read_only,
        }
    }
}

impl UserAdminApplicationService for UserAdminService {
    type Error = CustomError;

    fn find_by_username(&self, username: &str) -> Result<Option<ManagedUser>, Self::Error> {
        self.repository.find_by_username(username)
    }

    fn find_all(&self) -> Result<Vec<ManagedUser>, Self::Error> {
        self.repository.find_all()
    }

    fn update(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        self.repository.update(user)
    }

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error> {
        self.repository.delete_by_username(username)
    }
}
