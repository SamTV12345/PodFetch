use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use crate::app_state::AppState;
use podfetch_persistence::podcast::PodcastEntity as Podcast;
use crate::services::podcast::service::PodcastService;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::OptionalQuery;

use crate::podcast_episode_dto::PodcastEpisodeDto;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
pub use crate::rss::{RSSAPiKey, RSSQuery};
use rss::extension::itunes::{
    ITunesCategory, ITunesCategoryBuilder, ITunesChannelExtension, ITunesChannelExtensionBuilder,
    ITunesItemExtensionBuilder, ITunesOwner, ITunesOwnerBuilder,
};
use rss::{
    Category, CategoryBuilder, Channel, ChannelBuilder, EnclosureBuilder, GuidBuilder, Item,
    ItemBuilder,
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
get,
path="/rss",
responses(
(status = 200, description = "Gets the complete rss feed"))
, tag = "rss")]
pub async fn get_rss_feed(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<RSSQuery>,
    OptionalQuery(api_key): OptionalQuery<RSSAPiKey>,
) -> Result<impl IntoResponse, CustomError> {
    // If http basic is enabled, we need to check if the api key is valid
    if ENVIRONMENT_SERVICE.http_basic || ENVIRONMENT_SERVICE.oidc_configured {
        let api_key = api_key
            .as_ref()
            .and_then(|q| q.api_key.as_deref())
            .ok_or_else(|| CustomError::from(CustomErrorInner::Forbidden(Warning)))?;

        let api_key_exists = state.user_auth_service.is_api_key_valid(api_key);

        if !&api_key_exists {
            return Err(CustomErrorInner::Forbidden(Warning).into());
        }
    }

    let downloaded_episodes = match query {
        Some(q) => match q.top {
            Some(q) => PodcastEpisodeService::find_all_downloaded_podcast_episodes_with_top_k(q)?,
            None => PodcastEpisodeService::find_all_downloaded_podcast_episodes()?,
        },
        None => PodcastEpisodeService::find_all_downloaded_podcast_episodes()?,
    };

    let api_key = api_key.and_then(|c| c.api_key);

    let downloaded_episodes: Vec<PodcastEpisodeDto> = downloaded_episodes
        .into_iter()
        .map(|c| (c, api_key.clone(), None::<FavoritePodcastEpisode>).into())
        .collect();

    let feed_url = add_api_key_to_url(
        format!("{}{}", &ENVIRONMENT_SERVICE.server_url, &"rss"),
        &api_key,
    );
    let itunes_owner = get_itunes_owner("Podfetch", "dev@podfetch.com");
    let category = get_category("Technology".to_string());
    let itunes_ext = ITunesChannelExtensionBuilder::default()
        .owner(Some(itunes_owner))
        .categories(vec![category])
        .explicit(Some("no".to_string()))
        .author(Some("Podfetch".to_string()))
        .keywords(Some("Podcast, RSS, Feed".to_string()))
        .new_feed_url(feed_url.clone())
        .summary(Some("Your local rss feed for your podcasts".to_string()))
        .build();

    let items = get_podcast_items_rss(&downloaded_episodes);

    let channel_builder = ChannelBuilder::default()
        .language("en".to_string())
        .title("Podfetch")
        .link(feed_url)
        .description("Your local rss feed for your podcasts")
        .items(items.clone())
        .clone();

    let channel =
        generate_itunes_extension_conditionally(itunes_ext, channel_builder, None, &api_key);

    let response = Response::builder()
        .header("Content-Type", "application/rss+xml")
        .body(channel.to_string())
        .unwrap();
    Ok(response)
}

fn add_api_key_to_url(url: String, api_key: &Option<String>) -> String {
    if let Some(api_key) = api_key {
        if url.contains('?') {
            return format!("{url}&apiKey={api_key}");
        }
        return format!("{url}?apiKey={api_key}");
    }
    url
}

fn generate_itunes_extension_conditionally(
    mut itunes_ext: ITunesChannelExtension,
    mut channel_builder: ChannelBuilder,
    podcast: Option<Podcast>,
    api_key: &Option<String>,
) -> Channel {
    if let Some(e) = podcast {
        match !e.image_url.is_empty() {
            true => itunes_ext.set_image(add_api_key_to_url(
                ENVIRONMENT_SERVICE.server_url.to_string() + &*e.image_url,
                api_key,
            )),
            false => itunes_ext.set_image(add_api_key_to_url(
                ENVIRONMENT_SERVICE.server_url.to_string() + &*e.original_image_url,
                api_key,
            )),
        }
    }

    channel_builder.itunes_ext(itunes_ext).build()
}

