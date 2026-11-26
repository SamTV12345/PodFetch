use crate::domain::user::User;
use crate::domain::user_admin::{ManagedUser, UserAdminRepository};
use crate::role::{Role, STANDARD_USER, STANDARD_USER_ID};
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::error::ErrorSeverity::{Debug, Warning};
use common_infrastructure::error::{CustomError, CustomErrorInner};
use std::sync::Arc;
use uuid::Uuid;

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
            return self.ensure_admin_user();
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
            id: uuid::Uuid::now_v7(),
            username,
            role,
            password,
            explicit_consent,
            created_at: chrono::Utc::now().naive_utc(),
            api_key: None,
            country: None,
            language: None,
        })
    }

    /// Generates and persists an api_key for users that don't have one yet.
    /// Users that already have a key are returned untouched, without a write.
    pub fn ensure_api_key(&self, mut user: User) -> Result<User, CustomError> {
        if user.api_key.as_deref().is_some_and(|k| !k.is_empty()) {
            return Ok(user);
        }
        user.api_key = Some(Uuid::new_v4().simple().to_string());
        self.repository.update(user)
    }

    pub fn find_by_api_key(&self, api_key: &str) -> Result<Option<User>, CustomError> {
        if api_key.is_empty() {
            return Ok(None);
        }

        if let Some(admin_api_key) = &self.environment.api_key_admin
            && !admin_api_key.is_empty()
            && admin_api_key == api_key
        {
            return self.ensure_admin_user().map(Some);
        }

        self.repository.find_by_api_key(api_key)
    }

    pub fn is_api_key_valid(&self, api_key: &str) -> bool {
        self.find_by_api_key(api_key)
            .is_ok_and(|user| user.is_some())
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
            country: None,
            language: None,
        }
    }

    pub fn ensure_admin_user(&self) -> Result<User, CustomError> {
        let Some(admin_username) = &self.environment.username else {
            return Ok(self.read_only_admin_user());
        };

        if let Some(user) = self.repository.find_by_username(admin_username)? {
            return Ok(user);
        }

        self.repository.create(ManagedUser {
            id: uuid::Uuid::now_v7(),
            username: admin_username.clone(),
            role: Role::Admin.to_string(),
            password: self.environment.password.clone(),
            explicit_consent: true,
            created_at: chrono::Utc::now().naive_utc(),
            api_key: self.environment.api_key_admin.clone(),
            country: None,
            language: None,
        })
    }

    /// Persist the synthetic standard user so foreign keys that reference it
    /// resolve to a real row.
    ///
    /// When no named admin `USERNAME` is configured, every request is attributed
    /// to the in-memory [`Self::read_only_admin_user`] (id [`STANDARD_USER_ID`]).
    /// That user is never written to the `users` table, so any write keyed on it
    /// (`filters.user_id`, `podcasts.added_by`, …) fails the
    /// `REFERENCES users(id)` constraint. Seeding the row once at startup keeps
    /// the constraint intact while making the reference valid.
    ///
    /// No-op when a named admin username is configured, since the requester is
    /// then a real database user rather than the standard user.
    pub fn ensure_standard_user_present(&self) -> Result<(), CustomError> {
        if self.environment.username.is_some() {
            return Ok(());
        }

        self.repository.ensure_with_id(ManagedUser {
            id: STANDARD_USER_ID,
            username: STANDARD_USER.to_string(),
            role: Role::Admin.to_string(),
            password: None,
            explicit_consent: true,
            created_at: chrono::Utc::now().naive_utc(),
            api_key: self.environment.api_key_admin.clone(),
            country: None,
            language: None,
        })?;
        Ok(())
    }

    pub fn read_only_admin_user(&self) -> User {
        User {
            id: STANDARD_USER_ID,
            username: STANDARD_USER.to_string(),
            role: Role::Admin.to_string(),
            password: None,
            explicit_consent: true,
            created_at: Default::default(),
            api_key: self.environment.api_key_admin.clone(),
            country: None,
            language: None,
        }
    }
}
