use crate::app_state::AppState;
use crate::controllers::id_resolver::{ResolvedId, parse_resolved_id};
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::OptionalQuery;
use podfetch_persistence::podcast::PodcastEntity as Podcast;

use crate::podcast_episode_dto::PodcastEpisodeDto;
pub use crate::rss::{RSSAPiKey, RSSQuery};
use crate::url_rewriting::resolve_server_url_from_headers;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_domain::podcast_episode_transcript::TranscriptStatus;
use rss::extension::itunes::{
    ITunesCategory, ITunesCategoryBuilder, ITunesChannelExtension, ITunesChannelExtensionBuilder,
    ITunesItemExtensionBuilder, ITunesOwner, ITunesOwnerBuilder,
};
use rss::extension::{Extension, ExtensionBuilder};
use rss::{
    Category, CategoryBuilder, Channel, ChannelBuilder, EnclosureBuilder, GuidBuilder, Item,
    ItemBuilder, SourceBuilder,
};
use std::collections::{BTreeMap, HashMap};

/// Namespace declared on generated channels so `<podcast:transcript>` items
/// are valid Podcasting 2.0 markup.
const PODCAST_NAMESPACE_URL: &str = "https://podcastindex.org/namespace/1.0";

fn podcast_namespace() -> BTreeMap<String, String> {
    BTreeMap::from([("podcast".to_string(), PODCAST_NAMESPACE_URL.to_string())])
}
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
    headers: HeaderMap,
    OptionalQuery(query): OptionalQuery<RSSQuery>,
    OptionalQuery(api_key): OptionalQuery<RSSAPiKey>,
) -> Result<impl IntoResponse, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
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
        .map(|c| {
            PodcastEpisodeDto::from_episode_with_api_key(
                c,
                api_key.clone(),
                None::<FavoritePodcastEpisode>,
                &server_url,
            )
        })
        .collect();

    let feed_url = add_api_key_to_url(format!("{server_url}rss"), &api_key);
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

    // Map every episode's podcast id to its show so each item can name its
    // originating podcast in the aggregated feed (#2055).
    let podcast_by_id: HashMap<String, Podcast> = PodcastService::get_all_podcasts_raw()?
        .into_iter()
        .map(|p| (p.id.clone(), p))
        .collect();

    let items = get_podcast_items_rss(
        &state,
        &downloaded_episodes,
        &api_key,
        &server_url,
        Some(&podcast_by_id),
    );

    let channel_builder = ChannelBuilder::default()
        .namespaces(podcast_namespace())
        .language("en".to_string())
        .title("Podfetch")
        .link(feed_url)
        .description("Your local rss feed for your podcasts")
        .items(items.clone())
        .clone();

    let channel = generate_itunes_extension_conditionally(
        itunes_ext,
        channel_builder,
        None,
        &api_key,
        &server_url,
    );

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
    server_url: &str,
) -> Channel {
    if let Some(e) = podcast {
        match !e.image_url.is_empty() {
            true => itunes_ext.set_image(add_api_key_to_url(
                server_url.to_string() + &*e.image_url,
                api_key,
            )),
            false => itunes_ext.set_image(add_api_key_to_url(
                server_url.to_string() + &*e.original_image_url,
                api_key,
            )),
        }
    }

    channel_builder.itunes_ext(itunes_ext).build()
}

/// Resolve a podcast `{id}` path segment (UUID or legacy integer) to the
/// canonical podcast `Uuid`.
fn resolve_podcast_uuid(id: &str) -> Result<uuid::Uuid, CustomError> {
    match parse_resolved_id(id)? {
        ResolvedId::Uuid(uuid) => Ok(uuid),
        ResolvedId::Legacy(legacy) => {
            let podcast = PodcastService::get_podcast_by_legacy_id(legacy)?;
            uuid::Uuid::parse_str(&podcast.id)
                .map_err(|_| CustomErrorInner::NotFound(Warning).into())
        }
    }
}

