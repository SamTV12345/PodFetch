//! Media library behaviour that needs the database: the episodes' file
//! state, the SponsorBlock storage and the user cleanup.

#[cfg(test)]
mod tests {
    use crate::services::podcast::service::PodcastService;
    use crate::test_support::tests::{
        app_state, change_episode_files, episode_by_id, handle_test_startup, process_events,
    };
    use crate::usecases::podcast_episode::PodcastEpisodeUseCase;
    use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
    use diesel::{ExpressionMethods, RunQueryDsl};
    use podfetch_domain::podcast_episode::PodcastEpisodeRepository;
    use podfetch_identity::events::UserDeleted;
    use podfetch_kernel::UserId;
    use podfetch_media::files::{EpisodeFile, EpisodeFileRepository};
    use podfetch_media::handlers::CLEANUP_ON_USER_DELETED;
    use podfetch_media::persistence::{DieselEpisodeFileRepository, DieselSponsorblockRepository};
    use podfetch_media::sponsorblock::{SkipPreferences, SponsorSegment, SponsorblockRepository};
    use podfetch_persistence::db::{database, get_connection};
    use podfetch_persistence::podcast_episode::DieselPodcastEpisodeRepository;
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

    fn podcast(name: &str) -> String {
        PodcastService::add_podcast_to_database(
            name,
            &format!("dir-{name}"),
            &format!("https://example.com/{name}.xml"),
            "http://localhost:8080/ui/default.jpg",
            name,
        )
        .unwrap()
        .id
    }

    /// A catalog episode the segments can belong to; returns its id.
    fn episode() -> String {
        episode_with(
            &podcast("sponsorblock"),
            "sb-episode",
            "https://example.com/sb.mp3",
        )
    }

    /// A catalog episode of `podcast_id`; returns its id.
    fn episode_with(podcast_id: &str, key: &str, url: &str) -> String {
        let id = Uuid::now_v7().to_string();
        diesel::insert_into(pe::podcast_episodes)
            .values((
                pe::id.eq(id.clone()),
                pe::podcast_id.eq(podcast_id.to_string()),
                pe::episode_id.eq(key.to_string()),
                pe::name.eq(key.to_string()),
                pe::url.eq(url.to_string()),
                pe::date_of_recording.eq("2026-05-01T00:00:00Z".to_string()),
                pe::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe::total_time.eq(3600),
                pe::description.eq("media test".to_string()),
                pe::guid.eq(format!("guid-{key}")),
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
            .media
            .sponsorblock
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

    fn file_rows() -> i64 {
        use diesel::QueryDsl;
        podfetch_media::persistence::episode_file::episode_files::table
            .count()
            .get_result(&mut get_connection())
            .unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn episode_files_are_changed_in_place_and_not_kept_empty() {
        let _ts = handle_test_startup().await;
        let id = episode();
        let files = DieselEpisodeFileRepository::new(database());
        assert_eq!(files.find(&id).unwrap(), None);

        files
            .modify(&id, &mut |f| {
                f.file_episode_path = Some("p/podcast.mp3".into());
                f.deleted = true;
            })
            .unwrap();
        files
            .modify(&id, &mut |f| f.download_location = Some("Local".into()))
            .unwrap();
        assert_eq!(
            files.find(&id).unwrap(),
            Some(EpisodeFile {
                file_episode_path: Some("p/podcast.mp3".into()),
                download_location: Some("Local".into()),
                deleted: true,
                ..EpisodeFile::none(&id)
            })
        );
        assert_eq!(file_rows(), 1);

        files
            .modify(&id, &mut |f| *f = EpisodeFile::none(&id))
            .unwrap();
        assert_eq!(files.find(&id).unwrap(), None);
        assert_eq!(file_rows(), 0);
    }

    /// The catalog writes only its own columns: saving an episode it read
    /// before a download finished keeps what the download recorded.
    #[tokio::test]
    #[serial]
    async fn catalog_update_keeps_the_file_state() {
        let _ts = handle_test_startup().await;
        let id = episode();
        let read_before_download = episode_by_id(&id);
        change_episode_files(&read_before_download, |e| {
            e.file_episode_path = Some("p/podcast.mp3".into());
            e.download_location = Some("Local".into());
            e.download_time = Some(chrono::Utc::now().naive_utc());
        });

        let mut renamed = read_before_download.clone();
        renamed.name = "Renamed by a feed refresh".into();
        DieselPodcastEpisodeRepository::new(database())
            .update(&renamed.into())
            .unwrap();

        let stored = episode_by_id(&id);
        assert_eq!(stored.name, "Renamed by a feed refresh");
        assert_eq!(stored.file_episode_path.as_deref(), Some("p/podcast.mp3"));
        assert_eq!(stored.download_location.as_deref(), Some("Local"));
        assert!(stored.download_time.is_some());
    }

    /// Two podcasts can carry the same enclosure URL; a download marks only
    /// the episode that was downloaded.
    #[tokio::test]
    #[serial]
    async fn a_download_is_recorded_for_the_downloaded_episode_only() {
        let _ts = handle_test_startup().await;
        let url = "https://example.com/shared.mp3";
        let downloaded = episode_with(&podcast("first"), "first-ep", url);
        let other = episode_with(&podcast("second"), "second-ep", url);

        let stored = PodcastEpisodeUseCase::update_podcast_episode_status(
            &episode_by_id(&downloaded),
            Some(common_infrastructure::config::FileHandlerType::Local),
        )
        .unwrap();

        assert_eq!(stored.id, downloaded);
        assert!(stored.download_time.is_some());
        assert_eq!(stored.download_location.as_deref(), Some("Local"));
        let untouched = episode_by_id(&other);
        assert_eq!(untouched.download_time, None);
        assert_eq!(untouched.download_location, None);
    }

    /// Deleting a file keeps the user's "deleted" mark while the download
    /// state goes; downloading again lifts it.
    #[tokio::test]
    #[serial]
    async fn deleting_locally_marks_the_file_and_forgets_the_download() {
        let _ts = handle_test_startup().await;
        let id = episode();
        change_episode_files(&episode_by_id(&id), |e| {
            e.file_episode_path = Some("/tmp/podfetch-missing.mp3".into());
            e.download_location = Some("Local".into());
            e.download_time = Some(chrono::Utc::now().naive_utc());
        });

        PodcastEpisodeUseCase::remove_download_status_of_episode(Uuid::parse_str(&id).unwrap())
            .unwrap();
        PodcastEpisodeUseCase::update_deleted(&id, true).unwrap();
        let deleted = episode_by_id(&id);
        assert!(deleted.deleted);
        assert_eq!(deleted.file_episode_path, None);
        assert_eq!(deleted.download_location, None);
        assert_eq!(deleted.download_time, None);

        PodcastEpisodeUseCase::update_deleted(&id, false).unwrap();
        assert!(!episode_by_id(&id).deleted);
        assert_eq!(file_rows(), 0);
    }
}
