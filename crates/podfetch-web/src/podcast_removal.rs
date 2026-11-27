//! Removing a podcast from the catalog.
//!
//! The foreign keys onto `podcasts`/`podcast_episodes` stay in the
//! monolith, but several of them don't cascade, and no context may rely on
//! the ones that do. So every context first removes what it keeps about the
//! podcast and its episodes through its own API ([`clear_references`]);
//! then the catalog deletes the podcast and announces it with
//! `podcast.removed`. Each step is idempotent: a failed removal can simply
//! be retried.

use crate::app_state::AppState;
use crate::services::podcast::service::PodcastService;
use crate::services::transcript::service::TranscriptService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase;
use crate::usecases::watchtime::WatchtimeUseCase as WatchtimeService;
use common_infrastructure::error::CustomError;
use podfetch_catalog::domain::chapter::PodcastEpisodeChapterRepository;
use podfetch_catalog::persistence::chapter::PodcastEpisodeChapterRepositoryImpl;
use podfetch_domain::podcast_settings::PodcastSettingsRepository;
use podfetch_library::services::podcast_removal::LibraryPodcastRemoval;
use podfetch_persistence::adapters::PodcastSettingsRepositoryImpl;
use podfetch_persistence::db::database;
use uuid::Uuid;

/// Removes everything the other contexts keep about the podcast and its
/// episodes: library (favorites, tags, playlist entries, triage),
/// listening (events, gPodder episode actions), transcripts and the media
/// library's records (file state, SponsorBlock). Files on disk are not
/// touched here.
pub fn clear_references(
    state: &AppState,
    podcast_id: Uuid,
    episode_ids: &[Uuid],
) -> Result<(), CustomError> {
    LibraryPodcastRemoval::new(database()).forget_podcast(podcast_id, episode_ids)?;
    WatchtimeService::delete_watchtime(podcast_id)?;
    TranscriptService::default_service().forget_episodes(episode_ids)?;
    let episode_ids: Vec<String> = episode_ids.iter().map(Uuid::to_string).collect();
    state.media.files.forget(&episode_ids)?;
    state.media.sponsorblock.forget_episodes(&episode_ids)?;
    Ok(())
}

