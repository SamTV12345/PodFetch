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

    #[cfg(feature = "postgresql")]
    static POSTGRES_CONTAINER_INIT: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

    #[cfg(feature = "postgresql")]
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

    #[cfg(feature = "postgresql")]
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

    pub async fn handle_test_startup<'a>() -> TestServerWrapper<'a> {
        let mutex = lock_global_mutex_recovering_poison();

        #[cfg(feature = "postgresql")]
        {
            ensure_shared_postgres_container().await;
            truncate_postgres_tables();
        }

        let mut test_server =
            TestServer::new(crate::startup::handle_config_for_server_startup());
        test_server.add_header("Authorization", "Basic cG9zdGdyZXM6cG9zdGdyZXM=");
        TestServerWrapper { test_server, mutex }
    }

    #[cfg(all(feature = "sqlite", not(feature = "postgresql")))]
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