#[utoipa::path(
get,
path="/rss/{id}",
responses(
(status = 200, description = "Gets a specific rss feed"))
, tag = "rss")]
pub async fn get_rss_feed_for_podcast(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    OptionalQuery(api_key): OptionalQuery<RSSAPiKey>,
) -> Result<impl IntoResponse, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
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
    let podcast_uuid = resolve_podcast_uuid(&id)?;
    let podcast = PodcastService::get_podcast(podcast_uuid)?;

    let downloaded_episodes: Vec<PodcastEpisodeDto> =
        PodcastEpisodeService::find_all_downloaded_podcast_episodes_by_podcast_id(podcast_uuid)?
            .into_iter()
            .map(|c| {
                PodcastEpisodeDto::from_episode_with_api_key(
                    c,
                    api_key.clone(),
                    None::<FavoritePodcastEpisode>,
                    &server_url,
                )
            })
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
            format!("{server_url}rss/{id}"),
            &api_key,
        ))
        .summary(podcast.summary.clone())
        .build();

    // Per-podcast feed: the channel title already is the show name, so no
    // per-item source is needed.
    let items = get_podcast_items_rss(&state, &downloaded_episodes, &api_key, &server_url, None);
    let channel_builder = ChannelBuilder::default()
        .namespaces(podcast_namespace())
        .language(podcast.clone().language)
        .categories(categories)
        .title(podcast.name.clone())
        .link(add_api_key_to_url(
            format!("{server_url}rss/{id}"),
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
        &server_url,
    );
    let response = Response::builder()
        .header("Content-Type", "application/rss+xml")
        .body(channel.to_string())
        .unwrap();
    Ok(response)
}

fn get_podcast_items_rss(
    state: &AppState,
    downloaded_episodes: &[PodcastEpisodeDto],
    api_key: &Option<String>,
    server_url: &str,
    podcast_by_id: Option<&HashMap<String, Podcast>>,
) -> Vec<Item> {
    downloaded_episodes
        .iter()
        .map(|episode| {
            let mime_type = get_mime_type_for_episode(&episode.local_url);
            let enclosure = EnclosureBuilder::default()
                .url(&episode.local_url)
                .length(episode.total_time.to_string())
                .mime_type(mime_type)
                .build();

            // In the aggregated feed every item shares the "Podfetch" channel,
            // so readers can't tell which show an episode belongs to. Tag each
            // item with its originating podcast via the standard RSS <source>
            // element and the itunes author, without touching the title (#2055).
            let podcast = podcast_by_id.and_then(|m| m.get(&episode.podcast_id));

            let itunes_extension = ITunesItemExtensionBuilder::default()
                .duration(Some(episode.total_time.to_string()))
                .image(Some(episode.local_image_url.to_string()))
                .author(podcast.map(|p| p.name.clone()))
                .build();

            let guid = GuidBuilder::default()
                .permalink(false)
                .value(&episode.episode_id)
                .build();

            let source = podcast.map(|p| {
                SourceBuilder::default()
                    .url(add_api_key_to_url(
                        format!("{server_url}rss/{}", p.id),
                        api_key,
                    ))
                    .title(Some(p.name.clone()))
                    .build()
            });

            let mut item = ItemBuilder::default()
                .guid(Some(guid))
                .pub_date(Some(episode.date_of_recording.to_string()))
                .title(Some(episode.name.to_string()))
                .description(Some(episode.description.to_string()))
                // Without a <link> readers fall back to showing the opaque guid;
                // point it at the episode's page in the podfetch UI (#2055).
                .link(Some(format!(
                    "{server_url}ui/podcasts/{}/episodes",
                    episode.podcast_id
                )))
                .enclosure(Some(enclosure))
                .itunes_ext(itunes_extension)
                .source(source)
                .build();

            attach_transcript_extensions(state, episode, api_key, server_url, &mut item);
            item
        })
        .collect::<Vec<Item>>()
}

