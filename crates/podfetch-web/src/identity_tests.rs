//! Identity behaviour that needs the whole application (database, other
//! contexts), so it lives beside the test infrastructure of the web crate.

#[cfg(test)]
mod tests {
    use crate::test_support::tests::handle_test_startup;
    use common_infrastructure::config::EnvironmentService;
    use podfetch_domain::filter::{Filter, FilterRepository};
    use podfetch_identity::domain::user_admin::UserAdminRepository;
    use podfetch_identity::persistence::adapters::UserAdminRepositoryImpl;
    use podfetch_identity::role::{STANDARD_USER, STANDARD_USER_ID};
    use podfetch_identity::services::user_auth::service::UserAuthService;
    use podfetch_persistence::adapters::FilterRepositoryImpl;
    use podfetch_persistence::db::database;
    use serial_test::serial;
    use std::sync::Arc;

    /// Build a `UserAuthService` that behaves like a deployment with **no**
    /// configured admin username (the no-auth mode from the bug report), backed
    /// by the real migrated test database.
    fn users() -> UserAdminRepositoryImpl {
        UserAdminRepositoryImpl::new(database())
    }

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
            users().find_by_username(STANDARD_USER).unwrap().is_none(),
            "standard user must be absent before seeding",
        );

        // Reproduces the bug: a write referencing the absent user violates the FK.
        assert!(
            filter_repo.save(standard_filter()).is_err(),
            "FK must reject a reference to a non-existent user",
        );

        // The fix: seed the standard user once.
        service.ensure_standard_user_present().unwrap();

        let seeded = users()
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
            users().find_by_username(STANDARD_USER).unwrap().is_none(),
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
