use crate::app_state::AppState;
use crate::background::spawn_detached;
use crate::controllers::controller_utils::{get_default_image, unwrap_string};
use crate::controllers::id_resolver::{ResolvedId, parse_resolved_id};
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json, debug_handler};
use axum_extra::extract::OptionalQuery;
use common_infrastructure::config::{BASIC_AUTH, OIDC_AUTH};
use common_infrastructure::http::COMMON_USER_AGENT;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use opml::{OPML, Outline};
use rand::RngExt;
use reqwest::header::HeaderValue;
use rss::Channel;
use serde_json::Value;
use std::sync::LazyLock;
use std::thread;

use crate::filter::Filter;
pub use crate::podcast::{
    DeletePodcast, OpmlModel, PodcastAddModel, PodcastFavorUpdateModel, PodcastInsertModel,
    PodcastRSSAddModel, PodcastSearchModelUtoipa, PodcastSearchReturn, PodcastUpdateNameRequest,
    ProxyPodcastParams,
    SearchType::{ITunes, Podindex},
};
use crate::podcast::{
    build_podcast_search_plan, ensure_podindex_configured, ensure_proxy_api_access,
    map_podcast_error, map_proxy_podcast_error, parse_search_type, require_admin,
    require_privileged, require_proxy_episode, sanitize_proxy_request_headers,
};
use crate::services::file::service::{FileService, perform_podcast_variable_replacement};
use crate::url_rewriting::resolve_server_url_from_headers;
use common_infrastructure::config::is_env_var_present_and_true;
use common_infrastructure::http::get_http_client;
use common_infrastructure::request::add_basic_auth_headers_conditionally;
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_domain::user::User;
use reqwest::header::HeaderMap as ReqwestHeaderMap;

/// Client for streaming remote episodes through PodFetch.
static PROXY_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(50))
        .build()
        .expect("static reqwest client configuration is valid")
});

/// Resolve a `{id}` podcast path segment (UUID or legacy integer) to the
/// loaded podcast, returning the canonical `Uuid` alongside it.
fn resolve_podcast(
    id: &str,
) -> Result<(uuid::Uuid, podfetch_persistence::podcast::PodcastEntity), CustomError> {
    let podcast = match parse_resolved_id(id)? {
        ResolvedId::Uuid(uuid) => PodcastService::get_podcast(uuid)?,
        ResolvedId::Legacy(legacy) => PodcastService::get_podcast_by_legacy_id(legacy)?,
    };
    let uuid = uuid::Uuid::parse_str(&podcast.id)
        .map_err(|_| CustomErrorInner::NotFound(ErrorSeverity::Warning))?;
    Ok((uuid, podcast))
}

#[utoipa::path(
get,
path="/podcasts/filter",
responses(
(status = 200, description = "Gets the user specific filter.",body= Filter)),
tag="podcasts"
)]
pub async fn get_filter(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
) -> Result<Json<Filter>, CustomError> {
    let filter = state
        .filter_service
        .get_filter_by_user_id(requester.id)?
        .unwrap_or(crate::filter::default_podcast_filter());
    Ok(Json(filter))
}

#[utoipa::path(
get,
path="/podcasts/reverse/episode/{id}",
    responses(
(status = 200, description = "Does a reverse search of an episode id to find out the podcast it \
belongs to",body=
Vec<PodcastDto>)),
    tag="podcasts"
)]
pub async fn search_podcast_of_episode(
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<PodcastDto>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
    let podcast = match parse_resolved_id(&id)? {
        ResolvedId::Uuid(uuid) => PodcastService::get_podcast_by_episode_id(uuid)?,
        ResolvedId::Legacy(legacy) => {
            let episode = PodcastEpisodeService::get_podcast_episode_by_legacy_id(legacy)?
                .ok_or_else(|| CustomErrorInner::NotFound(ErrorSeverity::Warning))?;
            let episode_uuid = uuid::Uuid::parse_str(&episode.id)
                .map_err(|_| CustomErrorInner::NotFound(ErrorSeverity::Warning))?;
            PodcastService::get_podcast_by_episode_id(episode_uuid)?
        }
    };
    let dto = map_podcast_to_dto(podcast.into(), &server_url);
    Ok(Json(dto))
}

#[utoipa::path(
get,
path="/podcasts/search",
params(PodcastSearchModelUtoipa),
responses(
(status = 200, description = "Gets the podcasts matching the searching criteria",body=
Vec<PodcastDto>)),
tag="podcasts"
)]
pub async fn search_podcasts(
    State(state): State<AppState>,
    Query(query): Query<PodcastSearchModelUtoipa>,
    Extension(requester): Extension<User>,
    headers: HeaderMap,
) -> Result<Json<Vec<PodcastDto>>, CustomError> {
    let existing_filter = state.filter_service.get_filter_by_user_id(requester.id)?;
    let search_plan = build_podcast_search_plan(query, requester.id, existing_filter);
    state.filter_service.save_filter(search_plan.filter)?;
    let server_url = resolve_server_url_from_headers(&headers);

    if search_plan.favored_only {
        let podcasts = PodcastService::search_podcasts_favored(
            search_plan.order,
            search_plan.title,
            search_plan.order_option,
            search_plan.user_id,
            search_plan.tag,
            &requester,
            &server_url,
        )?;
        Ok(Json(podcasts))
    } else {
        let podcasts = PodcastService::search_podcasts(
            search_plan.order,
            search_plan.title,
            search_plan.order_option,
            search_plan.tag,
            &requester,
            &server_url,
        )?;
        Ok(Json(podcasts))
    }
}