/// Adds one `<podcast:transcript>` extension per archived transcript of the
/// episode. Transcript lookup failures are non-fatal for feed generation —
/// the item is simply exported without transcript tags.
fn attach_transcript_extensions(
    state: &AppState,
    episode: &PodcastEpisodeDto,
    api_key: &Option<String>,
    server_url: &str,
    item: &mut Item,
) {
    let Ok(episode_uuid) = uuid::Uuid::parse_str(&episode.id) else {
        return;
    };

    let transcripts = match state.transcript_service.get_by_episode_id(episode_uuid) {
        Ok(transcripts) => transcripts,
        Err(err) => {
            tracing::error!(
                "Error loading transcripts for rss item {}: {err}",
                episode.id
            );
            return;
        }
    };

    let extensions: Vec<Extension> = transcripts
        .iter()
        .filter(|t| {
            matches!(
                t.status,
                TranscriptStatus::Parsed | TranscriptStatus::Downloaded
            ) && t.file_path.is_some()
        })
        .map(|t| {
            // Feed readers fetch this URL without a login session, so it has
            // to be the apiKey-in-path file route whenever a key is present.
            let file_url = match api_key {
                Some(key) => format!(
                    "{server_url}api/v1/podcasts/episodes/{}/transcripts/{}/file/apiKey/{key}",
                    episode.id, t.id
                ),
                None => format!(
                    "{server_url}api/v1/podcasts/episodes/{}/transcripts/{}/file",
                    episode.id, t.id
                ),
            };

            let mut attrs = BTreeMap::new();
            attrs.insert("url".to_string(), file_url);
            attrs.insert("type".to_string(), t.mime_type.clone());
            if let Some(language) = &t.language {
                attrs.insert("language".to_string(), language.clone());
            }

            ExtensionBuilder::default()
                .name("podcast:transcript")
                .attrs(attrs)
                .build()
        })
        .collect();

    if !extensions.is_empty() {
        let mut extension_map = item.extensions().clone();
        extension_map
            .entry("podcast".to_string())
            .or_default()
            .insert("transcript".to_string(), extensions);
        item.set_extensions(extension_map);
    }
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

#[utoipa::path(
get,
path="/rss/apiKey/{apiKey}",
responses(
(status = 200, description = "Gets the complete rss feed (API key in path)"))
, tag = "rss")]
pub async fn get_rss_feed_with_path_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(api_key): Path<String>,
    OptionalQuery(query): OptionalQuery<RSSQuery>,
) -> Result<impl IntoResponse, CustomError> {
    let api_key_query = OptionalQuery(Some(RSSAPiKey {
        api_key: Some(api_key),
    }));
    get_rss_feed(State(state), headers, OptionalQuery(query), api_key_query).await
}

#[utoipa::path(
get,
path="/rss/apiKey/{apiKey}/{id}",
responses(
(status = 200, description = "Gets a specific rss feed (API key in path)"))
, tag = "rss")]
pub async fn get_rss_feed_for_podcast_with_path_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((api_key, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, CustomError> {
    let api_key_query = OptionalQuery(Some(RSSAPiKey {
        api_key: Some(api_key),
    }));
    get_rss_feed_for_podcast(State(state), headers, Path(id), api_key_query).await
}

pub fn get_websocket_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_rss_feed))
        .routes(routes!(get_rss_feed_for_podcast))
        .routes(routes!(get_rss_feed_with_path_api_key))
        .routes(routes!(get_rss_feed_for_podcast_with_path_api_key))
}