/// Removes the podcast: first the other contexts' data, then the catalog's
/// own (chapters, podcast settings, episodes, the podcast), then announces
/// `podcast.removed`.
pub fn remove_podcast(state: &AppState, podcast_id: Uuid) -> Result<(), CustomError> {
    let podcast = PodcastService::get_podcast(podcast_id)?;
    let episode_ids = PodcastEpisodeUseCase::get_episodes_by_podcast_id(podcast_id)?
        .iter()
        .map(|episode| {
            Uuid::parse_str(&episode.id).map_err(|_| {
                CustomError::from(common_infrastructure::error::CustomErrorInner::NotFound(
                    common_infrastructure::error::ErrorSeverity::Warning,
                ))
            })
        })
        .collect::<Result<Vec<Uuid>, CustomError>>()?;

    clear_references(state, podcast_id, &episode_ids)?;

    PodcastEpisodeChapterRepositoryImpl::new(database()).delete_by_episode_ids(&episode_ids)?;
    PodcastSettingsRepositoryImpl::new(database()).delete_settings(podcast_id)?;
    PodcastEpisodeUseCase::delete_episodes_of_podcast(podcast_id)?;
    PodcastService::delete_podcast(podcast_id)?;

    crate::podcast_events::publish(&podfetch_catalog::events::PodcastRemoved {
        podcast_id: podcast.id,
        podcast_name: podcast.name,
        rssfeed: podcast.rssfeed,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::tests::{app_state, handle_test_startup};
    use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
    use diesel::RunQueryDsl;
    use diesel::sql_types::{BigInt, Text};
    use podfetch_persistence::db::get_connection;
    use serial_test::serial;

    /// Tables that reference episodes without a foreign key.
    const WITHOUT_FOREIGN_KEY: &[(&str, &str)] = &[("episode_triages", "episode_id")];

    /// Tables the catalog itself deletes from after the other contexts.
    const CATALOG_TABLES: &[&str] = &[
        "podcast_episodes",
        "podcast_episode_chapters",
        "podcast_settings",
    ];

    #[derive(diesel::QueryableByName)]
    struct Reference {
        #[diesel(sql_type = Text)]
        table_name: String,
        #[diesel(sql_type = Text)]
        column_name: String,
        #[diesel(sql_type = Text)]
        target: String,
    }

    #[derive(diesel::QueryableByName)]
    struct Count {
        #[diesel(sql_type = BigInt)]
        n: i64,
    }

    fn run(sql: &str) {
        diesel::sql_query(sql)
            .execute(&mut get_connection())
            .unwrap_or_else(|e| panic!("{sql}: {e}"));
    }

    fn count(sql: &str) -> i64 {
        diesel::sql_query(sql)
            .get_result::<Count>(&mut get_connection())
            .unwrap_or_else(|e| panic!("{sql}: {e}"))
            .n
    }

    /// Every column that references `podcasts` or `podcast_episodes`, read
    /// from the schema: a new table with such a foreign key is covered
    /// without touching this test.
    fn references() -> Vec<(String, String, String)> {
        #[cfg(feature = "sqlite")]
        const SQLITE: &str = "SELECT m.name AS table_name, f.\"from\" AS column_name, \
             f.\"table\" AS target \
             FROM sqlite_master m, pragma_foreign_key_list(m.name) f \
             WHERE m.type = 'table' AND f.\"table\" IN ('podcasts', 'podcast_episodes')";
        #[cfg(feature = "postgresql")]
        const POSTGRES: &str = "SELECT tc.table_name::text AS table_name, \
             kcu.column_name::text AS column_name, ccu.table_name::text AS target \
             FROM information_schema.table_constraints tc \
             JOIN information_schema.key_column_usage kcu \
               ON tc.constraint_name = kcu.constraint_name \
             JOIN information_schema.constraint_column_usage ccu \
               ON tc.constraint_name = ccu.constraint_name \
             WHERE tc.constraint_type = 'FOREIGN KEY' \
               AND ccu.table_name IN ('podcasts', 'podcast_episodes')";
        // The backend in use, not the compiled features: builds with both
        // features run against whichever database is configured.
        let mut conn = get_connection();
        let sql = match &mut *conn {
            #[cfg(feature = "sqlite")]
            common_infrastructure::db::DBType::Sqlite(_) => SQLITE,
            #[cfg(feature = "postgresql")]
            common_infrastructure::db::DBType::Postgresql(_) => POSTGRES,
        };
        let mut found: Vec<(String, String, String)> = diesel::sql_query(sql)
            .load::<Reference>(&mut conn)
            .unwrap()
            .into_iter()
            .map(|r| (r.table_name, r.column_name, r.target))
            .collect();
        found.extend(
            WITHOUT_FOREIGN_KEY
                .iter()
                .map(|(t, c)| (t.to_string(), c.to_string(), "podcast_episodes".to_string())),
        );
        found.sort();
        found.dedup();
        found
    }

    /// Rows that still point at the podcast or its episodes, per table.
    fn remaining(podcast_id: &str, episode_ids: &[String]) -> Vec<(String, i64)> {
        let episodes = episode_ids
            .iter()
            .map(|id| format!("'{id}'"))
            .collect::<Vec<_>>()
            .join(",");
        references()
            .into_iter()
            .map(|(table, column, target)| {
                let ids = if target == "podcasts" {
                    format!("'{podcast_id}'")
                } else {
                    episodes.clone()
                };
                let n = count(&format!(
                    "SELECT COUNT(*) AS n FROM {table} WHERE {column} IN ({ids})"
                ));
                (format!("{table}.{column}"), n)
            })
            .collect()
    }

    /// One row in every table that references the podcast or its episodes.
    fn seed(user_id: &str) -> (String, Vec<String>) {
        let podcast_id = PodcastService::add_podcast_to_database(
            "Removal",
            "dir-removal",
            "https://example.com/removal.xml",
            "http://localhost:8080/ui/default.jpg",
            "removal",
        )
        .unwrap()
        .id;
        let episodes: Vec<String> = (0..2).map(|_| Uuid::now_v7().to_string()).collect();
        for (i, id) in episodes.iter().enumerate() {
            run(&format!(
                "INSERT INTO podcast_episodes (id, podcast_id, episode_id, name, url, \
                 date_of_recording, image_url, total_time, description, guid, \
                 episode_numbering_processed) VALUES ('{id}', '{podcast_id}', 'removal-{i}', \
                 'Episode {i}', 'https://example.com/{i}.mp3', '2026-05-0{}T00:00:00Z', \
                 'http://localhost:8080/ui/default.jpg', 60, 'd', 'guid-removal-{i}', FALSE)",
                i + 1
            ));
        }
        let e = &episodes[0];
        let unique = |name: &str| format!("{name}-{}", Uuid::now_v7());
        let (tag_removal, pl_removal, ch_removal, sb_removal) =
            (unique("tag"), unique("pl"), unique("ch"), unique("sb"));
        let (tr_removal, seg_removal, job_removal, le_removal) =
            (unique("tr"), unique("seg"), unique("job"), unique("le"));
        let now = "'2026-05-01 00:00:00'";
        for sql in [
            format!(
                "INSERT INTO favorites (user_id, podcast_id, favored) VALUES ('{user_id}', '{podcast_id}', TRUE)"
            ),
            format!(
                "INSERT INTO favorite_podcast_episodes (user_id, episode_id, favorite) VALUES ('{user_id}', '{e}', TRUE)"
            ),
            format!(
                "INSERT INTO tags (id, name, user_id, description, created_at, color) VALUES ('{tag_removal}', '{tag_removal}', '{user_id}', NULL, {now}, 'blue')"
            ),
            format!(
                "INSERT INTO tags_podcasts (tag_id, podcast_id) VALUES ('{tag_removal}', '{podcast_id}')"
            ),
            format!(
                "INSERT INTO playlists (id, name, user_id) VALUES ('{pl_removal}', 'removal', '{user_id}')"
            ),
            format!(
                "INSERT INTO playlist_items (playlist_id, episode, position) VALUES ('{pl_removal}', '{e}', 0)"
            ),
            format!(
                "INSERT INTO episode_triages (user_id, episode_id, status, updated_at) VALUES ('{user_id}', '{e}', 'queued', {now})"
            ),
            format!(
                "INSERT INTO podcast_episode_chapters (id, episode_id, title, start_time, end_time, created_at, updated_at) VALUES ('{ch_removal}', '{e}', 'Intro', 0, 10, {now}, {now})"
            ),
            format!(
                "INSERT INTO episode_sponsor_segments (id, episode_id, uuid, category, action_type, start_ms, end_ms, votes, locked, duration_mismatch, fetched_at) VALUES ('{sb_removal}', '{e}', 'u', 'sponsor', 'skip', 0, 1000, 0, FALSE, FALSE, {now})"
            ),
            format!(
                "INSERT INTO podcast_episode_transcripts (id, episode_id, source, mime_type, is_preferred, status, created_at, updated_at) VALUES ('{tr_removal}', '{e}', 'feed', 'text/vtt', TRUE, 'parsed', {now}, {now})"
            ),
            format!(
                "INSERT INTO podcast_episode_transcript_segments (id, transcript_id, idx, text) VALUES ('{seg_removal}', '{tr_removal}', 0, 'hello')"
            ),
            format!(
                "INSERT INTO transcription_jobs (id, episode_id, status, attempts, created_at, updated_at) VALUES ('{job_removal}', '{e}', 'pending', 0, {now}, {now})"
            ),
            format!(
                "INSERT INTO listening_events (id, user_id, device, podcast_episode_id, podcast_id, podcast_episode_db_id, delta_seconds, start_position, end_position, listened_at) VALUES ('{le_removal}', '{user_id}', 'web', 'removal-0', '{podcast_id}', '{e}', 10, 0, 10, {now})"
            ),
            format!("INSERT INTO podcast_settings (podcast_id) VALUES ('{podcast_id}')"),
            format!(
                "INSERT INTO episode_files (episode_id, file_episode_path, deleted) VALUES ('{e}', 'removal/0.mp3', FALSE)"
            ),
        ] {
            run(&sql);
        }
        (podcast_id, episodes)
    }

    fn admin_id() -> String {
        let username = ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "postgres".to_string());
        app_state()
            .user_auth_service
            .find_by_username(&username)
            .unwrap()
            .id
            .to_string()
    }

    /// The other contexts clear their rows while the catalog's rows still
    /// exist, so no `ON DELETE CASCADE` can have done it for them.
    #[tokio::test]
    #[serial]
    async fn every_context_clears_its_references_itself() {
        let _ts = handle_test_startup().await;
        let state = app_state();
        let (podcast_id, episodes) = seed(&admin_id());
        let unseeded: Vec<String> = remaining(&podcast_id, &episodes)
            .into_iter()
            .filter(|(_, n)| *n == 0)
            .map(|(column, _)| column)
            .collect();
        assert!(
            unseeded.is_empty(),
            "seed a row for {unseeded:?} and clear it in podcast_removal"
        );
        let episode_uuids: Vec<Uuid> = episodes
            .iter()
            .map(|e| Uuid::parse_str(e).unwrap())
            .collect();

        clear_references(
            &state,
            Uuid::parse_str(&podcast_id).unwrap(),
            &episode_uuids,
        )
        .unwrap();

        let left: Vec<(String, i64)> = remaining(&podcast_id, &episodes)
            .into_iter()
            .filter(|(column, n)| {
                *n > 0 && !CATALOG_TABLES.contains(&column.split('.').next().unwrap())
            })
            .collect();
        assert!(left.is_empty(), "still referenced: {left:?}");
        assert_eq!(
            count(&format!(
                "SELECT COUNT(*) AS n FROM podcast_episode_transcript_segments s \
                 JOIN podcast_episode_transcripts t ON t.id = s.transcript_id \
                 WHERE t.episode_id IN ('{}', '{}')",
                episodes[0], episodes[1]
            )),
            0
        );
        assert_eq!(
            count(&format!(
                "SELECT COUNT(*) AS n FROM podcasts WHERE id = '{podcast_id}'"
            )),
            1
        );
    }

    #[tokio::test]
    #[serial]
    async fn removing_a_podcast_leaves_nothing_behind_and_is_announced() {
        let _ts = handle_test_startup().await;
        let state = app_state();
        let (podcast_id, episodes) = seed(&admin_id());

        remove_podcast(&state, Uuid::parse_str(&podcast_id).unwrap()).unwrap();

        let left: Vec<(String, i64)> = remaining(&podcast_id, &episodes)
            .into_iter()
            .filter(|(_, n)| *n > 0)
            .collect();
        assert!(left.is_empty(), "still referenced: {left:?}");
        assert_eq!(
            count(&format!(
                "SELECT COUNT(*) AS n FROM podcasts WHERE id = '{podcast_id}'"
            )),
            0
        );
        assert_eq!(
            count(&format!(
                "SELECT COUNT(*) AS n FROM events WHERE event_type = 'podcast.removed' AND aggregate_id = '{podcast_id}'"
            )),
            1
        );
    }
}
