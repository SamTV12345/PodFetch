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
    use podfetch_catalog::domain::podcast_episode::PodcastEpisodeRepository;
    use podfetch_catalog::persistence::podcast_episode::DieselPodcastEpisodeRepository;
    use podfetch_identity::events::UserDeleted;
    use podfetch_kernel::UserId;
    use podfetch_media::files::{EpisodeFile, EpisodeFileRepository};
    use podfetch_media::handlers::CLEANUP_ON_USER_DELETED;
    use podfetch_media::persistence::{DieselEpisodeFileRepository, DieselSponsorblockRepository};
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

        crate::media::episode_files()
            .record_download(
                &downloaded,
                Some("Local".to_string()),
                chrono::Utc::now().naive_utc(),
            )
            .unwrap();
        let stored = episode_by_id(&downloaded);

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

        crate::media::episode_files().clear_download(&id).unwrap();
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

    fn set_auto_download(on: bool) {
        let settings = app_state().settings_service.clone();
        let mut current = settings.get_settings().unwrap().unwrap();
        current.auto_download = on;
        current.podcast_prefill = 5;
        settings.update_settings(current).unwrap();
    }

    /// Delivers the pending events with `state`'s own handlers (each
    /// `app_state()` has its own download queue), then waits until its
    /// downloads are done.
    async fn deliver_and_download(state: &crate::app_state::AppState) {
        let processor = state.event_bus.spawn_processor();
        for _ in 0..200 {
            if delivery_status(podfetch_media::handlers::AUTO_DOWNLOAD_ON_PODCAST_REFRESHED)
                .is_some_and(|s| s == "done")
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        processor.stop(std::time::Duration::from_secs(1)).await;
        let queue = state.media.auto_download.queue().clone();
        let idle = tokio::task::spawn_blocking(move || {
            queue.wait_idle(std::time::Duration::from_secs(30))
        })
        .await
        .unwrap();
        assert!(idle, "downloads finish");
    }

    fn delivery_status(handler: &str) -> Option<String> {
        app_state()
            .event_bus
            .deliveries(None, 500)
            .unwrap()
            .into_iter()
            .find(|d| d.handler == handler)
            .map(|d| d.status)
    }

    /// Catalog → event → media library → download: a refreshed podcast with
    /// auto-download gets its episodes downloaded (here: attempted, the
    /// enclosure is unreachable, which the download announces as failed).
    /// Episodes the user deleted stay deleted.
    #[tokio::test]
    #[serial]
    async fn a_refreshed_podcast_gets_its_episodes_downloaded() {
        let _ts = handle_test_startup().await;
        let state = app_state();
        set_auto_download(true);
        let podcast_id = podcast("auto");
        episode_with(&podcast_id, "wanted", "http://127.0.0.1:9/wanted.mp3");
        let deleted = episode_with(&podcast_id, "deleted", "http://127.0.0.1:9/deleted.mp3");
        PodcastEpisodeUseCase::update_deleted(&deleted, true).unwrap();

        crate::podcast_events::publish(&podfetch_catalog::events::PodcastRefreshed {
            podcast_id: podcast_id.clone(),
            podcast_name: "auto".into(),
        });
        deliver_and_download(&state).await;
        assert_eq!(
            delivery_status(podfetch_media::handlers::AUTO_DOWNLOAD_ON_PODCAST_REFRESHED)
                .as_deref(),
            Some("done")
        );

        let failed: Vec<String> = state
            .event_bus
            .deliveries(None, 500)
            .unwrap()
            .into_iter()
            .filter(|d| d.event_type == "podcast.episode_download_failed")
            .map(|d| d.aggregate_id)
            .collect();
        assert!(failed.contains(&"wanted".to_string()), "{failed:?}");
        assert!(!failed.contains(&"deleted".to_string()), "{failed:?}");
        set_auto_download(false);
    }

    #[tokio::test]
    #[serial]
    async fn without_auto_download_a_refresh_downloads_nothing() {
        let _ts = handle_test_startup().await;
        let state = app_state();
        set_auto_download(false);
        let podcast_id = podcast("manual");
        episode_with(&podcast_id, "manual-ep", "http://127.0.0.1:9/manual.mp3");

        crate::podcast_events::publish(&podfetch_catalog::events::PodcastRefreshed {
            podcast_id,
            podcast_name: "manual".into(),
        });
        deliver_and_download(&state).await;

        let attempted = state
            .event_bus
            .deliveries(None, 500)
            .unwrap()
            .into_iter()
            .any(|d| d.event_type.starts_with("podcast.episode_download"));
        assert!(!attempted);
    }

    /// Parallel downloads change the file state of different episodes at
    /// the same time. On SQLite a transaction that reads before it writes
    /// fails at once with "database is locked" when another connection
    /// wrote in between (the busy timeout doesn't apply); `modify` must take
    /// the write lock first.
    #[tokio::test]
    #[serial]
    async fn concurrent_file_changes_do_not_lock_each_other_out() {
        let _ts = handle_test_startup().await;
        let podcast_id = podcast("parallel");
        let ids: Vec<String> = (0..8)
            .map(|i| {
                episode_with(
                    &podcast_id,
                    &format!("par-{i}"),
                    &format!("https://example.com/{i}.mp3"),
                )
            })
            .collect();

        let errors: Vec<String> = std::thread::scope(|scope| {
            let workers: Vec<_> = ids
                .iter()
                .map(|id| {
                    scope.spawn(move || {
                        let files = DieselEpisodeFileRepository::new(database());
                        let mut errors = Vec::new();
                        for round in 0..25 {
                            if let Err(err) = files.modify(id, &mut |f| {
                                f.file_episode_path = Some(format!("{id}/{round}.mp3"));
                            }) {
                                errors.push(err.to_string());
                            }
                        }
                        errors
                    })
                })
                .collect();
            workers
                .into_iter()
                .flat_map(|w| w.join().unwrap())
                .collect()
        });

        assert!(
            errors.is_empty(),
            "{} failed: {:?}",
            errors.len(),
            errors.first()
        );
        for id in &ids {
            let stored = DieselEpisodeFileRepository::new(database())
                .find(id)
                .unwrap();
            assert_eq!(
                stored.and_then(|f| f.file_episode_path),
                Some(format!("{id}/24.mp3"))
            );
        }
    }

    // ── Library commands ────────────────────────────────────────────────

    /// A file on the local storage, in a directory of its own.
    fn local_file(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("podfetch-media-{}", Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, b"audio").unwrap();
        path.to_string_lossy().into_owned()
    }

    /// Records the episode as downloaded to `path` on the local storage.
    fn downloaded_to(id: &str, path: &str) {
        change_episode_files(&episode_by_id(id), |e| {
            e.file_episode_path = Some(path.to_string());
            e.download_location = Some("Local".into());
            e.download_time = Some(chrono::Utc::now().naive_utc());
        });
    }

    fn exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    #[tokio::test]
    #[serial]
    async fn a_file_deleted_on_request_is_gone_and_stays_gone() {
        let _ts = handle_test_startup().await;
        let id = episode_with(&podcast("lib-delete"), "d1", "https://example.com/d1.mp3");
        let path = local_file("d1.mp3");
        downloaded_to(&id, &path);

        app_state()
            .media
            .library
            .delete_on_request(&crate::media::media_episode(&episode_by_id(&id)))
            .unwrap();

        assert!(!exists(&path));
        let stored = episode_by_id(&id);
        assert!(stored.deleted, "not downloaded again on its own");
        assert_eq!(stored.download_location, None);
        assert_eq!(stored.file_episode_path, None);
    }

    #[tokio::test]
    #[serial]
    async fn a_resync_forgets_downloads_whose_file_is_gone() {
        let _ts = handle_test_startup().await;
        let podcast_id = podcast("lib-resync");
        let present = episode_with(&podcast_id, "r1", "https://example.com/r1.mp3");
        let lost = episode_with(&podcast_id, "r2", "https://example.com/r2.mp3");
        let path = local_file("r1.mp3");
        downloaded_to(&present, &path);
        downloaded_to(&lost, "/tmp/podfetch-media-gone/r2.mp3");

        let affected = app_state().media.library.resync(&podcast_id).unwrap();

        assert_eq!(affected, 1);
        assert_eq!(episode_by_id(&lost).download_location, None);
        assert!(!episode_by_id(&lost).deleted);
        assert_eq!(
            episode_by_id(&present).download_location.as_deref(),
            Some("Local")
        );
        assert!(exists(&path));
    }

    #[tokio::test]
    #[serial]
    async fn removing_a_podcasts_files_keeps_the_episodes_asked_for() {
        let _ts = handle_test_startup().await;
        let podcast_id = podcast("lib-remove");
        let kept = episode_with(&podcast_id, "k1", "https://example.com/k1.mp3");
        let removed = episode_with(&podcast_id, "k2", "https://example.com/k2.mp3");
        let never = episode_with(&podcast_id, "k3", "https://example.com/k3.mp3");
        let kept_path = local_file("k1.mp3");
        let removed_path = local_file("k2.mp3");
        downloaded_to(&kept, &kept_path);
        downloaded_to(&removed, &removed_path);

        let keep = std::collections::HashSet::from([kept.clone()]);
        let affected = app_state()
            .media
            .library
            .remove_files(&podcast_id, &keep)
            .unwrap();

        assert_eq!(affected, 1);
        assert!(exists(&kept_path));
        assert!(!exists(&removed_path));
        assert_eq!(episode_by_id(&removed).download_location, None);
        assert!(
            !episode_by_id(&removed).deleted,
            "may be downloaded again on its own"
        );
        assert_eq!(episode_by_id(&never).download_location, None);
    }

    /// Cleanup removes the files of old episodes, but not of favorites.
    #[tokio::test]
    #[serial]
    async fn the_cleanup_spares_favorites() {
        let _ts = handle_test_startup().await;
        let podcast_id = podcast("lib-cleanup");
        let old = episode_with(&podcast_id, "c1", "https://example.com/c1.mp3");
        let favorite = episode_with(&podcast_id, "c2", "https://example.com/c2.mp3");
        let old_path = local_file("c1.mp3");
        let favorite_path = local_file("c2.mp3");
        downloaded_to(&old, &old_path);
        downloaded_to(&favorite, &favorite_path);
        crate::library::favorite_episodes()
            .set_favorite(admin().0.into(), Uuid::parse_str(&favorite).unwrap(), true)
            .unwrap();
        // Recorded long ago, downloaded long ago.
        for id in [&old, &favorite] {
            change_episode_files(&episode_by_id(id), |e| {
                e.download_time = Some(
                    chrono::NaiveDate::from_ymd_opt(2020, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                );
            });
        }

        PodcastEpisodeUseCase::cleanup_old_episodes(&app_state().media.library, 1);

        assert!(!exists(&old_path));
        assert_eq!(episode_by_id(&old).download_location, None);
        assert!(exists(&favorite_path));
    }
}