#[utoipa::path(
get,
path="/podcasts/{id}",
responses(
(status = 200, description = "Find a podcast by its collection id", body = PodcastDto)
),
tag="podcasts"
)]
pub async fn find_podcast_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(user): Extension<User>,
    headers: HeaderMap,
) -> Result<Json<PodcastDto>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);

    let (podcast_uuid, podcast) = resolve_podcast(&id)?;
    let tags = state
        .tag_service
        .get_tags_of_podcast(podcast_uuid, user.id)?;
    let favorite = PodcastService::get_favorite_state(user.id, podcast_uuid)?;
    let podcast_dto =
        map_podcast_with_context_to_dto(podcast.into(), favorite, tags, &user, &server_url);
    Ok(Json(podcast_dto))
}

#[utoipa::path(
get,
path="/podcasts",
responses(
(status = 200, description = "Gets all stored podcasts as a list", body = [PodcastDto])
),
tag="podcasts"
)]
pub async fn find_all_podcasts(
    Extension(requester): Extension<User>,
    headers: HeaderMap,
) -> Result<Json<Vec<PodcastDto>>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
    let podcasts = PodcastService::get_podcasts(&requester, &server_url)?;

    Ok(Json(podcasts))
}

#[utoipa::path(
get,
path="/podcasts/{type_of}/{podcast}/search",
responses(
(status = 200, description = "Finds a podcast from the itunes url.", body = PodcastSearchReturn)
),
tag="podcasts"
)]
pub async fn find_podcast(
    Path(podcast_col): Path<(i32, String)>,
    Extension(requester): Extension<User>,
) -> Result<Json<PodcastSearchReturn>, CustomError> {
    require_privileged::<CustomError>(requester.is_privileged_user()).map_err(map_podcast_error)?;

    let (type_of, podcast) = podcast_col;
    match parse_search_type::<CustomError>(type_of).map_err(map_podcast_error)? {
        ITunes => {
            let res;
            {
                tracing::debug!("Searching for podcast: {podcast}");
                res = PodcastService::find_podcast(&podcast).await;
            }
            Ok(Json(PodcastSearchReturn::Itunes(res)))
        }
        Podindex => {
            ensure_podindex_configured::<CustomError>(
                !ENVIRONMENT_SERVICE.podindex_api_key.is_empty()
                    && !ENVIRONMENT_SERVICE.podindex_api_secret.is_empty(),
            )
            .map_err(map_podcast_error)?;

            Ok(Json(PodcastSearchReturn::Podindex(
                PodcastService::find_podcast_on_podindex(&podcast).await?,
            )))
        }
    }
}

#[utoipa::path(
post,
path="/podcasts/itunes",
request_body=PodcastAddModel,
responses(
(status = 200, description = "Adds a podcast to the database.", body = PodcastDto)),
tag="podcasts"
)]
#[debug_handler]
pub async fn add_podcast(
    Extension(requester): Extension<User>,
    headers: HeaderMap,
    Json(track_id): Json<PodcastAddModel>,
) -> Result<Json<PodcastDto>, CustomError> {
    PodcastService::ensure_can_add_podcasts(&requester, 1)?;

    let query: Vec<(&str, String)> = vec![
        ("id", track_id.track_id.to_string()),
        ("entity", "podcast".to_string()),
    ];

    let res = get_http_client(&ENVIRONMENT_SERVICE)
        .get("https://itunes.apple.com/lookup")
        .query(&query)
        .send()
        .await
        .map_err(map_reqwest_error)?
        .json::<Value>()
        .await
        .map_err(map_reqwest_error)?;

    let result = &res["results"][0];
    let collection_id = result["collectionId"]
        .as_i64()
        .and_then(|id| i32::try_from(id).ok())
        .ok_or_else(|| {
            CustomErrorInner::BadRequest(
                format!("iTunes has no podcast with id {}", track_id.track_id),
                ErrorSeverity::Info,
            )
        })?;

    let server_url = resolve_server_url_from_headers(&headers);
    let inserted = PodcastService::handle_insert_of_podcast(
        PodcastInsertModel {
            feed_url: unwrap_string(&result["feedUrl"]),
            title: unwrap_string(&result["collectionName"]),
            id: collection_id,
            image_url: unwrap_string(&result["artworkUrl600"]),
        },
        None,
        Some(requester.id),
    )
    .await?;
    Ok(Json(map_podcast_to_dto(inserted.into(), &server_url)))
}

#[utoipa::path(
post,
path="/podcasts/feed",
request_body = PodcastRSSAddModel,
responses(
(status = 200, description = "Adds a podcast by its feed url",body=PodcastDto)),
tag="podcasts"
)]
pub async fn add_podcast_by_feed(
    Extension(requester): Extension<User>,
    headers: HeaderMap,
    Json(rss_feed): Json<PodcastRSSAddModel>,
) -> Result<Json<PodcastDto>, CustomError> {
    PodcastService::ensure_can_add_podcasts(&requester, 1)?;
    let PodcastRSSAddModel { rss_feed_url } = rss_feed;
    let mut header_map = ReqwestHeaderMap::new();
    header_map.insert(
        reqwest::header::USER_AGENT,
        HeaderValue::from_static(COMMON_USER_AGENT),
    );
    add_basic_auth_headers_conditionally(rss_feed_url.clone(), &mut header_map);
    let body = get_http_client(&ENVIRONMENT_SERVICE)
        .get(&rss_feed_url)
        .headers(header_map)
        .send()
        .await
        .map_err(map_reqwest_error)?
        .bytes()
        .await
        .map_err(map_reqwest_error)?;
    let channel = parse_feed(&body)?;

    // `ThreadRng` is not `Send`, so it must not live across the await below.
    let id = rand::rng().random_range(100..10_000_000);
    let server_url = resolve_server_url_from_headers(&headers);
    let inserted = PodcastService::handle_insert_of_podcast(
        PodcastInsertModel {
            feed_url: rss_feed_url,
            title: channel.title.clone(),
            id,
            image_url: crate::podcast::channel_image_url(&channel)
                .unwrap_or_else(|| get_default_image(&server_url)),
        },
        Some(channel),
        Some(requester.id),
    )
    .await?;
    let res: PodcastDto = map_podcast_to_dto(inserted.into(), &server_url);

    Ok(Json(res))
}

