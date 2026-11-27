//! Audiobooks: the user cleanup.

#[cfg(test)]
mod tests {
    use crate::test_support::tests::{app_state, handle_test_startup, process_events};
    use chrono::Utc;
    use podfetch_audiobooks::domain::media_progress::{MediaProgress, MediaProgressRepository};
    use podfetch_audiobooks::persistence::MediaProgressRepositoryImpl;
    use podfetch_identity::events::UserDeleted;
    use podfetch_kernel::UserId;
    use podfetch_persistence::db::database;
    use serial_test::serial;
    use uuid::Uuid;

    fn progress_of(user_id: Uuid, item: &str) -> MediaProgress {
        let now = Utc::now().naive_utc();
        MediaProgress {
            id: MediaProgress::compose_id(item, None),
            user_id,
            library_item_id: item.to_string(),
            episode_id: None,
            media_type: "book".into(),
            duration: 100.0,
            current_time: 10.0,
            progress: 0.1,
            is_finished: false,
            hide_from_continue_listening: false,
            last_update: now,
            started_at: now,
            finished_at: None,
        }
    }

    /// The Audiobookshelf tables have no foreign key to the users; the
    /// audiobooks context removes a deleted user's progress itself.
    #[tokio::test]
    #[serial]
    async fn user_deleted_removes_audiobook_progress() {
        let _ts = handle_test_startup().await;
        let leaving = Uuid::now_v7();
        let staying = Uuid::now_v7();
        let repo = MediaProgressRepositoryImpl::new(database());
        repo.upsert(progress_of(leaving, "book-a")).unwrap();
        repo.upsert(progress_of(staying, "book-b")).unwrap();

        app_state()
            .event_bus
            .publish(&UserDeleted {
                user_id: UserId::from(leaving),
                username: "leaving_reader".into(),
            })
            .unwrap();
        process_events().await;

        assert!(repo.list_for_user(leaving).unwrap().is_empty());
        assert_eq!(repo.list_for_user(staying).unwrap().len(), 1);
    }
}
