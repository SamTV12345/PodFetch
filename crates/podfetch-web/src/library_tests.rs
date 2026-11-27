//! Lists that combine catalog and library data (API composition).

#[cfg(test)]
mod tests {
    use crate::services::podcast::service::PodcastService;
    use crate::test_support::tests::handle_test_startup;
    use common_infrastructure::config::EnvironmentService;
    use podfetch_catalog::domain::ordering::{OrderCriteria, OrderOption};
    use podfetch_identity::api::Principal;
    use podfetch_identity::persistence::adapters::UserAdminRepositoryImpl;
    use podfetch_identity::services::user_auth::service::UserAuthService;
    use podfetch_persistence::db::database;
    use serial_test::serial;
    use std::sync::Arc;
    use uuid::Uuid;

    fn create_user(name: &str) -> Principal {
        let user = UserAuthService::new(
            Arc::new(UserAdminRepositoryImpl::new(database())),
            Arc::new(EnvironmentService::for_tests()),
        )
        .create_user(name.to_string(), "user".to_string(), None, false)
        .unwrap();
        Principal::from(&user)
    }

    fn podcast(name: &str) -> Uuid {
        let created = PodcastService::add_podcast_to_database(
            name,
            &format!("dir-{name}"),
            &format!("https://example.com/{name}.xml"),
            "image.png",
            name,
        )
        .unwrap();
        Uuid::parse_str(&created.id).unwrap()
    }

    fn names(podcasts: &[crate::podcast::PodcastDto]) -> Vec<&str> {
        podcasts.iter().map(|p| p.name.as_str()).collect()
    }

    /// Only podcasts the user currently favors, in the requested order.
    /// Before the split this list came back sorted by id and still
    /// contained podcasts whose favorite had been removed.
    #[tokio::test]
    #[serial]
    async fn favored_search_keeps_order_and_skips_removed_favorites() {
        let _ts = handle_test_startup().await;
        let user = create_user("library_reader");
        let alpha = podcast("Alpha");
        let bravo = podcast("Bravo");
        let charlie = podcast("Charlie");
        let delta = podcast("Delta");
        for id in [alpha, bravo, charlie, delta] {
            PodcastService::update_favor_podcast(id, true, user.id.into()).unwrap();
        }
        PodcastService::update_favor_podcast(bravo, false, user.id.into()).unwrap();

        let desc = PodcastService::search_podcasts_favored(
            OrderCriteria::Desc,
            None,
            OrderOption::Title,
            user.id.into(),
            None,
            None,
            "http://localhost",
        )
        .unwrap();
        assert_eq!(names(&desc), ["Delta", "Charlie", "Alpha"]);
        assert!(desc.iter().all(|p| p.favorites));
    }

    /// The full list carries each user's own favorite flag and tags.
    #[tokio::test]
    #[serial]
    async fn search_combines_favorites_and_tags_of_the_requesting_user() {
        let _ts = handle_test_startup().await;
        let reader = create_user("library_reader");
        let other = create_user("library_other");
        let alpha = podcast("Alpha");
        let bravo = podcast("Bravo");
        PodcastService::update_favor_podcast(alpha, true, reader.id.into()).unwrap();
        PodcastService::update_favor_podcast(bravo, true, other.id.into()).unwrap();
        let tags = crate::library::tags();
        let news = tags
            .create_tag(reader.id.into(), "News".into(), None, "Red".into())
            .unwrap();
        tags.add_podcast_to_tag(reader.id.into(), &news.id, bravo)
            .unwrap();
        let foreign = tags
            .create_tag(other.id.into(), "Foreign".into(), None, "Blue".into())
            .unwrap();
        tags.add_podcast_to_tag(other.id.into(), &foreign.id, alpha)
            .unwrap();

        let all = PodcastService::search_podcasts(
            OrderCriteria::Asc,
            None,
            OrderOption::Title,
            None,
            &reader,
            None,
            "http://localhost",
        )
        .unwrap();
        assert_eq!(names(&all), ["Alpha", "Bravo"]);
        assert!(all[0].favorites, "reader favors Alpha");
        assert!(!all[1].favorites, "Bravo is only favored by someone else");
        assert!(all[0].tags.is_empty(), "other users' tags stay hidden");
        assert_eq!(all[1].tags.len(), 1);
        assert_eq!(all[1].tags[0].name, "News");

        let tagged = PodcastService::search_podcasts(
            OrderCriteria::Asc,
            Some("b".into()),
            OrderOption::Title,
            Some(news.id.clone()),
            &reader,
            None,
            "http://localhost",
        )
        .unwrap();
        assert_eq!(names(&tagged), ["Bravo"]);
    }
}