#[utoipa::path(
post,
path="/podcasts/opml",
request_body=OpmlModel,
responses(
(status = 200, description = "Adds all podcasts of an opml podcast list to the database.")),
tag="podcasts"
)]
pub async fn import_podcasts_from_opml(
    requester: Extension<User>,
    Json(opml): Json<OpmlModel>,
) -> Result<StatusCode, CustomError> {
    let document = OPML::from_str(&opml.content).map_err(|e| {
        CustomErrorInner::BadRequest(format!("Invalid OPML file: {e}"), ErrorSeverity::Info)
    })?;
    let feed_urls = opml_feed_urls(&document.body.outlines);
    let feed_count = u32::try_from(feed_urls.len()).unwrap_or(u32::MAX);
    PodcastService::ensure_can_add_podcasts(&requester, feed_count)?;

    let added_by = Some(requester.id);
    spawn_detached("opml-import", move || async move {
        futures::future::join_all(
            feed_urls
                .into_iter()
                .map(|feed_url| import_opml_feed(feed_url, added_by)),
        )
        .await;
        Ok::<_, CustomError>(())
    });

    Ok(StatusCode::OK)
}

/// Collects the feed URLs of all leaf outlines, depth-first.
fn opml_feed_urls(outlines: &[Outline]) -> Vec<String> {
    outlines
        .iter()
        .flat_map(|outline| {
            if outline.outlines.is_empty() {
                outline.xml_url.clone().into_iter().collect()
            } else {
                opml_feed_urls(&outline.outlines)
            }
        })
        .collect()
}

#[utoipa::path(
post,
path="/podcasts/podindex",
request_body=PodcastAddModel,
responses(
(status = 200, description = "Adds a podindex podcast to the database")),
tag="podcasts"
)]
pub async fn add_podcast_from_podindex(
    Extension(requester): Extension<User>,
    Json(id): Json<PodcastAddModel>,
) -> Result<StatusCode, CustomError> {
    PodcastService::ensure_can_add_podcasts(&requester, 1)?;
    ensure_podindex_configured::<CustomError>(
        !ENVIRONMENT_SERVICE.podindex_api_key.is_empty()
            && !ENVIRONMENT_SERVICE.podindex_api_secret.is_empty(),
    )
    .map_err(map_podcast_error)?;

    let added_by = Some(requester.id);
    let track_id = id.track_id;
    spawn_detached("podindex-import", move || async move {
        PodcastService::insert_podcast_from_podindex(track_id, added_by)
            .await
            .map(|_| ())
    });
    Ok(StatusCode::OK)
}

#[utoipa::path(
get,
path="/podcasts/{podcast}/query",
params(("podcast", description="The podcast episode query parameter.")),
responses(
(status = 200, description = "Queries for a podcast episode by a query string", body = Vec<PodcastEpisodeDto>)),
tag="podcasts",)]
pub async fn query_for_podcast(
    podcast: Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<PodcastEpisodeDto>>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
    let res = PodcastEpisodeService::query_for_podcast(&podcast)?
        .into_iter()
        .map(|p| {
            PodcastEpisodeDto::from_episode_with_user(
                p,
                None::<User>,
                None::<FavoritePodcastEpisode>,
                &server_url,
            )
        })
        .collect::<Vec<PodcastEpisodeDto>>();

    Ok(Json(res))
}

#[utoipa::path(
post,
path="/podcasts/all",
responses(
(status = 200, description = "Refreshes all podcasts")),
tag="podcasts"
)]
pub async fn refresh_all_podcasts(
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    require_privileged::<CustomError>(requester.is_privileged_user()).map_err(map_podcast_error)?;

    let podcasts = PodcastService::get_all_podcasts_raw()?;
    thread::spawn(move || {
        for podcast in podcasts {
            let refresh_result = PodcastService::refresh_podcast(&podcast);
            if let Err(e) = refresh_result {
                tracing::error!("Error refreshing podcast: {e}");
            }
            ChatServerHandle::broadcast_podcast_refreshed(&podcast);
        }
    });
    Ok(StatusCode::OK)
}

#[utoipa::path(
post,
path="/podcasts/{id}/refresh",
responses(
(status = 200, description = "Refreshes a podcast episode")),
tag="podcasts"
)]
pub async fn download_podcast(
    Extension(requester): Extension<User>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, CustomError> {
    require_privileged::<CustomError>(requester.is_privileged_user()).map_err(map_podcast_error)?;

    let (_podcast_uuid, podcast) = resolve_podcast(&id)?;
    thread::spawn(move || {
        match PodcastService::refresh_podcast(&podcast) {
            Ok(_) => tracing::info!("Successfully refreshed podcast {}", podcast.name),
            Err(e) => tracing::error!("Error refreshing podcast {}: {e}", podcast.name),
        }
        if let Err(e) = PodcastService::schedule_episode_download(&podcast) {
            tracing::error!("Error downloading episodes of {}: {e}", podcast.name);
        }
    });

    Ok("Refreshing podcast")
}

#[utoipa::path(
put,
path="/podcasts/favored",
request_body=PodcastFavorUpdateModel,
responses(
(status = 200, description = "Updates favoring a podcast.", body=String)),
tag="podcasts"
)]
pub async fn favorite_podcast(
    requester: Extension<User>,
    update_model: Json<PodcastFavorUpdateModel>,
) -> Result<StatusCode, CustomError> {
    let (podcast_uuid, _) = resolve_podcast(&update_model.id)?;
    PodcastService::update_favor_podcast(podcast_uuid, update_model.favored, requester.id)?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
put,
path="/podcasts/{id}/name",
request_body=PodcastUpdateNameRequest,
responses(
(status = 200, description = "Updates the name of a podcast.", body=String)),
tag="podcasts"
)]
pub async fn update_name_of_podcast(
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
    req: Json<PodcastUpdateNameRequest>,
) -> Result<StatusCode, CustomError> {
    require_admin::<CustomError>(requester.is_admin()).map_err(map_podcast_error)?;
    let (podcast_uuid, _found_podcast) = match resolve_podcast(&id) {
        Ok(p) => Ok(p),
        Err(..) => Err(CustomErrorInner::NotFound(Debug)),
    }?;

    PodcastService::update_podcast_name(podcast_uuid, &req.name)?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
get,
path="/podcasts/favored",
responses(
(status = 200, description = "Finds all favored podcasts.")),
tag="podcasts"
)]
pub async fn get_favored_podcasts(
    Extension(requester): Extension<User>,
    headers: HeaderMap,
) -> Result<Json<Vec<PodcastDto>>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
    let podcasts = PodcastService::get_favored_podcasts(requester, &server_url)?;
    Ok(Json(podcasts))
}

