//! Media library behaviour that needs the database: the SponsorBlock
//! storage and the user cleanup.

#[cfg(test)]
mod tests {
    use crate::services::podcast::service::PodcastService;
    use crate::test_support::tests::{app_state, handle_test_startup, process_events};
    use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
    use diesel::{ExpressionMethods, RunQueryDsl};
    use podfetch_identity::events::UserDeleted;
    use podfetch_kernel::UserId;
    use podfetch_media::handlers::CLEANUP_ON_USER_DELETED;
    use podfetch_media::persistence::DieselSponsorblockRepository;
    use podfetch_media::sponsorblock::{SkipPreferences, SponsorSegment, SponsorblockRepository};
    use podfetch_persistence::db::{database, get_connection};
    use podfetch_persistence::schema::podcast_episodes::dsl as pe;
    use serial_test::serial;
    use uuid::Uuid;

    fn repository() -> DieselSponsorblockRepository {
        DieselSponsorblockRepository::new(database())
    }

    fn admin() -> (UserId, String) {
        let username = ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "postgres".to_string());
        let id = app_state()
            .user_auth_service
            .find_by_username(&username)
            .unwrap()
            .id;
        (UserId::from(id), username)
    }

    /// A catalog episode the segments can belong to; returns its id.
    fn episode() -> String {
        let podcast_id = PodcastService::add_podcast_to_database(
            "SponsorBlock",
            "dir-sponsorblock",
            "https://example.com/sponsorblock.xml",
            "http://localhost:8080/ui/default.jpg",
            "sponsorblock",
        )
        .unwrap()
        .id;
        let id = Uuid::now_v7().to_string();
        diesel::insert_into(pe::podcast_episodes)
            .values((
                pe::id.eq(id.clone()),
                pe::podcast_id.eq(podcast_id),
                pe::episode_id.eq("sb-episode".to_string()),
                pe::name.eq("SponsorBlock episode".to_string()),
                pe::url.eq("https://example.com/sb.mp3".to_string()),
                pe::date_of_recording.eq("2026-05-01T00:00:00Z".to_string()),
                pe::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe::total_time.eq(3600),
                pe::description.eq("media test".to_string()),
                pe::guid.eq("guid-sb".to_string()),
                pe::episode_numbering_processed.eq(false),
            ))
            .execute(&mut get_connection())
            .unwrap();
        id
    }

    fn segment(start_ms: i64, end_ms: i64) -> SponsorSegment {
        SponsorSegment {
            uuid: Uuid::new_v4().to_string(),
            category: "sponsor".to_string(),
            action_type: "skip".to_string(),
            start_ms,
            end_ms,
            votes: 2,
            locked: true,
            duration_mismatch: true,
            // Whole seconds: Postgres keeps timestamps to the microsecond.
            fetched_at: chrono::DateTime::from_timestamp(1_780_000_000, 0)
                .unwrap()
                .naive_utc(),
        }
    }

    fn starts(episode_id: &str) -> Vec<i64> {
        repository()
            .segments(episode_id)
            .unwrap()
            .into_iter()
            .map(|s| s.start_ms)
            .collect()
    }

    #[tokio::test]
    #[serial]
    async fn segments_are_stored_ordered_and_replaced() {
        let _ts = handle_test_startup().await;
        let episode_id = episode();
        let later = segment(3000, 4000);
        let earlier = segment(1000, 2000);

        repository()
            .replace_segments(&episode_id, &[later.clone(), earlier.clone()])
            .unwrap();
        assert_eq!(starts(&episode_id), [1000, 3000]);
        assert_eq!(repository().segments(&episode_id).unwrap()[0], earlier);

        // Replacing with the same segments again keeps them once.
        repository()
            .replace_segments(&episode_id, &[later, earlier])
            .unwrap();
        assert_eq!(starts(&episode_id), [1000, 3000]);

        // Replacing drops the segments that are gone.
        repository()
            .replace_segments(&episode_id, &[segment(999, 1999)])
            .unwrap();
        assert_eq!(starts(&episode_id), [999]);

        repository().replace_segments(&episode_id, &[]).unwrap();
        assert!(starts(&episode_id).is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn preferences_are_created_then_updated() {
        let _ts = handle_test_startup().await;
        let (user, _) = admin();
        assert_eq!(repository().preferences(user).unwrap(), None);

        let first = SkipPreferences {
            skip_intro: true,
            ..SkipPreferences::default()
        };
        repository().save_preferences(user, &first).unwrap();
        assert_eq!(repository().preferences(user).unwrap(), Some(first));

        let second = SkipPreferences {
            enabled: false,
            ..SkipPreferences::default()
        };
        repository().save_preferences(user, &second).unwrap();
        assert_eq!(repository().preferences(user).unwrap(), Some(second));
    }

    /// The media library removes the preferences of a deleted user when it
    /// handles `identity.user_deleted`, without relying on the foreign key
    /// cascade: the user row still exists here.
    #[tokio::test]
    #[serial]
    async fn user_deleted_removes_sponsorblock_preferences() {
        let _ts = handle_test_startup().await;
        let state = app_state();
        let (user, username) = admin();
        state
            .sponsorblock_service
            .save_preferences(
                user,
                &SkipPreferences {
                    skip_outro: true,
                    ..SkipPreferences::default()
                },
            )
            .unwrap();

        state
            .event_bus
            .publish(&UserDeleted {
                user_id: user,
                username,
            })
            .unwrap();
        process_events().await;

        let cleanup = state
            .event_bus
            .deliveries(None, 100)
            .unwrap()
            .into_iter()
            .find(|d| d.handler == CLEANUP_ON_USER_DELETED)
            .expect("cleanup delivery");
        assert_eq!(cleanup.status, "done");
        assert_eq!(repository().preferences(user).unwrap(), None);
    }
}
