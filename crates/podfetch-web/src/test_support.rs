/// Integration test infrastructure.
#[cfg(test)]
pub mod tests {
    use axum_test::TestServer;
    use std::sync::MutexGuard;

    pub struct TestServerWrapper<'a> {
        pub test_server: TestServer,
        pub mutex: MutexGuard<'a, ()>,
    }

    pub static GLOBAL_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn lock_global_mutex_recovering_poison<'a>() -> MutexGuard<'a, ()> {
        match GLOBAL_MUTEX.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("GLOBAL_MUTEX is poisoned, recovering lock for continued test execution");
                poisoned.into_inner()
            }
        }
    }

    #[cfg(all(feature = "postgresql", not(feature = "sqlite")))]
    static POSTGRES_CONTAINER_INIT: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

    #[cfg(all(feature = "postgresql", not(feature = "sqlite")))]
    async fn ensure_shared_postgres_container() {
        use testcontainers::runners::AsyncRunner;
        use testcontainers::{ContainerRequest, ImageExt};
        use testcontainers_modules::postgres::Postgres;

        fn setup_container() -> ContainerRequest<Postgres> {
            Postgres::default().with_mapped_port(55002, 5432.into())
        }

        if tokio::net::TcpStream::connect("127.0.0.1:55002")
            .await
            .is_ok()
        {
            return;
        }

        POSTGRES_CONTAINER_INIT
            .get_or_init(|| async {
                match AsyncRunner::start(setup_container()).await {
                    Ok(container) => {
                        std::mem::forget(container);
                    }
                    Err(error) => {
                        if tokio::net::TcpStream::connect("127.0.0.1:55002")
                            .await
                            .is_ok()
                        {
                            return;
                        }
                        panic!("Could not start shared postgres test container: {error}");
                    }
                }
            })
            .await;
    }

    #[cfg(all(feature = "postgresql", not(feature = "sqlite")))]
    fn truncate_postgres_tables() {
        use diesel::{RunQueryDsl, sql_query};
        use podfetch_persistence::db::get_connection;

        sql_query(
            r#"
DO $$
DECLARE
    truncate_stmt text;
BEGIN
    SELECT
        'TRUNCATE TABLE '
        || string_agg(format('%I.%I', schemaname, tablename), ', ')
        || ' RESTART IDENTITY CASCADE'
    INTO truncate_stmt
    FROM pg_tables
    WHERE schemaname = 'public'
      AND tablename <> '__diesel_schema_migrations';

    IF truncate_stmt IS NOT NULL THEN
        EXECUTE truncate_stmt;
    END IF;
END $$;
"#,
        )
        .execute(&mut get_connection())
        .expect("Could not truncate postgres tables for test setup");
    }

    #[cfg(feature = "sqlite")]
    fn cleanup_sqlite() {
        use diesel::RunQueryDsl;
        use podfetch_persistence::db::get_connection;

        // Order matters: delete children before parents (foreign keys)
        let tables: &[&str] = &[
            "listening_events",
            "playlist_items",
            "favorite_podcast_episodes",
            "subscriptions",
            "episodes",
            "podcast_episodes",
            "podcast_episode_chapters",
            "tags_podcasts",
            "tags",
            "invites",
            "sessions",
            "settings",
            "podcast_settings",
            "podcasts",
            "notifications",
            "devices",
            "users",
        ];

        let mut conn = get_connection();
        for table in tables {
            let _ = diesel::sql_query(format!("DELETE FROM {table}"))
                .execute(&mut conn);
        }
    }

    /// Set environment variables so that `EnvironmentService::new()` produces
    /// a test-friendly configuration.  `common-infrastructure` is compiled
    /// **without** `cfg(test)` when used as a dependency, so its
    /// `ENVIRONMENT_SERVICE` calls `new()` instead of `for_tests()`.
    /// We bridge that gap by injecting the right env vars before the
    /// `LazyLock` is first accessed.
    ///
    /// Call this before any access to `ENVIRONMENT_SERVICE` — including in
    /// tests that do NOT use `handle_test_startup`.
    pub fn ensure_test_env_vars() {
        use std::sync::Once;
        static INIT: Once = Once::new();
        // SAFETY: This runs exactly once (via `Once`) before any other test
        // code accesses the environment, and tests are serialised by
        // GLOBAL_MUTEX, so there are no concurrent readers.
        INIT.call_once(|| unsafe {
            // Enable GPodder routes in tests
            std::env::set_var("GPODDER_INTEGRATION_ENABLED", "true");
            // Enable basic auth so GPodder tests can authenticate
            std::env::set_var("BASIC_AUTH", "true");
            std::env::set_var("USERNAME", "postgres");
            // EnvironmentService::new() applies sha256::digest to the raw
            // value, so we pass the plaintext here.
            std::env::set_var("PASSWORD", "postgres");
            std::env::set_var("SERVER_URL", "http://localhost:8000/");
            std::env::set_var("API_KEY", "test-api-key");
            #[cfg(feature = "sqlite")]
            std::env::set_var("DATABASE_URL", "sqlite://./podcast.db");
            #[cfg(all(feature = "postgresql", not(feature = "sqlite")))]
            std::env::set_var(
                "DATABASE_URL",
                "postgres://postgres:postgres@127.0.0.1:55002/postgres",
            );
        });
    }

    pub async fn handle_test_startup<'a>() -> TestServerWrapper<'a> {
        ensure_test_env_vars();
        let mutex = lock_global_mutex_recovering_poison();

        #[cfg(all(feature = "postgresql", not(feature = "sqlite")))]
        {
            ensure_shared_postgres_container().await;
            truncate_postgres_tables();
        }

        // Clean DB BEFORE building the router so insert_default_settings
        // starts with a fresh database every time.
        #[cfg(feature = "sqlite")]
        cleanup_sqlite();

        let mut test_server =
            TestServer::new(crate::startup::build_server_router());
        test_server.add_header("Authorization", "Basic cG9zdGdyZXM6cG9zdGdyZXM=");
        TestServerWrapper { test_server, mutex }
    }

    #[cfg(feature = "sqlite")]
    impl Drop for TestServerWrapper<'_> {
        fn drop(&mut self) {
            use diesel::RunQueryDsl;
            use podfetch_persistence::db::get_connection;
            {
                use podfetch_persistence::schema::listening_events::dsl::listening_events;
                diesel::delete(listening_events)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::playlist_items::dsl::playlist_items;
                diesel::delete(playlist_items)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::favorite_podcast_episodes::dsl::favorite_podcast_episodes;
                diesel::delete(favorite_podcast_episodes)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::subscriptions::dsl::subscriptions;
                diesel::delete(subscriptions)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::episodes::dsl::episodes;
                diesel::delete(episodes)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::podcast_episodes::dsl::podcast_episodes;
                diesel::delete(podcast_episodes)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::tags_podcasts::dsl::tags_podcasts;
                diesel::delete(tags_podcasts)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::tags::dsl::tags;
                diesel::delete(tags).execute(&mut get_connection()).unwrap();
            }
            {
                use podfetch_persistence::schema::invites::dsl::invites;
                diesel::delete(invites)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::settings::dsl::settings;
                diesel::delete(settings)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::podcast_settings::dsl::podcast_settings;
                diesel::delete(podcast_settings)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::podcasts::dsl::podcasts;
                diesel::delete(podcasts)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::notifications::dsl::notifications;
                diesel::delete(notifications)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::devices::dsl::devices;
                diesel::delete(devices)
                    .execute(&mut get_connection())
                    .unwrap();
            }
            {
                use podfetch_persistence::schema::users::dsl::users;
                diesel::delete(users)
                    .execute(&mut get_connection())
                    .unwrap();
            }
        }
    }
}