#[utoipa::path(
put,
path="/podcasts/{id}/active",
responses(
(status = 200, description = "Updates the active state of a podcast. If inactive the podcast \
will not be refreshed automatically.")),
tag="podcasts"
)]
pub async fn update_active_podcast(
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    require_privileged::<CustomError>(requester.is_privileged_user()).map_err(map_podcast_error)?;

    let (podcast_uuid, _) = resolve_podcast(&id)?;
    PodcastService::update_active_podcast(podcast_uuid)?;
    Ok(StatusCode::OK)
}

/// Imports a single feed of an OPML file and reports the outcome to the UI.
async fn import_opml_feed(feed_url: String, added_by: Option<uuid::Uuid>) {
    match fetch_and_insert_feed(&feed_url, added_by).await {
        Ok(podcast) => ChatServerHandle::broadcast_opml_added(&podcast),
        Err(error) => ChatServerHandle::broadcast_opml_error(error.to_string()),
    }
}

async fn fetch_and_insert_feed(
    feed_url: &str,
    added_by: Option<uuid::Uuid>,
) -> Result<podfetch_persistence::podcast::PodcastEntity, CustomError> {
    let body = get_http_client(&ENVIRONMENT_SERVICE)
        .get(feed_url)
        .send()
        .await
        .map_err(map_reqwest_error)?
        .bytes()
        .await
        .map_err(map_reqwest_error)?;
    let channel = parse_feed(&body)?;
    // An empty image URL makes the default image apply at response time.
    let image_url = crate::podcast::channel_image_url(&channel).unwrap_or_default();

    PodcastService::handle_insert_of_podcast(
        PodcastInsertModel {
            feed_url: feed_url.to_owned(),
            title: channel.title.clone(),
            id: rand::random::<i32>(),
            image_url,
        },
        Some(channel),
        added_by,
    )
    .await
}

fn parse_feed(body: &[u8]) -> Result<Channel, CustomError> {
    Channel::read_from(body).map_err(|e| {
        CustomErrorInner::BadRequest(format!("Invalid podcast feed: {e}"), ErrorSeverity::Info)
            .into()
    })
}

use crate::podcast::PodcastDto;
use crate::podcast::{map_podcast_to_dto, map_podcast_with_context_to_dto};
use crate::podcast_episode_dto::PodcastEpisodeDto;
use crate::rss::RSSAPiKey;
use crate::server::ChatServerHandle;
use crate::settings::Setting;
use crate::usecases::watchtime::WatchtimeUseCase as WatchtimeService;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::podcast_episode::EpisodeFormatDto;
use crate::podcast_settings::PodcastSetting;
use common_infrastructure::error::{
    CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error,
};
use common_infrastructure::rss::PodcastParsed;

#[utoipa::path(
delete,
path="/podcasts/{id}",
request_body=DeletePodcast,
responses(
(status = 200, description = "Deletes a podcast by id")),
tag="podcasts"
)]
pub async fn delete_podcast(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
    Json(data): Json<DeletePodcast>,
) -> Result<StatusCode, CustomError> {
    require_privileged::<CustomError>(requester.is_privileged_user()).map_err(map_podcast_error)?;

    let (podcast_uuid, podcast) = resolve_podcast(&id)?;
    // Collect the file paths while the episodes still exist, but delete the
    // files only after the database rows: if a database step fails, the
    // podcast stays complete and the delete can simply be retried.
    let files = data
        .delete_files
        .then(|| FileService::collect_podcast_files(&podcast));

    WatchtimeService::delete_watchtime(podcast_uuid)?;
    PodcastEpisodeService::delete_episodes_of_podcast(podcast_uuid)?;
    state.tag_service.delete_podcast_tags(podcast_uuid)?;
    PodcastService::delete_podcast(podcast_uuid)?;

    if let Some(files) = files {
        let directory = files.podcast.directory_name.clone();
        match tokio::task::spawn_blocking(move || FileService::remove_podcast_files(&files)).await {
            Ok(Ok(())) => {}
            // The podcast is gone; leftover files must not turn that into an
            // error the user can't act on (the podcast can't be deleted twice).
            Ok(Err(err)) => {
                tracing::error!(
                    "Podcast {podcast_uuid} deleted, but its files in {directory} could not be removed: {err}"
                );
            }
            Err(err) => {
                tracing::error!(
                    "Podcast {podcast_uuid} deleted, but removing its files in {directory} failed: {err}"
                );
            }
        }
    }
    Ok(StatusCode::OK)
}
use axum::response::Response;
use common_infrastructure::error::ErrorSeverity::Debug;

