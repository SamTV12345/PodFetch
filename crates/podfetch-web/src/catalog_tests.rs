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

    // ── Feed refresh ────────────────────────────────────────────────────

    use common_infrastructure::error::CustomError;
    use podfetch_catalog::feed::{FeedFetcher, FeedSync, FetchedFeed};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Serves feeds by URL; they can change between refreshes, and a URL
    /// can redirect (permanently or not) to another one.
    #[derive(Default)]
    struct Feeds {
        content: Mutex<HashMap<String, String>>,
        redirects: Mutex<HashMap<String, (String, bool)>>,
    }

    impl Feeds {
        fn set(&self, url: &str, xml: String) {
            self.content.lock().unwrap().insert(url.to_string(), xml);
        }

        fn redirect(&self, from: &str, to: &str, permanently: bool) {
            self.redirects
                .lock()
                .unwrap()
                .insert(from.to_string(), (to.to_string(), permanently));
        }
    }

    impl FeedFetcher for Feeds {
        fn fetch(&self, feed_url: &str) -> Result<FetchedFeed, CustomError> {
            let (url, moved_permanently) = self
                .redirects
                .lock()
                .unwrap()
                .get(feed_url)
                .cloned()
                .unwrap_or_else(|| (feed_url.to_string(), false));
            let content = self
                .content
                .lock()
                .unwrap()
                .get(&url)
                .cloned()
                .ok_or_else(|| {
                    CustomError::from(common_infrastructure::error::CustomErrorInner::NotFound(
                        common_infrastructure::error::ErrorSeverity::Warning,
                    ))
                })?;
            Ok(FetchedFeed {
                url,
                content,
                moved_permanently,
            })
        }
    }

    fn rss(channel_extra: &str, items: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:podcast="https://podcastindex.org/namespace/1.0">
<channel><title>Sync</title><link>https://example.com</link><description>About sync</description>
<language>de</language><itunes:author>Author</itunes:author><itunes:category text="Tech"/>
{channel_extra}{items}</channel></rss>"#
        )
    }

    fn item(guid: &str, title: &str, extra: &str) -> String {
        format!(
            r#"<item><title>{title}</title><guid>{guid}</guid><pubDate>Thu, 02 Jan 2025 10:00:00 +0000</pubDate>
<enclosure url="https://example.com/{guid}.mp3" type="audio/mpeg" length="1"/>{extra}</item>"#
        )
    }

    const FEED: &str = "https://example.com/sync.xml";

    fn sync_fixture() -> (
        FeedSync,
        Arc<Feeds>,
        podfetch_catalog::domain::podcast::Podcast,
    ) {
        let feeds = Arc::new(Feeds::default());
        let sync = FeedSync::new(podfetch_persistence::db::database(), feeds.clone());
        let podcast_id = PodcastService::add_podcast_to_database(
            "sync",
            "dir-sync",
            FEED,
            "http://localhost:8080/ui/default.jpg",
            "sync",
        )
        .unwrap()
        .id;
        let podcast = app_state()
            .catalog
            .podcast(Uuid::parse_str(&podcast_id).unwrap())
            .unwrap()
            .unwrap();
        (sync, feeds, podcast)
    }

    #[tokio::test]
    #[serial]
    async fn a_refresh_adds_new_episodes_and_reports_what_others_need() {
        let _ts = handle_test_startup().await;
        let (sync, feeds, podcast) = sync_fixture();
        feeds.set(
            FEED,
            rss(
                r#"<itunes:image href="https://example.com/cover.jpg"/>"#,
                &[
                    item(
                        "g1",
                        "First",
                        r#"<itunes:duration>1:30</itunes:duration><itunes:image href="https://example.com/e1.jpg"/>
<podcast:transcript url="https://example.com/g1.vtt" type="text/vtt" language="de"/>"#,
                    ),
                    item("g2", "Second", ""),
                    r#"<item><title>No enclosure</title><guid>g3</guid></item>"#.to_string(),
                ]
                .concat(),
            ),
        );

        let report = sync.sync(&podcast).unwrap();

        let mut names: Vec<String> = report.inserted.iter().map(|e| e.name.clone()).collect();
        names.sort();
        assert_eq!(
            names,
            ["First", "Second"],
            "items without enclosure are skipped"
        );
        let first = report.inserted.iter().find(|e| e.guid == "g1").unwrap();
        assert_eq!(first.total_time, 90);
        assert_eq!(first.image_url, "https://example.com/e1.jpg");
        assert_eq!(first.url, "https://example.com/g1.mp3");
        assert_eq!(first.date_of_recording, "2025-01-02T10:00:00+00:00");
        assert_eq!(report.transcripts.len(), 1);
        assert_eq!(report.transcripts[0].0, first.id);
        assert_eq!(report.transcripts[0].1[0].url, "https://example.com/g1.vtt");
        assert_eq!(
            report.new_cover.as_deref(),
            Some("https://example.com/cover.jpg"),
            "the podcast had only the default cover"
        );

        let stored = app_state().catalog.podcast(podcast.id).unwrap().unwrap();
        assert_eq!(stored.author.as_deref(), Some("Author"));
        assert_eq!(stored.keywords.as_deref(), Some("Tech"));
        assert_eq!(stored.language.as_deref(), Some("de"));
        assert_eq!(stored.original_image_url, "https://example.com/cover.jpg");
    }

    #[tokio::test]
    #[serial]
    async fn a_second_refresh_updates_known_episodes_only() {
        let _ts = handle_test_startup().await;
        let (sync, feeds, podcast) = sync_fixture();
        feeds.set(FEED, rss("", &item("g1", "First", "")));
        sync.sync(&podcast).unwrap();
        let podcast = app_state().catalog.podcast(podcast.id).unwrap().unwrap();

        feeds.set(
            FEED,
            rss(
                "",
                &item(
                    "g1",
                    "First, renamed",
                    r#"<podcast:transcript url="https://example.com/g1.srt" type="application/srt"/>"#,
                ),
            ),
        );
        let report = sync.sync(&podcast).unwrap();

        assert!(report.inserted.is_empty());
        assert_eq!(report.new_cover, None);
        let episodes = app_state().catalog.episodes_of_podcast(podcast.id).unwrap();
        assert_eq!(episodes.len(), 1);
        assert_eq!(episodes[0].name, "First, renamed");
        assert_eq!(
            report.transcripts,
            [(
                episodes[0].id,
                vec![podfetch_catalog::feed::FeedTranscript {
                    url: "https://example.com/g1.srt".into(),
                    mime_type: "application/srt".into(),
                    language: None,
                }]
            )],
            "tags of known episodes are reported too"
        );
    }

    #[tokio::test]
    #[serial]
    async fn a_new_feed_url_is_followed() {
        let _ts = handle_test_startup().await;
        let (sync, feeds, podcast) = sync_fixture();
        feeds.set(FEED, rss("", &item("g1", "First", "")));
        sync.sync(&podcast).unwrap();
        let podcast = app_state().catalog.podcast(podcast.id).unwrap().unwrap();

        // The old feed announces its new home; its items carry new
        // enclosure URLs.
        feeds.set(
            FEED,
            rss(
                "<itunes:new-feed-url>https://example.com/moved.xml</itunes:new-feed-url>",
                &item("g1", "First", "").replace("g1.mp3", "moved/g1.mp3"),
            ),
        );
        sync.sync(&podcast).unwrap();

        let stored = app_state().catalog.podcast(podcast.id).unwrap().unwrap();
        assert_eq!(stored.rssfeed, "https://example.com/moved.xml");
        let episodes = app_state().catalog.episodes_of_podcast(podcast.id).unwrap();
        assert_eq!(episodes[0].url, "https://example.com/moved/g1.mp3");
    }

    #[tokio::test]
    #[serial]
    async fn an_invalid_feed_is_a_bad_request() {
        let _ts = handle_test_startup().await;
        let (sync, feeds, podcast) = sync_fixture();
        feeds.set(FEED, "<html>not a feed</html>".to_string());

        let err = sync.sync(&podcast).unwrap_err();

        assert!(
            matches!(
                err.inner,
                common_infrastructure::error::CustomErrorInner::BadRequest(..)
            ),
            "{err:?}"
        );
        assert!(
            app_state()
                .catalog
                .episodes_of_podcast(podcast.id)
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    #[serial]
    async fn a_feed_that_moved_permanently_is_stored_under_its_new_url() {
        let _ts = handle_test_startup().await;
        let (sync, feeds, podcast) = sync_fixture();
        let moved = "https://example.com/new-home.xml";
        feeds.set(
            moved,
            rss("", &item("g1", "First", "").replace("g1.mp3", "new/g1.mp3")),
        );
        feeds.redirect(FEED, moved, true);

        sync.sync(&podcast).unwrap();

        let stored = app_state().catalog.podcast(podcast.id).unwrap().unwrap();
        assert_eq!(stored.rssfeed, moved);
        let episodes = app_state().catalog.episodes_of_podcast(podcast.id).unwrap();
        assert_eq!(episodes[0].url, "https://example.com/new/g1.mp3");
    }

    #[tokio::test]
    #[serial]
    async fn a_temporary_redirect_keeps_the_feed_url() {
        let _ts = handle_test_startup().await;
        let (sync, feeds, podcast) = sync_fixture();
        let detour = "https://tracker.example.com/feed.xml";
        feeds.set(detour, rss("", &item("g1", "First", "")));
        feeds.redirect(FEED, detour, false);

        sync.sync(&podcast).unwrap();

        let stored = app_state().catalog.podcast(podcast.id).unwrap().unwrap();
        assert_eq!(stored.rssfeed, FEED);
        assert_eq!(
            app_state()
                .catalog
                .episodes_of_podcast(podcast.id)
                .unwrap()
                .len(),
            1
        );
    }

    // ── Subscribing ─────────────────────────────────────────────────────

    use common_infrastructure::error::CustomErrorInner;
    use podfetch_catalog::subscribe::{PodcastStorage, SubscribeRequest, Subscriptions};

    /// Records what the media library is asked to store.
    #[derive(Default)]
    struct Storage {
        directories: Mutex<Vec<bool>>,
        covers: Mutex<Vec<(Uuid, String)>>,
    }

    impl PodcastStorage for Storage {
        fn create_directory(
            &self,
            channel: &rss::Channel,
            directory_id_known: bool,
        ) -> Result<String, CustomError> {
            self.directories.lock().unwrap().push(directory_id_known);
            Ok(format!("podcasts/{}", channel.title()))
        }

        fn store_cover(
            &self,
            podcast: &podfetch_catalog::domain::podcast::Podcast,
        ) -> Result<(), CustomError> {
            self.covers
                .lock()
                .unwrap()
                .push((podcast.id, podcast.image_url.clone()));
            Ok(())
        }
    }

    const NEW_FEED: &str = "https://example.com/new.xml";

    fn subscribe_fixture(limit: u32) -> (Subscriptions, Arc<Feeds>, Arc<Storage>) {
        let feeds = Arc::new(Feeds::default());
        let storage = Arc::new(Storage::default());
        let subscriptions = Subscriptions::new(
            podfetch_persistence::db::database(),
            feeds.clone(),
            storage.clone(),
            limit,
        );
        (subscriptions, feeds, storage)
    }

    fn user(name: &str) -> Uuid {
        podfetch_identity::services::user_auth::service::UserAuthService::new(
            Arc::new(
                podfetch_identity::persistence::adapters::UserAdminRepositoryImpl::new(
                    podfetch_persistence::db::database(),
                ),
            ),
            Arc::new(common_infrastructure::config::EnvironmentService::for_tests()),
        )
        .create_user(name.to_string(), "user".to_string(), None, false)
        .unwrap()
        .id
    }

    fn request(feed_url: &str) -> SubscribeRequest {
        SubscribeRequest {
            feed_url: feed_url.to_string(),
            ..SubscribeRequest::default()
        }
    }

    #[tokio::test]
    #[serial]
    async fn a_feed_is_subscribed_with_what_it_says_about_itself() {
        let _ts = handle_test_startup().await;
        let (subscriptions, feeds, storage) = subscribe_fixture(0);
        let cover = "https://example.com/cover.jpg";
        feeds.set(
            NEW_FEED,
            rss(
                &format!(r#"<itunes:image href="{cover}"/>"#),
                &item("g1", "First", ""),
            ),
        );
        let user = user("subscriber");

        let podcast = subscriptions
            .subscribe(SubscribeRequest {
                added_by: Some(user),
                ..request(NEW_FEED)
            })
            .unwrap();

        assert_eq!(podcast.name, "Sync");
        assert_eq!(podcast.rssfeed, NEW_FEED);
        assert_eq!(podcast.image_url, cover);
        assert_eq!(podcast.original_image_url, cover);
        assert_eq!(podcast.directory_name, "podcasts/Sync");
        assert_eq!(podcast.added_by, Some(user));
        let directory_id: i32 = podcast.directory_id.parse().unwrap();
        assert!((100..10_000_000).contains(&directory_id));
        assert_eq!(*storage.directories.lock().unwrap(), [false]);
        assert_eq!(
            *storage.covers.lock().unwrap(),
            [(podcast.id, cover.to_string())]
        );
        assert_eq!(
            app_state()
                .catalog
                .podcast_by_feed(NEW_FEED)
                .unwrap()
                .map(|p| p.id),
            Some(podcast.id)
        );
        assert!(
            app_state()
                .catalog
                .episodes_of_podcast(podcast.id)
                .unwrap()
                .is_empty(),
            "the episodes come with the first refresh"
        );
    }

    #[tokio::test]
    #[serial]
    async fn what_the_directory_knows_wins_over_the_feed() {
        let _ts = handle_test_startup().await;
        let (subscriptions, feeds, storage) = subscribe_fixture(0);
        feeds.set(NEW_FEED, rss("", ""));
        // Another feed with the same directory id is in the catalog already.
        PodcastService::add_podcast_to_database("old", "42", FEED, "", "podcasts/Mine").unwrap();

        let podcast = subscriptions
            .subscribe(SubscribeRequest {
                title: Some("Mine".to_string()),
                image_url: Some("https://example.com/art.jpg".to_string()),
                directory_id: Some(42),
                ..request(NEW_FEED)
            })
            .unwrap();

        assert_eq!(podcast.name, "Mine");
        assert_eq!(podcast.directory_id, "42");
        assert_eq!(podcast.original_image_url, "https://example.com/art.jpg");
        assert_eq!(*storage.directories.lock().unwrap(), [true]);
    }

    #[tokio::test]
    #[serial]
    async fn a_feed_without_an_image_gets_the_default_one() {
        let _ts = handle_test_startup().await;
        let (subscriptions, feeds, _storage) = subscribe_fixture(0);
        feeds.set(NEW_FEED, rss("", ""));

        let podcast = subscriptions.subscribe(request(NEW_FEED)).unwrap();

        assert!(common_infrastructure::runtime::is_default_image_url(
            &podcast.image_url
        ));
        assert_eq!(podcast.original_image_url, "");
    }

    #[tokio::test]
    #[serial]
    async fn a_feed_already_subscribed_is_a_conflict() {
        let _ts = handle_test_startup().await;
        let (subscriptions, feeds, storage) = subscribe_fixture(0);
        feeds.set(NEW_FEED, rss("", ""));
        subscriptions.subscribe(request(NEW_FEED)).unwrap();

        let err = subscriptions.subscribe(request(NEW_FEED)).unwrap_err();

        assert!(
            matches!(err.inner, CustomErrorInner::Conflict(..)),
            "{err:?}"
        );
        assert_eq!(storage.directories.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    #[serial]
    async fn an_invalid_feed_is_not_subscribed() {
        let _ts = handle_test_startup().await;
        let (subscriptions, feeds, storage) = subscribe_fixture(0);
        feeds.set(NEW_FEED, "<html>not a feed</html>".to_string());

        let err = subscriptions.subscribe(request(NEW_FEED)).unwrap_err();

        assert!(
            matches!(err.inner, CustomErrorInner::BadRequest(..)),
            "{err:?}"
        );
        assert!(
            app_state()
                .catalog
                .podcast_by_feed(NEW_FEED)
                .unwrap()
                .is_none()
        );
        assert!(storage.directories.lock().unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn users_subscribe_within_their_limit() {
        let _ts = handle_test_startup().await;
        let (subscriptions, feeds, _storage) = subscribe_fixture(1);
        feeds.set(NEW_FEED, rss("", ""));
        let user = user("limited");

        subscriptions.ensure_can_add(user, false, 1).unwrap();
        subscriptions
            .subscribe(SubscribeRequest {
                added_by: Some(user),
                ..request(NEW_FEED)
            })
            .unwrap();

        let err = subscriptions.ensure_can_add(user, false, 1).unwrap_err();
        assert!(
            matches!(err.inner, CustomErrorInner::BadRequest(..)),
            "{err:?}"
        );
        subscriptions.ensure_can_add(user, true, 1).unwrap();
        subscriptions
            .ensure_can_add(Uuid::now_v7(), false, 1)
            .unwrap();

        let (closed, _, _) = subscribe_fixture(0);
        let err = closed.ensure_can_add(user, false, 1).unwrap_err();
        assert!(
            matches!(err.inner, CustomErrorInner::Forbidden(..)),
            "{err:?}"
        );
    }
}
