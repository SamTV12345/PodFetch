//! Identity behaviour that needs the whole application (database, other
//! contexts), so it lives beside the test infrastructure of the web crate.

#[cfg(test)]
mod tests {
    use crate::test_support::tests::handle_test_startup;
    use common_infrastructure::config::EnvironmentService;
    use podfetch_events::{Event, EventBus, EventBusConfig};
    use podfetch_identity::domain::user_admin::UserAdminRepository;
    use podfetch_identity::events::UserDeleted;
    use podfetch_identity::persistence::adapters::UserAdminRepositoryImpl;
    use podfetch_identity::role::{STANDARD_USER, STANDARD_USER_ID};
    use podfetch_identity::services::user_admin::service::UserAdminService;
    use podfetch_identity::services::user_auth::service::UserAuthService;
    use podfetch_library::domain::filter::{Filter, FilterRepository};
    use podfetch_library::persistence::FilterRepositoryImpl;
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

    /// Deleting a user stores `UserDeleted` together with the deletion, so
    /// the other contexts can clean up their data for that user.
    #[tokio::test]
    #[serial]
    async fn deleting_a_user_publishes_user_deleted() {
        let _ts = handle_test_startup().await;

        let environment = Arc::new(EnvironmentService::for_tests());
        let auth = UserAuthService::new(
            Arc::new(UserAdminRepositoryImpl::new(database())),
            environment.clone(),
        );
        let bus = EventBus::new(database(), EventBusConfig::default());
        bus.subscribe::<UserDeleted, _, _>("test.user_deleted", |_, _| async { Ok(()) });
        let admin = UserAdminService::new(
            Arc::new(UserAdminRepositoryImpl::new(database())),
            environment,
            bus.clone(),
        );

        let created = auth
            .create_user("leaving_user".to_string(), "user".to_string(), None, false)
            .unwrap();
        admin.delete_user_by_username("leaving_user").unwrap();
        // Unknown users are not an error and publish nothing.
        admin.delete_user_by_username("never_existed").unwrap();

        assert!(users().find_by_username("leaving_user").unwrap().is_none());
        let published: Vec<_> = bus
            .deliveries(None, 100)
            .unwrap()
            .into_iter()
            .filter(|d| d.event_type == UserDeleted::TYPE)
            .collect();
        assert_eq!(published.len(), 1, "exactly one UserDeleted per deletion");
        assert_eq!(published[0].aggregate_id, created.id.to_string());
        let payload: serde_json::Value = serde_json::from_str(&published[0].payload).unwrap();
        assert_eq!(payload["username"], "leaving_user");
        assert_eq!(payload["user_id"], created.id.to_string());
    }

    /// `authenticate` accepts the right password only and treats unknown
    /// users and users without a local password alike.
    #[tokio::test]
    #[serial]
    async fn authenticate_checks_the_local_password() {
        let _ts = handle_test_startup().await;
        let auth = UserAuthService::new(
            Arc::new(UserAdminRepositoryImpl::new(database())),
            Arc::new(EnvironmentService::for_tests()),
        );
        let created = auth
            .create_user(
                "auth_checked".to_string(),
                "user".to_string(),
                // Stored as the sha256 hash, like the onboarding does.
                Some(sha256::digest("s3cret")),
                false,
            )
            .unwrap();

        assert_eq!(
            auth.authenticate("auth_checked", "s3cret").unwrap().id,
            created.id
        );
        let forbidden = |result: Result<_, common_infrastructure::error::CustomError>| {
            matches!(
                result.err().map(|e| e.inner),
                Some(common_infrastructure::error::CustomErrorInner::Forbidden(_))
            )
        };
        assert!(forbidden(auth.authenticate("auth_checked", "wrong")));
        assert!(forbidden(auth.authenticate("nobody_here", "s3cret")));

        // The repository's update skips `None` fields, so clear it directly.
        {
            use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
            use podfetch_persistence::schema::users::dsl as users;
            diesel::update(users::users.filter(users::username.eq("auth_checked")))
                .set(users::password.eq(None::<String>))
                .execute(&mut podfetch_persistence::db::get_connection())
                .unwrap();
        }
        assert!(forbidden(auth.authenticate("auth_checked", "s3cret")));
    }

    /// `rotate_api_key` replaces the key: the old one no longer finds the user.
    #[tokio::test]
    #[serial]
    async fn rotate_api_key_invalidates_the_old_key() {
        let _ts = handle_test_startup().await;
        let auth = UserAuthService::new(
            Arc::new(UserAdminRepositoryImpl::new(database())),
            Arc::new(EnvironmentService::for_tests()),
        );
        auth.create_user("rotating".to_string(), "user".to_string(), None, false)
            .unwrap();
        let user = auth
            .ensure_api_key(auth.find_by_username("rotating").unwrap())
            .unwrap();
        let old_key = user.api_key.clone().unwrap();

        let rotated = auth.rotate_api_key(user).unwrap();
        let new_key = rotated.api_key.unwrap();

        assert_ne!(new_key, old_key);
        assert!(auth.find_by_api_key(&old_key).unwrap().is_none());
        assert_eq!(
            auth.find_by_api_key(&new_key).unwrap().unwrap().username,
            "rotating"
        );
    }
}
