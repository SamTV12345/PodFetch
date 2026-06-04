use crate::db::get_connection;
use crate::schema::{episode_sponsor_segments, sponsorblock_user_settings};
use chrono::NaiveDateTime;
use common_infrastructure::db::PersistenceError;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug, Clone, PartialEq)]
#[diesel(table_name = episode_sponsor_segments)]
pub struct SponsorSegmentEntity {
    pub id: String,
    pub episode_id: String,
    pub uuid: String,
    pub category: String,
    pub action_type: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub votes: i32,
    pub locked: bool,
    pub duration_mismatch: bool,
    pub fetched_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Debug, Clone, PartialEq)]
#[diesel(table_name = sponsorblock_user_settings)]
#[diesel(primary_key(user_id))]
pub struct SponsorblockUserSettingsEntity {
    pub user_id: String,
    pub enabled: bool,
    pub skip_sponsor: bool,
    pub skip_selfpromo: bool,
    pub skip_interaction: bool,
    pub skip_intro: bool,
    pub skip_outro: bool,
    pub skip_preview: bool,
    pub skip_filler: bool,
    pub skip_music_offtopic: bool,
}

pub struct SponsorblockRepository;

impl SponsorblockRepository {
    /// Idempotently replace ALL stored segments for an episode with `segments`.
    pub fn replace_segments_for_episode(
        episode_id_value: &str,
        segments: Vec<SponsorSegmentEntity>,
    ) -> Result<(), PersistenceError> {
        use self::episode_sponsor_segments::dsl as s;
        let mut conn = get_connection();
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(s::episode_sponsor_segments.filter(s::episode_id.eq(episode_id_value)))
                .execute(conn)?;
            for segment in &segments {
                diesel::insert_into(s::episode_sponsor_segments)
                    .values(segment)
                    .execute(conn)?;
            }
            Ok(())
        })
        .map_err(Into::into)
    }

    pub fn get_segments_for_episode(
        episode_id_value: &str,
    ) -> Result<Vec<SponsorSegmentEntity>, PersistenceError> {
        use self::episode_sponsor_segments::dsl as s;
        s::episode_sponsor_segments
            .filter(s::episode_id.eq(episode_id_value))
            .order(s::start_ms.asc())
            .load::<SponsorSegmentEntity>(&mut get_connection())
            .map_err(Into::into)
    }

    pub fn get_user_settings(
        user_id_value: &str,
    ) -> Result<Option<SponsorblockUserSettingsEntity>, PersistenceError> {
        use self::sponsorblock_user_settings::dsl as u;
        u::sponsorblock_user_settings
            .filter(u::user_id.eq(user_id_value))
            .first::<SponsorblockUserSettingsEntity>(&mut get_connection())
            .optional()
            .map_err(Into::into)
    }

    pub fn upsert_user_settings(
        settings: SponsorblockUserSettingsEntity,
    ) -> Result<(), PersistenceError> {
        use self::sponsorblock_user_settings::dsl as u;
        let mut conn = get_connection();
        let existing: Option<SponsorblockUserSettingsEntity> = u::sponsorblock_user_settings
            .filter(u::user_id.eq(&settings.user_id))
            .first::<SponsorblockUserSettingsEntity>(&mut conn)
            .optional()
            .map_err(PersistenceError::from)?;
        match existing {
            Some(_) => {
                diesel::update(
                    u::sponsorblock_user_settings.filter(u::user_id.eq(&settings.user_id)),
                )
                .set(&settings)
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
            }
            None => {
                diesel::insert_into(u::sponsorblock_user_settings)
                    .values(&settings)
                    .execute(&mut conn)
                    .map_err(PersistenceError::from)?;
            }
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod sponsorblock_tests {
    use super::*;
    use crate::db::{database, run_migrations};
    use std::sync::{Mutex, MutexGuard};

    static TEST_DB_LOCK: Mutex<()> = Mutex::new(());

    fn setup() -> MutexGuard<'static, ()> {
        let guard = TEST_DB_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        run_migrations();
        guard
    }

    // ── Inline seed schemas ──────────────────────────────────────────────────

    mod seed_schema {
        diesel::table! {
            users (id) {
                id -> Text,
                username -> Text,
                role -> Text,
                explicit_consent -> Bool,
                created_at -> Timestamp,
            }
        }

        diesel::table! {
            podcasts (id) {
                id -> Text,
                name -> Text,
                directory_id -> Text,
                rssfeed -> Text,
                image_url -> Text,
                active -> Bool,
                original_image_url -> Text,
                directory_name -> Text,
            }
        }

        diesel::table! {
            podcast_episodes (id) {
                id -> Text,
                podcast_id -> Text,
                episode_id -> Text,
                name -> Text,
                url -> Text,
                date_of_recording -> Text,
                image_url -> Text,
                total_time -> Integer,
                description -> Text,
                guid -> Text,
                deleted -> Bool,
                episode_numbering_processed -> Bool,
            }
        }
    }

    #[derive(diesel::Insertable)]
    #[diesel(table_name = seed_schema::users)]
    struct SeedUser {
        id: String,
        username: String,
        role: String,
        explicit_consent: bool,
        created_at: chrono::NaiveDateTime,
    }

    #[derive(diesel::Insertable)]
    #[diesel(table_name = seed_schema::podcasts)]
    struct SeedPodcast {
        id: String,
        name: String,
        directory_id: String,
        rssfeed: String,
        image_url: String,
        active: bool,
        original_image_url: String,
        directory_name: String,
    }

    #[derive(diesel::Insertable)]
    #[diesel(table_name = seed_schema::podcast_episodes)]
    struct SeedEpisode {
        id: String,
        podcast_id: String,
        episode_id: String,
        name: String,
        url: String,
        date_of_recording: String,
        image_url: String,
        total_time: i32,
        description: String,
        guid: String,
        deleted: bool,
        episode_numbering_processed: bool,
    }

    // ── Seed helpers ─────────────────────────────────────────────────────────

    fn seed_user() -> String {
        use seed_schema::users;
        let id = uuid::Uuid::new_v4().to_string();
        let mut conn = database().connection().expect("db connection");
        diesel::insert_into(users::table)
            .values(SeedUser {
                id: id.clone(),
                username: format!("sb-test-{id}"),
                role: "user".to_string(),
                explicit_consent: false,
                created_at: chrono::Utc::now().naive_utc(),
            })
            .execute(&mut conn)
            .expect("seed user");
        id
    }

    fn seed_podcast() -> String {
        use seed_schema::podcasts;
        let id = uuid::Uuid::new_v4().to_string();
        let mut conn = database().connection().expect("db connection");
        diesel::insert_into(podcasts::table)
            .values(SeedPodcast {
                id: id.clone(),
                name: format!("Test Podcast {id}"),
                directory_id: uuid::Uuid::new_v4().to_string(),
                rssfeed: format!("https://example.com/feed/{id}.xml"),
                image_url: "https://example.com/img.png".to_string(),
                active: true,
                original_image_url: "https://example.com/img.png".to_string(),
                directory_name: format!("podcast-{id}"),
            })
            .execute(&mut conn)
            .expect("seed podcast");
        id
    }

    fn seed_episode(podcast_id: &str) -> String {
        use seed_schema::podcast_episodes;
        let id = uuid::Uuid::new_v4().to_string();
        let mut conn = database().connection().expect("db connection");
        diesel::insert_into(podcast_episodes::table)
            .values(SeedEpisode {
                id: id.clone(),
                podcast_id: podcast_id.to_string(),
                episode_id: uuid::Uuid::new_v4().to_string(),
                name: "Test Episode".to_string(),
                url: format!("https://example.com/ep/{id}.mp3"),
                date_of_recording: "2024-01-01".to_string(),
                image_url: "https://example.com/ep.png".to_string(),
                total_time: 3600,
                description: "Test description".to_string(),
                guid: uuid::Uuid::new_v4().to_string(),
                deleted: false,
                episode_numbering_processed: false,
            })
            .execute(&mut conn)
            .expect("seed episode");
        id
    }

    fn make_segment(episode_id: &str, start_ms: i64, end_ms: i64) -> SponsorSegmentEntity {
        SponsorSegmentEntity {
            id: uuid::Uuid::new_v4().to_string(),
            episode_id: episode_id.to_string(),
            uuid: uuid::Uuid::new_v4().to_string(),
            category: "sponsor".to_string(),
            action_type: "skip".to_string(),
            start_ms,
            end_ms,
            votes: 0,
            locked: false,
            duration_mismatch: false,
            fetched_at: chrono::Utc::now().naive_utc(),
        }
    }

    fn default_user_settings(user_id: &str, enabled: bool) -> SponsorblockUserSettingsEntity {
        SponsorblockUserSettingsEntity {
            user_id: user_id.to_string(),
            enabled,
            skip_sponsor: true,
            skip_selfpromo: true,
            skip_interaction: false,
            skip_intro: false,
            skip_outro: false,
            skip_preview: false,
            skip_filler: false,
            skip_music_offtopic: false,
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn replace_is_idempotent() {
        let _guard = setup();
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);

        let seg1 = make_segment(&episode_id, 1000, 2000);
        let seg2 = make_segment(&episode_id, 3000, 4000);

        // First replace
        SponsorblockRepository::replace_segments_for_episode(
            &episode_id,
            vec![seg1.clone(), seg2.clone()],
        )
        .expect("first replace");

        // Second replace with same data
        SponsorblockRepository::replace_segments_for_episode(
            &episode_id,
            vec![seg1, seg2],
        )
        .expect("second replace");

        let result = SponsorblockRepository::get_segments_for_episode(&episode_id)
            .expect("get segments");

        assert_eq!(result.len(), 2, "expected exactly 2 segments");
        assert!(
            result[0].start_ms <= result[1].start_ms,
            "segments not ordered by start_ms ascending"
        );
        assert_eq!(result[0].start_ms, 1000);
        assert_eq!(result[1].start_ms, 3000);
    }

    #[test]
    fn replace_removes_stale_segments() {
        let _guard = setup();
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);

        // Insert 3 segments
        let segs3 = vec![
            make_segment(&episode_id, 100, 200),
            make_segment(&episode_id, 300, 400),
            make_segment(&episode_id, 500, 600),
        ];
        SponsorblockRepository::replace_segments_for_episode(&episode_id, segs3)
            .expect("replace 3");

        // Replace with 1 segment
        let seg_single = make_segment(&episode_id, 999, 1999);
        SponsorblockRepository::replace_segments_for_episode(&episode_id, vec![seg_single])
            .expect("replace 1");

        let result = SponsorblockRepository::get_segments_for_episode(&episode_id)
            .expect("get segments");

        assert_eq!(result.len(), 1, "expected exactly 1 segment after replace");
        assert_eq!(result[0].start_ms, 999);
    }

    #[test]
    fn user_settings_upsert_creates_then_updates() {
        let _guard = setup();
        let user_id = seed_user();

        // First upsert: create with enabled = true
        SponsorblockRepository::upsert_user_settings(default_user_settings(&user_id, true))
            .expect("upsert create");

        let settings = SponsorblockRepository::get_user_settings(&user_id)
            .expect("get settings after create")
            .expect("settings should exist");
        assert!(settings.enabled, "expected enabled=true after create");

        // Second upsert: update with enabled = false
        SponsorblockRepository::upsert_user_settings(default_user_settings(&user_id, false))
            .expect("upsert update");

        let settings2 = SponsorblockRepository::get_user_settings(&user_id)
            .expect("get settings after update")
            .expect("settings should still exist");
        assert!(!settings2.enabled, "expected enabled=false after update");

        // Confirm still exactly one row by checking there's only one result
        assert_eq!(settings2.user_id, user_id, "user_id matches");
    }
}