#[utoipa::path(
get,
path="/rss/{id}",
responses(
(status = 200, description = "Gets a specific rss feed"))
, tag = "rss")]
pub async fn get_rss_feed_for_podcast(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    OptionalQuery(api_key): OptionalQuery<RSSAPiKey>,
) -> Result<impl IntoResponse, CustomError> {
    let server_url = ENVIRONMENT_SERVICE.server_url.clone();
    // If http basic is enabled, we need to check if the api key is valid
    if ENVIRONMENT_SERVICE.http_basic || ENVIRONMENT_SERVICE.oidc_configured {
        let api_key = api_key
            .as_ref()
            .and_then(|q| q.api_key.as_deref())
            .ok_or_else(|| CustomError::from(CustomErrorInner::Forbidden(Warning)))?;

        let api_key_exists = state.user_auth_service.is_api_key_valid(api_key);

        if !&api_key_exists {
            return Err(CustomErrorInner::Forbidden(Warning).into());
        }
    }
    let api_key = api_key.and_then(|c| c.api_key);
    let podcast = PodcastService::get_podcast(id)?;

    let downloaded_episodes: Vec<PodcastEpisodeDto> =
        PodcastEpisodeService::find_all_downloaded_podcast_episodes_by_podcast_id(id)?
            .into_iter()
            .map(|c| (c, api_key.clone(), None::<FavoritePodcastEpisode>).into())
            .collect();

    let mut itunes_owner = get_itunes_owner("", "");

    if let Some(author) = podcast.author.clone() {
        itunes_owner = get_itunes_owner(&author, "local@local.com")
    }

    let mut categories: Vec<Category> = vec![];
    if let Some(keyword) = podcast.keywords.clone() {
        let keywords: Vec<String> = keyword.split(',').map(|s| s.to_string()).collect();
        categories = keywords
            .iter()
            .map(|keyword| CategoryBuilder::default().name(keyword).build())
            .collect();
    }

    let keyword_categories = podcast
        .clone()
        .keywords
        .unwrap_or_default()
        .split(',')
        .filter(|keyword| !keyword.is_empty())
        .map(|s| s.to_string())
        .collect();

    let itunes_ext = ITunesChannelExtensionBuilder::default()
        .owner(Some(itunes_owner))
        .categories(get_categories(keyword_categories))
        .explicit(podcast.clone().explicit)
        .author(podcast.clone().author)
        .keywords(podcast.clone().keywords)
        .new_feed_url(add_api_key_to_url(
            format!("{}{}/{}", &server_url, &"rss", &id),
            &api_key,
        ))
        .summary(podcast.summary.clone())
        .build();

    let items = get_podcast_items_rss(&downloaded_episodes);
    let channel_builder = ChannelBuilder::default()
        .language(podcast.clone().language)
        .categories(categories)
        .title(podcast.name.clone())
        .link(add_api_key_to_url(
            format!("{}{}/{}", &server_url, &"rss", &id),
            &api_key,
        ))
        .description(podcast.clone().summary.unwrap_or_default())
        .items(items.clone())
        .clone();

    let channel = generate_itunes_extension_conditionally(
        itunes_ext,
        channel_builder,
        Some(podcast.clone()),
        &api_key,
    );
    let response = Response::builder()
        .header("Content-Type", "application/rss+xml")
        .body(channel.to_string())
        .unwrap();
    Ok(response)
}