#[utoipa::path(
get,
path="/proxy/podcast",
responses(
(status = 200, description = "Proxies a podcast so people can stream podcasts from the remote \
server")),
tag="podcasts"
)]
pub(crate) async fn proxy_podcast(
    State(state): State<AppState>,
    Query(params): Query<ProxyPodcastParams>,
    OptionalQuery(api_key): OptionalQuery<RSSAPiKey>,
    req: axum::extract::Request,
) -> Result<http::response::Response<Body>, CustomError> {
    let mut req = req.map(|body| reqwest::Body::wrap_stream(body.into_data_stream()));
    let headers = req.headers_mut();

    let is_auth_enabled =
        is_env_var_present_and_true(BASIC_AUTH) || is_env_var_present_and_true(OIDC_AUTH);
    ensure_proxy_api_access::<CustomError, _>(
        is_auth_enabled,
        api_key.and_then(|q| q.api_key),
        |key| state.user_auth_service.is_api_key_valid(key),
    )
    .map_err(map_proxy_podcast_error)?;

    let episode = require_proxy_episode::<_, CustomError>(
        PodcastEpisodeService::get_podcast_episode_by_id(&params.episode_id)?,
    )
    .map_err(map_proxy_podcast_error)?;
    sanitize_proxy_request_headers(headers);

    let cloned_headers = headers.clone();

    add_basic_auth_headers_conditionally(episode.url.clone(), headers);

    let invalid_url = |_| {
        CustomError::from(CustomErrorInner::BadRequest(
            format!("Episode has an invalid URL: {}", episode.url),
            ErrorSeverity::Warning,
        ))
    };
    *req.uri_mut() = episode.url.parse().map_err(invalid_url)?;
    let upstream_request = reqwest::Request::try_from(req).map_err(map_reqwest_error)?;

    let client = &*PROXY_CLIENT;
    let mut resp = client
        .execute(upstream_request)
        .await
        .map_err(map_reqwest_error)?;

    // Only reached when the redirect limit is exhausted; follow once more with
    // the original headers.
    if resp.status().is_redirection()
        && let Some(location) = resp.headers().get(reqwest::header::LOCATION)
        && let Ok(location) = location.to_str()
    {
        resp = client
            .get(location)
            .headers(cloned_headers)
            .send()
            .await
            .map_err(map_reqwest_error)?;
    }

    let status = resp.status();
    let headers = resp.headers().clone();
    let mut response = Response::new(Body::from_stream(resp.bytes_stream()));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    Ok(response)
}

#[utoipa::path(
get,
path="/proxy/podcast/apiKey/{apiKey}",
responses(
(status = 200, description = "Proxies a podcast (API key in path)")),
tag="podcasts"
)]
pub(crate) async fn proxy_podcast_with_path_api_key(
    State(state): State<AppState>,
    Path(api_key): Path<String>,
    Query(params): Query<ProxyPodcastParams>,
    req: axum::extract::Request,
) -> Result<http::response::Response<Body>, CustomError> {
    let api_key_query = OptionalQuery(Some(RSSAPiKey {
        api_key: Some(api_key),
    }));
    proxy_podcast(State(state), Query(params), api_key_query, req).await
}

#[utoipa::path(
    put,
    path="/podcasts/{id}/settings",
    responses(
(status = 200, description = "Updates the settings of a podcast by id", body=PodcastSetting)),
    tag="podcasts"
)]
pub async fn update_podcast_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
    Json(mut settings): Json<PodcastSetting>,
) -> Result<Json<PodcastSetting>, CustomError> {
    require_privileged::<CustomError>(requester.is_privileged_user()).map_err(map_podcast_error)?;
    let (podcast_uuid, _) = resolve_podcast(&id)?;
    settings.podcast_id = podcast_uuid.to_string();
    let updated_podcast = state.podcast_settings_service.update_settings(settings)?;

    Ok(Json(updated_podcast))
}

#[utoipa::path(
    get,
    path="/podcasts/{id}/settings",
    responses(
(status = 200, description = "Gets the settings of a podcast by id", body=PodcastSetting)),
    tag="podcasts"
)]
pub async fn get_podcast_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<Json<PodcastSetting>, CustomError> {
    require_privileged::<CustomError>(requester.is_privileged_user()).map_err(map_podcast_error)?;
    let (podcast_uuid, _) = resolve_podcast(&id)?;
    let settings = state.podcast_settings_service.get_settings(podcast_uuid)?;

    match settings {
        None => Err(CustomErrorInner::NotFound(Debug).into()),
        Some(s) => Ok(Json(s)),
    }
}

#[utoipa::path(
    post,
    path="/podcasts/formatting",
    responses(
(status = 200, description = "Retrieve the podcast sample format")),
    tag="podcasts"
)]
pub async fn retrieve_podcast_sample_format(
    sample_string: Json<EpisodeFormatDto>,
) -> Result<Json<String>, CustomError> {
    let podcast = PodcastParsed {
        date: "2021-01-01".to_string(),
        summary: "A podcast about homelabing".to_string(),
        title: "The homelab podcast".to_string(),
        keywords: "computer, server, apps".to_string(),
        language: "en".to_string(),
        explicit: "false".to_string(),
    };
    let settings = Setting {
        id: uuid::Uuid::nil().to_string(),
        episode_numbering: false,
        auto_download: false,
        auto_update: false,
        auto_cleanup: false,
        auto_cleanup_days: 0,
        podcast_prefill: 0,
        replace_invalid_characters: false,
        use_existing_filename: false,
        replacement_strategy: "remove".to_string(),
        episode_format: String::new(),
        podcast_format: sample_string.0.content,
        direct_paths: true,
        auto_transcode_opus: false,
        use_one_cover_for_all_episodes: false,
        max_parallel_downloads: 3,
        sponsorblock_enabled: true,
        nfo_format: "off".to_string(),
        cover_filename: "image".to_string(),
    };
    let result = perform_podcast_variable_replacement(settings.into(), podcast, None);

    match result {
        Ok(v) => Ok(Json(v)),
        Err(e) => Err(CustomErrorInner::BadRequest(e.to_string(), ErrorSeverity::Info).into()),
    }
}

