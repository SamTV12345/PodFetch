use crate::role::{Role, STANDARD_USER, STANDARD_USER_ID};
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::error::ErrorSeverity::{Debug, Warning};
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::user::User;
use podfetch_domain::user_admin::{ManagedUser, UserAdminRepository};
use std::sync::Arc;

#[derive(Clone)]
pub struct UserAuthService {
    repository: Arc<dyn UserAdminRepository<Error = CustomError>>,
    environment: Arc<EnvironmentService>,
}

impl UserAuthService {
    pub fn new(
        repository: Arc<dyn UserAdminRepository<Error = CustomError>>,
        environment: Arc<EnvironmentService>,
    ) -> Self {
        Self {
            repository,
            environment,
        }
    }

    pub fn find_by_username(&self, username: &str) -> Result<User, CustomError> {
        if let Some(admin_username) = &self.environment.username
            && admin_username == username
        {
            return Ok(self.configured_admin_user());
        }

        self.repository
            .find_by_username(username)?
            .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))
    }

    pub fn create_user(
        &self,
        username: String,
        role: String,
        password: Option<String>,
        explicit_consent: bool,
    ) -> Result<User, CustomError> {
        if let Some(admin_username) = &self.environment.username
            && admin_username == &username
        {
            return Err(CustomErrorInner::Forbidden(Warning).into());
        }

        self.repository.create(ManagedUser {
            id: 0,
            username,
            role,
            password,
            explicit_consent,
            created_at: chrono::Utc::now().naive_utc(),
            api_key: None,
        })
    }

    pub fn find_by_api_key(&self, api_key: &str) -> Result<Option<User>, CustomError> {
        if api_key.is_empty() {
            return Ok(None);
        }

        if let Some(admin_api_key) = &self.environment.api_key_admin
            && !admin_api_key.is_empty()
            && admin_api_key == api_key
        {
            return Ok(Some(self.read_only_admin_user()));
        }

        self.repository.find_by_api_key(api_key)
    }

    pub fn is_api_key_valid(&self, api_key: &str) -> bool {
        self.find_by_api_key(api_key)
            .map(|user| user.is_some())
            .unwrap_or(false)
    }

    pub fn configured_admin_user(&self) -> User {
        User {
            id: STANDARD_USER_ID,
            username: self
                .environment
                .username
                .clone()
                .unwrap_or_else(|| STANDARD_USER.to_string()),
            role: Role::Admin.to_string(),
            password: self.environment.password.clone(),
            explicit_consent: true,
            created_at: Default::default(),
            api_key: self.environment.api_key_admin.clone(),
        }
    }

    pub fn read_only_admin_user(&self) -> User {
        User {
            id: 9999,
            username: STANDARD_USER.to_string(),
            role: Role::Admin.to_string(),
            password: None,
            explicit_consent: true,
            created_at: Default::default(),
            api_key: self.environment.api_key_admin.clone(),
        }
    }
}