fn get_podcast_items_rss(downloaded_episodes: &[PodcastEpisodeDto]) -> Vec<Item> {
    downloaded_episodes
        .iter()
        .map(|episode| {
            let mime_type = get_mime_type_for_episode(&episode.local_url);
            let enclosure = EnclosureBuilder::default()
                .url(&episode.local_url)
                .length(episode.total_time.to_string())
                .mime_type(mime_type)
                .build();

            let itunes_extension = ITunesItemExtensionBuilder::default()
                .duration(Some(episode.total_time.to_string()))
                .image(Some(episode.local_image_url.to_string()))
                .build();

            let guid = GuidBuilder::default()
                .permalink(false)
                .value(&episode.episode_id)
                .build();

            ItemBuilder::default()
                .guid(Some(guid))
                .pub_date(Some(episode.date_of_recording.to_string()))
                .title(Some(episode.name.to_string()))
                .description(Some(episode.description.to_string()))
                .enclosure(Some(enclosure))
                .itunes_ext(itunes_extension)
                .build()
        })
        .collect::<Vec<Item>>()
}

fn get_mime_type_for_episode(url: &str) -> String {
    let extension = PodcastEpisodeService::get_url_file_suffix(url)
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "mp4" | "m4v" | "mov" | "webm" => format!("video/{extension}"),
        "m4a" => "audio/mp4".to_string(),
        "mp3" => "audio/mpeg".to_string(),
        "aac" => "audio/aac".to_string(),
        "ogg" => "audio/ogg".to_string(),
        "wav" => "audio/wav".to_string(),
        _ if extension.is_empty() => "application/octet-stream".to_string(),
        _ => format!("audio/{extension}"),
    }
}

fn get_categories(categories: Vec<String>) -> Vec<ITunesCategory> {
    categories
        .iter()
        .map(|category| get_category(category.to_string()))
        .collect::<Vec<ITunesCategory>>()
}

fn get_category(category: String) -> ITunesCategory {
    ITunesCategoryBuilder::default().text(category).build()
}

fn get_itunes_owner(name: &str, email: &str) -> ITunesOwner {
    ITunesOwnerBuilder::default()
        .name(Some(name.to_string()))
        .email(Some(email.to_string()))
        .build()
}

pub fn get_websocket_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_rss_feed))
        .routes(routes!(get_rss_feed_for_podcast))
}

