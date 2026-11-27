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

    /// Serves feeds by URL; they can change between refreshes.
    #[derive(Default)]
    struct Feeds(Mutex<HashMap<String, String>>);

    impl Feeds {
        fn set(&self, url: &str, xml: String) {
            self.0.lock().unwrap().insert(url.to_string(), xml);
        }
    }

    impl FeedFetcher for Feeds {
        fn fetch(&self, feed_url: &str) -> Result<FetchedFeed, CustomError> {
            let content = self
                .0
                .lock()
                .unwrap()
                .get(feed_url)
                .cloned()
                .ok_or_else(|| {
                    CustomError::from(common_infrastructure::error::CustomErrorInner::NotFound(
                        common_infrastructure::error::ErrorSeverity::Warning,
                    ))
                })?;
            Ok(FetchedFeed {
                url: feed_url.to_string(),
                content,
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
}