pub fn get_podcast_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_filter))
        .routes(routes!(search_podcasts))
        .routes(routes!(find_podcast_by_id))
        .routes(routes!(find_all_podcasts))
        .routes(routes!(find_podcast))
        .routes(routes!(add_podcast))
        .routes(routes!(add_podcast_by_feed))
        .routes(routes!(import_podcasts_from_opml))
        .routes(routes!(add_podcast_from_podindex))
        .routes(routes!(get_favored_podcasts))
        .routes(routes!(refresh_all_podcasts))
        .routes(routes!(download_podcast))
        .routes(routes!(favorite_podcast))
        .routes(routes!(update_active_podcast))
        .routes(routes!(delete_podcast))
        .routes(routes!(update_podcast_settings))
        .routes(routes!(get_podcast_settings))
        .routes(routes!(retrieve_podcast_sample_format))
        .routes(routes!(query_for_podcast))
        .routes(routes!(update_name_of_podcast))
        .routes(routes!(search_podcast_of_episode))
}

#[cfg(test)]
pub mod tests {
    use crate::app_state::AppState;
    use crate::controllers::podcast_controller::PodcastUpdateNameRequest;
    use crate::controllers::podcast_controller::find_podcast;
    use crate::podcast::{OpmlModel, PodcastAddModel, PodcastRSSAddModel, SearchType};
    use crate::podcast_episode::EpisodeFormatDto;
    use crate::podcast_settings::PodcastSetting;
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use axum::extract::{Path, State};
    use axum::http::HeaderMap;
    use axum::{Extension, Json};
    use common_infrastructure::error::CustomErrorInner;
    use podfetch_domain::user::User;
    use serde_json::json;
    use serial_test::serial;
    use uuid::Uuid;

    fn unique_name(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    fn non_privileged_user() -> User {
        UserTestDataBuilder::new().build()
    }

    fn app_state() -> AppState {
        AppState::new()
    }

    #[tokio::test]
    #[serial]
    async fn test_retrieve_podcast_sample_format() {
        let ts_server = handle_test_startup().await;
        let resp = ts_server
            .test_server
            .post("/api/v1/podcasts/formatting")
            .json(&EpisodeFormatDto {
                content: "test".to_string(),
            })
            .await;
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.json::<String>(), "test");
    }