#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::test_support::tests::handle_test_startup;
    use podfetch_persistence::podcast::PodcastEntity as Podcast;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use serial_test::serial;
    use uuid::Uuid;

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    fn create_api_key_user() -> String {
        let state = AppState::new();
        let mut user = UserTestDataBuilder::new().build();
        user.username = format!("rss-test-user-{}", Uuid::new_v4());
        let api_key = format!("rss-test-key-{}", Uuid::new_v4());
        user.api_key = Some(api_key.clone());
        let _created = state.user_admin_service.create_user(user).unwrap();
        api_key
    }

    fn with_api_key(path: &str, key: &str) -> String {
        if path.contains('?') {
            format!("{path}&apiKey={key}")
        } else {
            format!("{path}?apiKey={key}")
        }
    }

    fn create_podcast_for_rss() -> Podcast {
        let unique = Uuid::new_v4().to_string();
        let slug = format!("rss-podcast-{unique}");
        crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("RSS Podcast {unique}"),
            &slug,
            &format!("https://example.com/{slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &slug,
        )
        .unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_returns_xml_or_forbidden_when_auth_is_enforced() {
        let server = handle_test_startup().await;
        let api_key = create_api_key_user();
        let request_path = with_api_key("/rss", &api_key);

        let response = server.test_server.get(&request_path).await;
        let status = response.status_code();
        assert!(status == 200 || status == 403);
        if status == 200 {
            assert_eq!(
                response.maybe_content_type().unwrap(),
                "application/rss+xml"
            );
            let body = response.text();
            assert!(body.contains("<rss"));
            assert!(body.contains("<channel>"));
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_rss_endpoints_return_client_error_for_wrong_http_method() {
        let server = handle_test_startup().await;

        let post_rss_response = server.test_server.post("/rss").await;
        assert_client_error_status(post_rss_response.status_code().as_u16());

        let post_rss_id_response = server.test_server.post("/rss/1").await;
        assert_client_error_status(post_rss_id_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_rejects_invalid_top_query() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/rss?top=abc").await;
        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_accepts_valid_top_query() {
        let server = handle_test_startup().await;
        let api_key = create_api_key_user();
        let request_path = with_api_key("/rss?top=1", &api_key);

        let response = server.test_server.get(&request_path).await;
        let status = response.status_code();
        assert!(status == 200 || status == 403);
        if status == 200 {
            assert_eq!(
                response.maybe_content_type().unwrap(),
                "application/rss+xml"
            );
            let body = response.text();
            assert!(body.contains("<rss"));
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_for_podcast_rejects_non_numeric_id() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/rss/not-a-number").await;
        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_for_unknown_podcast_returns_not_found() {
        let server = handle_test_startup().await;
        let api_key = create_api_key_user();
        let request_path = with_api_key("/rss/999999", &api_key);

        let response = server.test_server.get(&request_path).await;
        let status = response.status_code();
        assert!(status == 404 || status == 403);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_for_existing_podcast_returns_xml_or_forbidden() {
        let server = handle_test_startup().await;
        let podcast = create_podcast_for_rss();
        let api_key = create_api_key_user();
        let request_path = with_api_key(&format!("/rss/{}", podcast.id), &api_key);

        let response = server.test_server.get(&request_path).await;
        let status = response.status_code();
        assert!(status == 200 || status == 403);
        if status == 200 {
            assert_eq!(
                response.maybe_content_type().unwrap(),
                "application/rss+xml"
            );
            let body = response.text();
            assert!(body.contains("<rss"));
            assert!(body.contains("<channel>"));
        }
    }

    #[test]
    fn test_add_api_key_to_url_variants() {
        assert_eq!(
            super::add_api_key_to_url("https://example.com/rss".to_string(), &None),
            "https://example.com/rss"
        );
        assert_eq!(
            super::add_api_key_to_url(
                "https://example.com/rss".to_string(),
                &Some("k1".to_string())
            ),
            "https://example.com/rss?apiKey=k1"
        );
        assert_eq!(
            super::add_api_key_to_url(
                "https://example.com/rss?top=1".to_string(),
                &Some("k1".to_string())
            ),
            "https://example.com/rss?top=1&apiKey=k1"
        );
    }

    #[test]
    fn test_get_mime_type_for_episode_mappings() {
        assert_eq!(
            super::get_mime_type_for_episode("https://example.com/file.MP3"),
            "audio/mpeg"
        );
        assert_eq!(
            super::get_mime_type_for_episode("https://example.com/file.m4a"),
            "audio/mp4"
        );
        assert_eq!(
            super::get_mime_type_for_episode("https://example.com/file.mp4"),
            "video/mp4"
        );
        assert_eq!(
            super::get_mime_type_for_episode("https://example.com/file.abc"),
            "audio/abc"
        );
        assert_eq!(
            super::get_mime_type_for_episode("https://example.com/file"),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_generate_itunes_extension_conditionally_prefers_image_url() {
        let mut podcast = Podcast::default();
        podcast.image_url = "ui/custom.png".to_string();
        podcast.original_image_url = "ui/original.png".to_string();

        let itunes_ext = super::ITunesChannelExtensionBuilder::default().build();
        let channel_builder = super::ChannelBuilder::default()
            .title("t")
            .link("l")
            .description("d")
            .items(vec![])
            .clone();

        let channel = super::generate_itunes_extension_conditionally(
            itunes_ext,
            channel_builder,
            Some(podcast),
            &Some("k1".to_string()),
        );

        let xml = channel.to_string();
        assert!(xml.contains("ui/custom.png?apiKey=k1"));
        assert!(!xml.contains("ui/original.png?apiKey=k1"));
    }

    #[test]
    fn test_generate_itunes_extension_conditionally_falls_back_to_original_image_url() {
        let mut podcast = Podcast::default();
        podcast.image_url = "".to_string();
        podcast.original_image_url = "ui/original.png".to_string();

        let itunes_ext = super::ITunesChannelExtensionBuilder::default().build();
        let channel_builder = super::ChannelBuilder::default()
            .title("t")
            .link("l")
            .description("d")
            .items(vec![])
            .clone();

        let channel = super::generate_itunes_extension_conditionally(
            itunes_ext,
            channel_builder,
            Some(podcast),
            &Some("k1".to_string()),
        );

        let xml = channel.to_string();
        assert!(xml.contains("ui/original.png?apiKey=k1"));
    }

    #[test]
    fn test_get_categories_returns_same_number_as_input() {
        let categories = vec![
            "Technology".to_string(),
            "Science".to_string(),
            "Education".to_string(),
        ];
        let mapped = super::get_categories(categories);
        assert_eq!(mapped.len(), 3);
    }
}




