//! Listening behaviour that needs the whole application: progress against
//! catalog episodes, the timeline composition and the user cleanup.

#[cfg(test)]
mod tests {
    use crate::services::podcast::service::PodcastService;
    use crate::test_support::tests::{app_state, handle_test_startup};
    use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
    use diesel::{ExpressionMethods, RunQueryDsl};
    use podfetch_identity::events::UserDeleted;
    use podfetch_kernel::UserId;
    use podfetch_listening::domain::episode::{EpisodeRepository, Listener};
    use podfetch_listening::handlers::CLEANUP_ON_USER_DELETED;
    use podfetch_persistence::db::get_connection;
    use podfetch_persistence::schema::podcast_episodes::dsl as pe;
    use serde_json::{Value, json};
    use serial_test::serial;
    use std::time::Duration;
    use uuid::Uuid;

    fn admin() -> (Uuid, String) {
        let username = ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "postgres".to_string());
        let id = app_state()
            .user_auth_service
            .find_by_username(&username)
            .unwrap()
            .id;
        (id, username)
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

    /// An episode of 100 seconds recorded on `date`.
    fn episode(podcast_id: &str, key: &str, date: &str) {
        diesel::insert_into(pe::podcast_episodes)
            .values((
                pe::id.eq(Uuid::now_v7().to_string()),
                pe::podcast_id.eq(podcast_id.to_string()),
                pe::episode_id.eq(key.to_string()),
                pe::name.eq(key.to_string()),
                pe::url.eq(format!("https://example.com/{key}.mp3")),
                pe::date_of_recording.eq(date.to_string()),
                pe::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe::total_time.eq(100),
                pe::description.eq("listening test".to_string()),
                pe::guid.eq(format!("guid-{key}")),
                pe::deleted.eq(false),
                pe::episode_numbering_processed.eq(false),
            ))
            .execute(&mut get_connection())
            .unwrap();
    }

    async fn listen(server: &axum_test::TestServer, key: &str, seconds: i32) {
        let response = server
            .post("/api/v1/podcasts/episode")
            .json(&json!({ "podcastEpisodeId": key, "time": seconds }))
            .await;
        assert_eq!(response.status_code(), 200);
    }

    fn keys(items: &[Value], pointer: &str) -> Vec<String> {
        items
            .iter()
            .map(|item| item.pointer(pointer).unwrap().as_str().unwrap().to_string())
            .collect()
    }

    /// `onlyUnlistened` hides finished episodes and keeps the started ones;
    /// the history of each episode comes from the listening context.
    #[tokio::test]
    #[serial]
    async fn episode_list_filters_heard_episodes() {
        let ts = handle_test_startup().await;
        let server = &ts.test_server;
        let podcast_id = podcast("Progress");
        episode(&podcast_id, "finished", "2026-03-03T00:00:00Z");
        episode(&podcast_id, "started", "2026-03-02T00:00:00Z");
        episode(&podcast_id, "fresh", "2026-03-01T00:00:00Z");
        listen(server, "finished", 100).await;
        listen(server, "started", 40).await;

        let all = server
            .get(&format!("/api/v1/podcasts/{podcast_id}/episodes"))
            .await
            .json::<Vec<Value>>();
        assert_eq!(
            keys(&all, "/podcastEpisode/episode_id"),
            ["finished", "started", "fresh"]
        );
        assert_eq!(all[1]["podcastHistoryItem"]["position"], 40);
        assert!(all[2]["podcastHistoryItem"].is_null());

        let unheard = server
            .get(&format!(
                "/api/v1/podcasts/{podcast_id}/episodes?only_unlistened=true"
            ))
            .await
            .json::<Vec<Value>>();
        assert_eq!(
            keys(&unheard, "/podcastEpisode/episode_id"),
            ["started", "fresh"]
        );
    }

    /// The timeline restricted to favorite podcasts and to episodes without
    /// any action, with a total that matches the filters.
    #[tokio::test]
    #[serial]
    async fn timeline_combines_favorites_and_listening_state() {
        let ts = handle_test_startup().await;
        let server = &ts.test_server;
        let (admin_id, _) = admin();
        let favored = podcast("Favored");
        let other = podcast("Other");
        PodcastService::update_favor_podcast(Uuid::parse_str(&favored).unwrap(), true, admin_id)
            .unwrap();
        episode(&favored, "fav-new", "2026-04-03T00:00:00Z");
        episode(&favored, "fav-heard", "2026-04-02T00:00:00Z");
        episode(&other, "other-new", "2026-04-04T00:00:00Z");
        listen(server, "fav-heard", 10).await;

        let timeline = |query: &'static str| async move {
            server
                .get(&format!("/api/v1/podcasts/timeline?{query}"))
                .await
                .json::<Value>()
        };

        let everything =
            timeline("favoredOnly=false&notListened=false&favoredEpisodes=false").await;
        assert_eq!(everything["totalElements"], 3);
        let data = everything["data"].as_array().unwrap();
        assert_eq!(
            keys(data, "/podcast_episode/episode_id"),
            ["other-new", "fav-new", "fav-heard"]
        );
        assert_eq!(
            data[2]["history"]["position"], 10,
            "history of the heard episode"
        );
        assert_eq!(
            data[1]["favorite"]["favored"], true,
            "podcast favorite flag"
        );

        let favored_unheard =
            timeline("favoredOnly=true&notListened=true&favoredEpisodes=false").await;
        assert_eq!(favored_unheard["totalElements"], 1);
        assert_eq!(
            keys(
                favored_unheard["data"].as_array().unwrap(),
                "/podcast_episode/episode_id"
            ),
            ["fav-new"]
        );
    }

    /// Listening removes the actions of a deleted user when it handles
    /// `identity.user_deleted`, without relying on the foreign key cascade.
    #[tokio::test]
    #[serial]
    async fn user_deleted_removes_episode_actions() {
        let ts = handle_test_startup().await;
        let state = app_state();
        let (admin_id, admin_name) = admin();
        let podcast_id = podcast("Cleanup");
        episode(&podcast_id, "cleanup", "2026-05-01T00:00:00Z");
        listen(&ts.test_server, "cleanup", 30).await;
        let listener = Listener::new(admin_id, admin_name.clone());
        let actions = crate::listening::episode_actions();
        assert_eq!(actions.guids_with_actions(&listener).unwrap().len(), 1);

        let processor = state.event_bus.spawn_processor();
        state
            .event_bus
            .publish(&UserDeleted {
                user_id: UserId::from(admin_id),
                username: admin_name,
            })
            .unwrap();
        let mut done = false;
        for _ in 0..100 {
            done = state
                .event_bus
                .deliveries(None, 100)
                .unwrap()
                .iter()
                .any(|d| d.handler == CLEANUP_ON_USER_DELETED && d.status == "done");
            if done {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        processor.stop(Duration::from_secs(1)).await;

        assert!(done, "cleanup handler ran");
        assert!(actions.guids_with_actions(&listener).unwrap().is_empty());
        assert!(
            crate::listening::listening_events()
                .get_by_user_and_range(admin_id, None, None)
                .unwrap()
                .is_empty()
        );
    }
}
