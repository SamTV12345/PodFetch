//! The `episode_files` migration moves the file state of episodes out of
//! `podcast_episodes` without losing any of it, and its down migration puts
//! everything back. Runs on SQLite, and on Postgres with the `postgresql`
//! feature alone (server on 127.0.0.1:55002, as in CI).

use diesel::sql_types::{Bool, Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl};

const MIGRATION: &str = "2026-11-27-120000";

#[derive(QueryableByName, Debug, PartialEq, Eq)]
struct FileState {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Nullable<Text>)]
    download_time: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    file_episode_path: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    file_image_path: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    download_location: Option<String>,
    #[diesel(sql_type = Bool)]
    deleted: bool,
}

const SEED: &str = "
INSERT INTO podcasts (id, name, directory_id, rssfeed, image_url)
VALUES ('p1', 'Podcast', 'dir', 'https://example.com/feed.xml', 'img');
INSERT INTO podcast_episodes
    (id, podcast_id, episode_id, name, url, date_of_recording, image_url, guid,
     download_time, file_episode_path, file_image_path, download_location, deleted)
VALUES
    ('e1', 'p1', 'f1', 'Downloaded', 'https://example.com/1.mp3', '2026-01-01', 'img', 'g1',
     '2026-01-02 10:00:00', 'podcasts/p/1.mp3', 'podcasts/p/1.jpg', 'Local', FALSE),
    ('e2', 'p1', 'f2', 'Deleted', 'https://example.com/2.mp3', '2026-01-01', 'img', 'g2',
     NULL, NULL, NULL, NULL, TRUE),
    ('e3', 'p1', 'f3', 'Untouched', 'https://example.com/3.mp3', '2026-01-01', 'img', 'g3',
     NULL, NULL, NULL, NULL, FALSE);
";

fn expected() -> Vec<FileState> {
    vec![
        FileState {
            id: "e1".into(),
            download_time: Some("2026-01-02 10:00:00".into()),
            file_episode_path: Some("podcasts/p/1.mp3".into()),
            file_image_path: Some("podcasts/p/1.jpg".into()),
            download_location: Some("Local".into()),
            deleted: false,
        },
        FileState {
            id: "e2".into(),
            download_time: None,
            file_episode_path: None,
            file_image_path: None,
            download_location: None,
            deleted: true,
        },
    ]
}

const AFTER_UP: &str = "
SELECT episode_id AS id, CAST(download_time AS TEXT) AS download_time, file_episode_path,
       file_image_path, download_location, deleted
FROM episode_files ORDER BY episode_id";

const AFTER_DOWN: &str = "
SELECT id, CAST(download_time AS TEXT) AS download_time, file_episode_path,
       file_image_path, download_location, deleted
FROM podcast_episodes ORDER BY id";

/// Runs the scenario on `conn`: migrate up to just before `MIGRATION`,
/// seed, migrate up, check, revert, check, migrate up again.
macro_rules! scenario {
    ($conn:expr, $migrations:expr) => {{
        use diesel::connection::SimpleConnection;
        use diesel_migrations::MigrationHarness;

        let conn = $conn;
        loop {
            let pending = conn.pending_migrations($migrations).unwrap();
            let next = pending.first().expect("our migration is pending");
            if next
                .name()
                .version()
                .to_string()
                .starts_with(&MIGRATION.replace('-', ""))
            {
                break;
            }
            conn.run_next_migration($migrations).unwrap();
        }
        conn.batch_execute(SEED).unwrap();

        conn.run_pending_migrations($migrations).unwrap();
        let moved = diesel::sql_query(AFTER_UP).load::<FileState>(conn).unwrap();
        assert_eq!(moved, expected(), "only episodes with file state get a row");
        assert!(
            conn.batch_execute("SELECT file_episode_path FROM podcast_episodes")
                .is_err(),
            "the columns are gone from podcast_episodes"
        );

        conn.revert_last_migration($migrations).unwrap();
        let restored = diesel::sql_query(AFTER_DOWN)
            .load::<FileState>(conn)
            .unwrap();
        let mut expected_back = expected();
        expected_back.push(FileState {
            id: "e3".into(),
            download_time: None,
            file_episode_path: None,
            file_image_path: None,
            download_location: None,
            deleted: false,
        });
        assert_eq!(
            restored, expected_back,
            "down migration restores every value"
        );

        conn.run_pending_migrations($migrations).unwrap();
        let again = diesel::sql_query(AFTER_UP).load::<FileState>(conn).unwrap();
        assert_eq!(
            again,
            expected(),
            "up again after down gives the same state"
        );
    }};
}

#[cfg(feature = "sqlite")]
#[test]
fn sqlite_moves_file_state_and_back() {
    use diesel::Connection;
    use diesel::connection::SimpleConnection;
    use diesel::sqlite::SqliteConnection;

    let path =
        std::env::temp_dir().join(format!("podfetch_episode_files_{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let mut conn = SqliteConnection::establish(path.to_str().unwrap()).unwrap();
    // As in production: foreign keys on.
    conn.batch_execute("PRAGMA foreign_keys = ON;").unwrap();
    scenario!(&mut conn, podfetch_persistence::db::SQLITE_MIGRATIONS);
    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[cfg(all(feature = "postgresql", not(feature = "sqlite")))]
#[test]
fn postgres_moves_file_state_and_back() {
    use diesel::Connection;
    use diesel::pg::PgConnection;

    const SERVER: &str = "postgres://postgres:postgres@127.0.0.1:55002";
    let name = format!("pf_episode_files_{}", std::process::id());
    let mut admin = PgConnection::establish(&format!("{SERVER}/postgres")).unwrap();
    diesel::sql_query(format!("DROP DATABASE IF EXISTS {name}"))
        .execute(&mut admin)
        .unwrap();
    diesel::sql_query(format!("CREATE DATABASE {name}"))
        .execute(&mut admin)
        .unwrap();
    let mut conn = PgConnection::establish(&format!("{SERVER}/{name}")).unwrap();
    scenario!(&mut conn, podfetch_persistence::db::POSTGRES_MIGRATIONS);
    drop(conn);
    diesel::sql_query(format!("DROP DATABASE {name}"))
        .execute(&mut admin)
        .unwrap();
}
