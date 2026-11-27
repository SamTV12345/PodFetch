//! The catalog's API against the database.

#[cfg(test)]
mod tests {
    use crate::services::podcast::service::PodcastService;
    use crate::test_support::tests::{app_state, handle_test_startup};
    use diesel::{ExpressionMethods, RunQueryDsl};
    use podfetch_persistence::db::get_connection;
    use podfetch_persistence::schema::podcast_episodes::dsl as pe;
    use serial_test::serial;
    use uuid::Uuid;

    fn podcast(name: &str) -> Uuid {
        let id = PodcastService::add_podcast_to_database(
            name,
            &format!("dir-{name}"),
            &format!("https://example.com/{name}.xml"),
            "http://localhost:8080/ui/default.jpg",
            name,
        )
        .unwrap()
        .id;
        Uuid::parse_str(&id).unwrap()
    }

    fn episode(podcast_id: Uuid, key: &str, date: &str) -> Uuid {
        let id = Uuid::now_v7();
        diesel::insert_into(pe::podcast_episodes)
            .values((
                pe::id.eq(id.to_string()),
                pe::podcast_id.eq(podcast_id.to_string()),
                pe::episode_id.eq(key.to_string()),
                pe::name.eq(key.to_string()),
                pe::url.eq(format!("https://example.com/{key}.mp3")),
                pe::date_of_recording.eq(date.to_string()),
                pe::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe::total_time.eq(60),
                pe::description.eq("catalog test".to_string()),
                pe::guid.eq(format!("guid-{key}")),
                pe::episode_numbering_processed.eq(false),
            ))
            .execute(&mut get_connection())
            .unwrap();
        id
    }

    #[tokio::test]
    #[serial]
    async fn answers_what_other_contexts_ask() {
        let _ts = handle_test_startup().await;
        let catalog = app_state().catalog;
        let podcast_id = podcast("api");
        let other = podcast("other");
        let old = episode(podcast_id, "old", "2026-01-01T00:00:00Z");
        let middle = episode(podcast_id, "middle", "2026-02-01T00:00:00Z");
        let new = episode(podcast_id, "new", "2026-03-01T00:00:00Z");
        episode(other, "foreign", "2026-04-01T00:00:00Z");

        let found = catalog.podcast(podcast_id).unwrap().unwrap();
        assert_eq!(found.name, "api");
        assert_eq!(
            catalog
                .podcast_by_feed("https://example.com/api.xml")
                .unwrap()
                .map(|p| p.id),
            Some(podcast_id)
        );
        assert!(catalog.podcast(Uuid::now_v7()).unwrap().is_none());
        assert!(
            catalog
                .podcast_by_feed("https://example.com/unknown.xml")
                .unwrap()
                .is_none()
        );

        assert_eq!(catalog.episode(middle).unwrap().unwrap().name, "middle");
        assert!(catalog.episode(Uuid::now_v7()).unwrap().is_none());

        let mut all: Vec<Uuid> = catalog
            .episodes_of_podcast(podcast_id)
            .unwrap()
            .into_iter()
            .map(|e| e.id)
            .collect();
        all.sort();
        let mut expected = vec![old, middle, new];
        expected.sort();
        assert_eq!(all, expected, "only the podcast's own episodes");

        let latest: Vec<Uuid> = catalog
            .latest_episodes(podcast_id, 2)
            .unwrap()
            .into_iter()
            .map(|e| e.id)
            .collect();
        assert_eq!(latest, [new, middle], "newest first, at most n");
    }
}