#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use podfetch_persistence::db::get_connection;
    use podfetch_persistence::podcast::PodcastEntity as Podcast;
    use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
    use podfetch_persistence::schema::podcast_episodes::dsl as pe_dsl;
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

    fn insert_downloaded_episode(
        podcast_id: &str,
        episode_id: &str,
        guid: &str,
        file_episode_path: &str,
        file_image_path: &str,
    ) -> PodcastEpisode {
        use diesel::ExpressionMethods;
        use diesel::RunQueryDsl;

        diesel::insert_into(pe_dsl::podcast_episodes)
            .values((
                pe_dsl::id.eq(Uuid::new_v4().to_string()),
                pe_dsl::podcast_id.eq(podcast_id.to_string()),
                pe_dsl::episode_id.eq(episode_id.to_string()),
                pe_dsl::name.eq("RSS Rewrite Episode".to_string()),
                pe_dsl::url.eq(format!("https://example.com/{episode_id}.mp3")),
                pe_dsl::date_of_recording.eq("2026-03-01T00:00:00Z".to_string()),
                pe_dsl::image_url.eq("https://example.com/image.jpg".to_string()),
                pe_dsl::total_time.eq(1800),
                pe_dsl::description.eq("rss rewrite test".to_string()),
                pe_dsl::guid.eq(guid.to_string()),
                pe_dsl::deleted.eq(false),
                pe_dsl::episode_numbering_processed.eq(false),
                pe_dsl::file_episode_path.eq(Some(file_episode_path.to_string())),
                pe_dsl::file_image_path.eq(Some(file_image_path.to_string())),
                pe_dsl::download_location.eq(Some("Local".to_string())),
            ))
            .get_result::<PodcastEpisode>(&mut get_connection())
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

    #[tokio::test]
    #[serial]
    async fn test_rss_feed_for_podcast_includes_podcast_transcript_tag_for_archived_transcript() {
        use podfetch_domain::podcast_episode_transcript::{
            PodcastEpisodeTranscriptRepository, TranscriptSource, TranscriptStatus,
            UpsertTranscript,
        };
        use podfetch_persistence::adapters::PodcastEpisodeTranscriptRepositoryImpl;
        use podfetch_persistence::db::database;

        let server = handle_test_startup().await;
        let podcast = create_podcast_for_rss();
        let api_key = create_api_key_user();
        let unique = Uuid::new_v4();
        let episode = insert_downloaded_episode(
            &podcast.id.to_string(),
            &format!("rss-transcript-ep-{unique}"),
            &format!("rss-transcript-guid-{unique}"),
            &format!("podcasts/rss-transcript-{unique}/episode.mp3"),
            &format!("podcasts/rss-transcript-{unique}/image.jpg"),
        );

        let episode_uuid = Uuid::parse_str(&episode.id).unwrap();
        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let transcript_id = repo
            .upsert(UpsertTranscript {
                episode_id: episode_uuid,
                source: TranscriptSource::Feed,
                original_url: Some(format!("https://example.com/{unique}.vtt")),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();
        repo.set_status(transcript_id, TranscriptStatus::Parsed, None)
            .unwrap();
        repo.set_file_path(transcript_id, "/tmp/rss-transcript-test.vtt")
            .unwrap();

        let request_path = with_api_key(&format!("/rss/{}", podcast.id), &api_key);
        let response = server.test_server.get(&request_path).await;
        let status = response.status_code();
        if status != 200 {
            // Skip when auth is enforced in this environment
            assert_eq!(status, 403);
            return;
        }

        let body = response.text();
        assert!(
            body.contains(r#"xmlns:podcast="https://podcastindex.org/namespace/1.0""#),
            "channel must declare the podcast namespace, got: {body}"
        );
        assert!(
            body.contains("<podcast:transcript"),
            "feed must contain a podcast:transcript tag, got: {body}"
        );
        let expected_url_fragment = format!(
            "/api/v1/podcasts/episodes/{}/transcripts/{}/file/apiKey/{}",
            episode.id, transcript_id, api_key
        );
        assert!(
            body.contains(&expected_url_fragment),
            "transcript url must point at the apiKey file route, got: {body}"
        );
        assert!(body.contains(r#"type="text/vtt""#));
    }

    #[tokio::test]
    #[serial]
    async fn test_rss_feed_omits_transcript_tag_for_unarchived_transcript() {
        use podfetch_domain::podcast_episode_transcript::{
            PodcastEpisodeTranscriptRepository, TranscriptSource, UpsertTranscript,
        };
        use podfetch_persistence::adapters::PodcastEpisodeTranscriptRepositoryImpl;
        use podfetch_persistence::db::database;

        let server = handle_test_startup().await;
        let podcast = create_podcast_for_rss();
        let api_key = create_api_key_user();
        let unique = Uuid::new_v4();
        let episode = insert_downloaded_episode(
            &podcast.id.to_string(),
            &format!("rss-transcript-pending-ep-{unique}"),
            &format!("rss-transcript-pending-guid-{unique}"),
            &format!("podcasts/rss-transcript-pending-{unique}/episode.mp3"),
            &format!("podcasts/rss-transcript-pending-{unique}/image.jpg"),
        );

        // Pending transcript without an archived file must not be exported.
        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        repo.upsert(UpsertTranscript {
            episode_id: Uuid::parse_str(&episode.id).unwrap(),
            source: TranscriptSource::Feed,
            original_url: Some(format!("https://example.com/{unique}.vtt")),
            mime_type: "text/vtt".to_string(),
            language: Some("en".to_string()),
        })
        .unwrap();

        let request_path = with_api_key(&format!("/rss/{}", podcast.id), &api_key);
        let response = server.test_server.get(&request_path).await;
        let status = response.status_code();
        if status != 200 {
            assert_eq!(status, 403);
            return;
        }

        assert!(!response.text().contains("<podcast:transcript"));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_for_podcast_rewrites_episode_urls_using_forwarded_headers() {
        let mut server = handle_test_startup().await;
        server
            .test_server
            .add_header("x-forwarded-host", "podfetch.example.com");
        server.test_server.add_header("x-forwarded-proto", "https");

        let podcast = create_podcast_for_rss();
        let api_key = create_api_key_user();
        let unique = Uuid::new_v4();
        let file_episode_path = format!("podcasts/rss-rewrite-{unique}/episode.mp3");
        let file_image_path = format!("podcasts/rss-rewrite-{unique}/image.jpg");
        let _episode = insert_downloaded_episode(
            &podcast.id.to_string(),
            &format!("rss-rewrite-ep-{unique}"),
            &format!("rss-rewrite-guid-{unique}"),
            &file_episode_path,
            &file_image_path,
        );

        let request_path = with_api_key(&format!("/rss/{}", podcast.id), &api_key);
        let response = server.test_server.get(&request_path).await;
        let status = response.status_code();
        if status != 200 {
            // Skip when auth is enforced in this environment
            assert_eq!(status, 403);
            return;
        }

        let body = response.text();
        let expected_episode_url =
            format!("https://podfetch.example.com/{file_episode_path}");
        let expected_image_url = format!("https://podfetch.example.com/{file_image_path}");
        assert!(
            body.contains(&expected_episode_url),
            "expected rewritten enclosure URL {expected_episode_url:?} in body, got: {body}"
        );
        assert!(
            body.contains(&expected_image_url),
            "expected rewritten image URL {expected_image_url:?} in body, got: {body}"
        );
        assert!(
            !body.contains(&format!("http://localhost:8000/{file_episode_path}")),
            "body should not contain internal localhost URL for the episode, got: {body}"
        );
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
        let podcast = Podcast {
            image_url: "ui/custom.png".to_string(),
            original_image_url: "ui/original.png".to_string(),
            ..Default::default()
        };

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
            "http://localhost:8000/",
        );

        let xml = channel.to_string();
        assert!(xml.contains("ui/custom.png?apiKey=k1"));
        assert!(!xml.contains("ui/original.png?apiKey=k1"));
    }

    #[test]
    fn test_generate_itunes_extension_conditionally_falls_back_to_original_image_url() {
        let podcast = Podcast {
            image_url: String::new(),
            original_image_url: "ui/original.png".to_string(),
            ..Default::default()
        };

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
            "http://localhost:8000/",
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

    // ── Path-based API key tests ────────────────────────────────────────

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_with_path_api_key_returns_xml() {
        let server = handle_test_startup().await;
        let api_key = create_api_key_user();

        let response = server
            .test_server
            .get(&format!("/rss/apiKey/{api_key}"))
            .await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(
            response.maybe_content_type().unwrap(),
            "application/rss+xml"
        );
        let body = response.text();
        assert!(body.contains("<rss"));
        assert!(body.contains("<channel>"));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_with_path_api_key_rejects_invalid_key() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .get("/rss/apiKey/invalid-key-that-does-not-exist")
            .await;
        assert_eq!(response.status_code(), 403);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_with_path_api_key_supports_top_query() {
        let server = handle_test_startup().await;
        let api_key = create_api_key_user();

        let response = server
            .test_server
            .get(&format!("/rss/apiKey/{api_key}?top=1"))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.text();
        assert!(body.contains("<rss"));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_for_podcast_with_path_api_key_returns_xml() {
        let server = handle_test_startup().await;
        let podcast = create_podcast_for_rss();
        let api_key = create_api_key_user();

        let response = server
            .test_server
            .get(&format!("/rss/apiKey/{api_key}/{}", podcast.id))
            .await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(
            response.maybe_content_type().unwrap(),
            "application/rss+xml"
        );
        let body = response.text();
        assert!(body.contains("<rss"));
        assert!(body.contains(&podcast.name));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_for_podcast_with_path_api_key_rejects_invalid_key() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/rss/apiKey/invalid-key/1").await;
        assert_eq!(response.status_code(), 403);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_rss_feed_for_podcast_with_path_api_key_returns_not_found_for_unknown_id() {
        let server = handle_test_startup().await;
        let api_key = create_api_key_user();

        let response = server
            .test_server
            .get(&format!("/rss/apiKey/{api_key}/999999"))
            .await;
        assert_eq!(response.status_code(), 404);
    }
}
