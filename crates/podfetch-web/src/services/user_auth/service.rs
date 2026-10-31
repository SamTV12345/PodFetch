use crate::role::{Role, STANDARD_USER, STANDARD_USER_ID};
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::error::ErrorSeverity::{Debug, Warning};
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::user::User;
use podfetch_domain::user_admin::{ManagedUser, UserAdminRepository};
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
            id: podfetch_domain::ids::new_id(),
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
            id: podfetch_domain::ids::new_id(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::tests::handle_test_startup;
    use common_infrastructure::config::EnvironmentService;
    use podfetch_domain::filter::{Filter, FilterRepository};
    use podfetch_persistence::adapters::{FilterRepositoryImpl, UserAdminRepositoryImpl};
    use podfetch_persistence::db::database;
    use serial_test::serial;

    /// Build a `UserAuthService` that behaves like a deployment with **no**
    /// configured admin username (the no-auth mode from the bug report), backed
    /// by the real migrated test database.
    fn no_auth_service() -> UserAuthService {
        let mut environment = EnvironmentService::for_tests();
        environment.username = None;
        UserAuthService::new(
            Arc::new(UserAdminRepositoryImpl::new(database())),
            Arc::new(environment),
        )
    }

    /// Regression test for "Unable to add new podcast feed" (#2109).
    ///
    /// In no-auth mode every request is attributed to the standard user
    /// (id [`STANDARD_USER_ID`]), which historically was never written to the
    /// `users` table. Any write keyed on that id then failed the
    /// `REFERENCES users(id)` foreign key. Seeding the row keeps the FK intact
    /// while making the reference valid.
    #[tokio::test]
    #[serial]
    async fn standard_user_is_seeded_so_fk_bound_writes_succeed() {
        // Migrates the DB, enables `PRAGMA foreign_keys = ON`, and starts clean.
        let _ts = handle_test_startup().await;

        let service = no_auth_service();
        let filter_repo = FilterRepositoryImpl::new(database());
        let standard_filter = || Filter::new(STANDARD_USER_ID, None, true, None, false);

        // The requester in no-auth mode is the standard user...
        assert_eq!(service.read_only_admin_user().id, STANDARD_USER_ID);
        // ...but it does not yet exist in the database.
        assert!(
            service
                .repository
                .find_by_username(STANDARD_USER)
                .unwrap()
                .is_none(),
            "standard user must be absent before seeding",
        );

        // Reproduces the bug: a write referencing the absent user violates the FK.
        assert!(
            filter_repo.save(standard_filter()).is_err(),
            "FK must reject a reference to a non-existent user",
        );

        // The fix: seed the standard user once.
        service.ensure_standard_user_present().unwrap();

        let seeded = service
            .repository
            .find_by_username(STANDARD_USER)
            .unwrap()
            .expect("standard user must exist after seeding");
        assert_eq!(
            seeded.id, STANDARD_USER_ID,
            "seed must preserve the fixed id"
        );

        // Idempotent: a second startup must not error or duplicate.
        service.ensure_standard_user_present().unwrap();

        // The previously-failing FK-bound write now succeeds.
        filter_repo
            .save(standard_filter())
            .expect("filter save must succeed once the standard user exists");
    }

    /// When a named admin username is configured the requester is a real
    /// database user, so the standard user must not be seeded.
    #[tokio::test]
    #[serial]
    async fn standard_user_is_not_seeded_when_admin_username_configured() {
        let _ts = handle_test_startup().await;

        let service = UserAuthService::new(
            Arc::new(UserAdminRepositoryImpl::new(database())),
            Arc::new(EnvironmentService::for_tests()),
        );

        service.ensure_standard_user_present().unwrap();

        assert!(
            service
                .repository
                .find_by_username(STANDARD_USER)
                .unwrap()
                .is_none(),
            "standard user must not be seeded when a named admin is configured",
        );
    }

    /// OIDC onboarding creates users without an api_key, but the UI needs one
    /// to open its socket.io connection (#2282).
    #[tokio::test]
    #[serial]
    async fn ensure_api_key_generates_once_and_persists() {
        let _ts = handle_test_startup().await;

        let service = UserAuthService::new(
            Arc::new(UserAdminRepositoryImpl::new(database())),
            Arc::new(EnvironmentService::for_tests()),
        );

        let created = service
            .create_user("oidc_user".to_string(), "user".to_string(), None, false)
            .unwrap();
        assert!(created.api_key.is_none());

        let with_key = service.ensure_api_key(created).unwrap();
        let api_key = with_key.api_key.clone().expect("api key must be set");
        assert_eq!(api_key.len(), 32);
        assert!(!api_key.contains('-'));

        // Persisted, and a second call keeps the existing key.
        let stored = service.find_by_username("oidc_user").unwrap();
        assert_eq!(stored.api_key.as_deref(), Some(api_key.as_str()));
        let again = service.ensure_api_key(stored).unwrap();
        assert_eq!(again.api_key.as_deref(), Some(api_key.as_str()));
    }
}