    #[tokio::test]
    #[serial]
    async fn test_retrieve_podcast_sample_format_with_podcast_title() {
        let ts_server = handle_test_startup().await;
        let resp = ts_server
            .test_server
            .post("/api/v1/podcasts/formatting")
            .json(&EpisodeFormatDto {
                content: "{podcastTitle}".to_string(),
            })
            .await;
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.json::<String>(), "The homelab podcast");
    }

    #[tokio::test]
    #[serial]
    async fn test_change_name_of_podcast() {
        let ts_server = handle_test_startup().await;
        let collection_name = unique_name("collection");
        let podcast_name = unique_name("The homelab podcast");
        let directory_name = unique_name("test123");
        let saved_podcast =
            crate::services::podcast::service::PodcastService::add_podcast_to_database(
                &collection_name,
                &podcast_name,
                "https://example.com/feed",
                "https://example.com/image\
                                         .jpg",
                &directory_name,
            )
            .unwrap();
        let resp = ts_server
            .test_server
            .put(&format!("/api/v1/podcasts/{}/name", saved_podcast.id))
            .json(&PodcastUpdateNameRequest {
                name: "New Podcast Name".to_string(),
            })
            .await;
        assert_eq!(resp.status_code(), 200);
        assert_eq!(
            crate::services::podcast::service::PodcastService::get_podcast(
                Uuid::parse_str(&saved_podcast.id).unwrap()
            )
            .unwrap()
            .name,
            "New Podcast Name"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_retrieve_podcast_sample_format_with_podcast_description() {
        let ts_server = handle_test_startup().await;
        let resp = ts_server
            .test_server
            .post("/api/v1/podcasts/formatting")
            .json(&EpisodeFormatDto {
                content: "{podcastDescription}".to_string(),
            })
            .await;
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.json::<String>(), "A podcast about homelabing");
    }

    #[tokio::test]
    #[serial]
    async fn test_retrieve_podcast_sample_format_with_podcast_title_date() {
        let ts_server = handle_test_startup().await;
        let resp = ts_server
            .test_server
            .post("/api/v1/podcasts/formatting")
            .json(&EpisodeFormatDto {
                content: "{podcastDescription}-{date}".to_string(),
            })
            .await;
        assert_eq!(resp.status_code(), 200);
        assert_eq!(
            resp.json::<String>(),
            "A podcast about homelabing-2021-01-01"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_proxy_podcast_without_api_key_does_not_fail_deserialization() {
        let ts_server = handle_test_startup().await;
        let resp = ts_server
            .test_server
            .get("/proxy/podcast?episodeId=does-not-exist&apiKey=test-api-key")
            .await;
        assert_eq!(resp.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_name_of_podcast_with_unknown_id_returns_not_found() {
        let ts_server = handle_test_startup().await;

        let resp = ts_server
            .test_server
            .put("/api/v1/podcasts/999999/name")
            .json(&PodcastUpdateNameRequest {
                name: unique_name("Unknown Podcast Rename"),
            })
            .await;

        assert_eq!(resp.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_name_of_podcast_rejects_invalid_payload() {
        let ts_server = handle_test_startup().await;
        let saved_podcast =
            crate::services::podcast::service::PodcastService::add_podcast_to_database(
                &unique_name("invalid-payload-collection"),
                &unique_name("Invalid Payload Podcast"),
                "https://example.com/invalid-payload-feed.xml",
                "https://example.com/invalid-payload-image.jpg",
                &unique_name("invalid-payload-id"),
            )
            .unwrap();

        let resp = ts_server
            .test_server
            .put(&format!("/api/v1/podcasts/{}/name", saved_podcast.id))
            .json(&json!({
                "otherField": "missing-name"
            }))
            .await;

        assert_client_error_status(resp.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_update_name_of_podcast_noop_with_same_name() {
        let ts_server = handle_test_startup().await;
        let original_name = unique_name("Noop Rename Podcast");
        let saved_podcast =
            crate::services::podcast::service::PodcastService::add_podcast_to_database(
                &unique_name("noop-collection"),
                &original_name,
                "https://example.com/noop-rename-feed.xml",
                "https://example.com/noop-rename-image.jpg",
                &unique_name("noop-rename-id"),
            )
            .unwrap();

        let resp = ts_server
            .test_server
            .put(&format!("/api/v1/podcasts/{}/name", saved_podcast.id))
            .json(&PodcastUpdateNameRequest {
                name: original_name.clone(),
            })
            .await;
        assert_eq!(resp.status_code(), 200);

        let persisted = crate::services::podcast::service::PodcastService::get_podcast(
            Uuid::parse_str(&saved_podcast.id).unwrap(),
        )
        .unwrap();
        assert_eq!(persisted.name, original_name);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_name_of_podcast_returns_forbidden_for_non_admin() {
        let non_admin = UserTestDataBuilder::new().build();
        let saved_podcast =
            crate::services::podcast::service::PodcastService::add_podcast_to_database(
                &unique_name("forbidden-collection"),
                &unique_name("Forbidden Rename Podcast"),
                "https://example.com/forbidden-rename-feed.xml",
                "https://example.com/forbidden-rename-image.jpg",
                &unique_name("forbidden-rename-id"),
            )
            .unwrap();

        let result = super::update_name_of_podcast(
            Path(saved_podcast.id),
            Extension(non_admin),
            Json(PodcastUpdateNameRequest {
                name: unique_name("Should Not Update"),
            }),
        )
        .await;

        match result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_podcast_settings_returns_not_found_when_missing() {
        let ts_server = handle_test_startup().await;
        let saved_podcast =
            crate::services::podcast::service::PodcastService::add_podcast_to_database(
                &unique_name("settings-missing-collection"),
                &unique_name("Settings Missing Podcast"),
                "https://example.com/settings-missing-feed.xml",
                "https://example.com/settings-missing-image.jpg",
                &unique_name("settings-missing-id"),
            )
            .unwrap();

        let resp = ts_server
            .test_server
            .get(&format!("/api/v1/podcasts/{}/settings", saved_podcast.id))
            .await;

        assert_eq!(resp.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_and_get_podcast_settings_happy_path() {
        let ts_server = handle_test_startup().await;
        let saved_podcast =
            crate::services::podcast::service::PodcastService::add_podcast_to_database(
                &unique_name("settings-happy-collection"),
                &unique_name("Settings Happy Podcast"),
                "https://example.com/settings-happy-feed.xml",
                "https://example.com/settings-happy-image.jpg",
                &unique_name("settings-happy-id"),
            )
            .unwrap();

        let update_payload = PodcastSetting {
            podcast_id: saved_podcast.id.clone(),
            episode_numbering: true,
            auto_download: true,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 30,
            replace_invalid_characters: true,
            use_existing_filename: true,
            replacement_strategy: "replace-with-dash".to_string(),
            episode_format: "{episodeTitle}".to_string(),
            podcast_format: "{podcastTitle}".to_string(),
            direct_paths: false,
            activated: true,
            podcast_prefill: 10,
            use_one_cover_for_all_episodes: false,
            nfo_format: "off".to_string(),
            cover_filename: "cover".to_string(),
            auto_transcribe: false,
        };

        let update_resp = ts_server
            .test_server
            .put(&format!("/api/v1/podcasts/{}/settings", saved_podcast.id))
            .json(&update_payload)
            .await;
        assert_eq!(update_resp.status_code(), 200);

        let get_resp = ts_server
            .test_server
            .get(&format!("/api/v1/podcasts/{}/settings", saved_podcast.id))
            .await;
        assert_eq!(get_resp.status_code(), 200);
        let persisted = get_resp.json::<PodcastSetting>();
        assert_eq!(persisted.podcast_id, saved_podcast.id);
        assert!(persisted.episode_numbering);
        assert!(persisted.auto_download);
        assert_eq!(persisted.replacement_strategy, "replace-with-dash");
    }

    #[tokio::test]
    #[serial]
    async fn test_podcast_endpoints_return_client_error_for_wrong_http_methods() {
        let ts_server = handle_test_startup().await;
        let saved_podcast =
            crate::services::podcast::service::PodcastService::add_podcast_to_database(
                &unique_name("method-mismatch-collection"),
                &unique_name("Method Mismatch Podcast"),
                "https://example.com/method-mismatch-feed.xml",
                "https://example.com/method-mismatch-image.jpg",
                &unique_name("method-mismatch-id"),
            )
            .unwrap();

        let post_on_name = ts_server
            .test_server
            .post(&format!("/api/v1/podcasts/{}/name", saved_podcast.id))
            .json(&json!({ "name": "noop" }))
            .await;
        assert_client_error_status(post_on_name.status_code().as_u16());

        let post_on_settings = ts_server
            .test_server
            .post(&format!("/api/v1/podcasts/{}/settings", saved_podcast.id))
            .json(&PodcastSetting::default())
            .await;
        assert_client_error_status(post_on_settings.status_code().as_u16());

        let get_on_formatting = ts_server
            .test_server
            .get("/api/v1/podcasts/formatting")
            .await;
        assert_client_error_status(get_on_formatting.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_podcast_endpoints_reject_non_numeric_path_ids() {
        let ts_server = handle_test_startup().await;

        let update_name_response = ts_server
            .test_server
            .put("/api/v1/podcasts/not-a-number/name")
            .json(&json!({ "name": "invalid" }))
            .await;
        assert_client_error_status(update_name_response.status_code().as_u16());

        let get_settings_response = ts_server
            .test_server
            .get("/api/v1/podcasts/not-a-number/settings")
            .await;
        assert_client_error_status(get_settings_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_find_podcast_invalid_search_type_returns_bad_request() {
        let ts_server = handle_test_startup().await;

        let response = ts_server
            .test_server
            .get("/api/v1/podcasts/99/rust/search")
            .await;
        assert_eq!(response.status_code(), 400);
    }

    #[tokio::test]
    #[serial]
    async fn test_non_privileged_user_is_forbidden_for_admin_podcast_handlers() {
        let non_privileged = non_privileged_user();

        let add_itunes_result = super::add_podcast(
            Extension(non_privileged.clone()),
            HeaderMap::new(),
            Json(PodcastAddModel {
                track_id: 12345,
                user_id: 0,
            }),
        )
        .await;
        match add_itunes_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for add_podcast"),
        }

        let add_feed_result = super::add_podcast_by_feed(
            Extension(non_privileged.clone()),
            HeaderMap::new(),
            Json(PodcastRSSAddModel {
                rss_feed_url: "https://example.com/feed.xml".to_string(),
            }),
        )
        .await;
        match add_feed_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for add_podcast_by_feed"),
        }

        let import_result = super::import_podcasts_from_opml(
            Extension(non_privileged.clone()),
            Json(OpmlModel {
                content: "<opml version=\"2.0\"><head><title>Test</title></head><body><outline text=\"feed\" xmlUrl=\"https://example.com/feed.xml\" /></body></opml>".to_string(),
            }),
        )
        .await;
        match import_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for import_podcasts_from_opml"),
        }

        let find_result = find_podcast(
            Path((SearchType::ITunes as i32, "rust".to_string())),
            Extension(non_privileged.clone()),
        )
        .await;
        match find_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for find_podcast"),
        }

        let add_podindex_result = super::add_podcast_from_podindex(
            Extension(non_privileged.clone()),
            Json(PodcastAddModel {
                track_id: 321,
                user_id: 0,
            }),
        )
        .await;
        match add_podindex_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for add_podcast_from_podindex"),
        }

        let refresh_all_result =
            super::refresh_all_podcasts(Extension(non_privileged.clone())).await;
        match refresh_all_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for refresh_all_podcasts"),
        }

        let download_result =
            super::download_podcast(Extension(non_privileged.clone()), Path("1".to_string())).await;
        match download_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for download_podcast"),
        }

        let update_active_result =
            super::update_active_podcast(Path("1".to_string()), Extension(non_privileged.clone()))
                .await;
        match update_active_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for update_active_podcast"),
        }

        let delete_podcast_result = super::delete_podcast(
            State(app_state()),
            Path("1".to_string()),
            Extension(non_privileged.clone()),
            Json(super::DeletePodcast {
                delete_files: false,
            }),
        )
        .await;
        match delete_podcast_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for delete_podcast"),
        }

        let update_settings_result = super::update_podcast_settings(
            State(app_state()),
            Path("1".to_string()),
            Extension(non_privileged.clone()),
            Json(PodcastSetting::default()),
        )
        .await;
        match update_settings_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for update_podcast_settings"),
        }

        let get_settings_result = super::get_podcast_settings(
            State(app_state()),
            Path("1".to_string()),
            Extension(non_privileged),
        )
        .await;
        match get_settings_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for get_podcast_settings"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_podcast_endpoints_return_not_found_for_invalid_paths() {
        let ts_server = handle_test_startup().await;

        let wrong_route_response = ts_server
            .test_server
            .get("/api/v1/podcasts/does-not-exist/unknown")
            .await;
        assert_eq!(wrong_route_response.status_code(), 404);

        let trailing_slash_response = ts_server
            .test_server
            .get("/api/v1/podcasts/formatting/")
            .await;
        assert_client_error_status(trailing_slash_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_podcast_endpoints_reject_invalid_payloads() {
        let ts_server = handle_test_startup().await;

        let favorite_invalid_response = ts_server
            .test_server
            .put("/api/v1/podcasts/favored")
            .json(&json!({
                "id": "not-a-number",
                "favored": "yes"
            }))
            .await;
        assert_client_error_status(favorite_invalid_response.status_code().as_u16());

        let delete_invalid_response = ts_server
            .test_server
            .delete("/api/v1/podcasts/1")
            .json(&json!({
                "otherField": true
            }))
            .await;
        assert_client_error_status(delete_invalid_response.status_code().as_u16());
    }

    // ── Path-based API key proxy tests ──────────────────────────────────

    fn create_api_key_user() -> String {
        let state = AppState::new();
        let mut user = UserTestDataBuilder::new().build();
        user.username = format!("proxy-test-user-{}", Uuid::new_v4());
        let api_key = format!("proxy-test-key-{}", Uuid::new_v4());
        user.api_key = Some(api_key.clone());
        let _created = state.user_admin_service.create_user(user).unwrap();
        api_key
    }

    #[tokio::test]
    #[serial]
    async fn test_proxy_podcast_with_path_api_key_returns_not_found_for_unknown_episode() {
        let ts_server = handle_test_startup().await;
        let api_key = create_api_key_user();

        let resp = ts_server
            .test_server
            .get(&format!(
                "/proxy/podcast/apiKey/{api_key}?episodeId=does-not-exist"
            ))
            .await;
        assert_eq!(resp.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_proxy_podcast_with_path_api_key_rejects_invalid_key() {
        let ts_server = handle_test_startup().await;

        let resp = ts_server
            .test_server
            .get("/proxy/podcast/apiKey/invalid-key?episodeId=does-not-exist")
            .await;
        assert_eq!(resp.status_code(), 403);
    }
}
