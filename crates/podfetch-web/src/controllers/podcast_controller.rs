use crate::app_state::AppState;
use crate::controllers::controller_utils::{get_default_image, unwrap_string};
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use async_recursion::async_recursion;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json, debug_handler};
use axum_extra::extract::OptionalQuery;
use common_infrastructure::config::{BASIC_AUTH, OIDC_AUTH};
use common_infrastructure::http::COMMON_USER_AGENT;
use common_infrastructure::runtime::{DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE};
use opml::{OPML, Outline};
use rand::RngExt;
use rand::rngs::ThreadRng;
use rss::Channel;
use serde_json::Value;
use std::thread;
use tokio::task::spawn_blocking;

use crate::filter::Filter;
pub use crate::podcast::{
    DeletePodcast, OpmlModel, PodcastAddModel, PodcastFavorUpdateModel, PodcastInsertModel,
    PodcastRSSAddModel, PodcastSearchModelUtoipa, PodcastSearchReturn, PodcastUpdateNameRequest,
    ProxyPodcastParams,
    SearchType::{ITunes, Podindex},
};
use crate::podcast::{
    build_podcast_search_plan, check_podcast_add_permission, ensure_podindex_configured,
    ensure_proxy_api_access, map_podcast_error, map_proxy_podcast_error, parse_podcast_id,
    parse_search_type, require_admin, require_privileged, require_proxy_episode,
    sanitize_proxy_request_headers, spawn_podindex_download,
};
use crate::services::file::service::{FileService, perform_podcast_variable_replacement};
use crate::url_rewriting::create_url_rewriter;
use common_infrastructure::config::is_env_var_present_and_true;
use common_infrastructure::http::get_http_client;
use common_infrastructure::request::add_basic_auth_headers_conditionally;
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_domain::podcast::PodcastRepository;
use podfetch_domain::user::User;
use podfetch_persistence::db::database;
use podfetch_persistence::podcast::DieselPodcastRepository;
use reqwest::header::HeaderMap as ReqwestHeaderMap;
use tokio::runtime::Runtime;

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
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Result<Json<PodcastDto>, CustomError> {
    let rewriter = create_url_rewriter(&headers);
    let podcast = PodcastService::get_podcast_by_episode_id(id)?;
    let mut dto = map_podcast_to_dto(podcast.into());
    dto.rewrite_urls(&rewriter);
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
    let rewriter = create_url_rewriter(&headers);
    let existing_filter = state.filter_service.get_filter_by_user_id(requester.id)?;
    let search_plan = build_podcast_search_plan(query, requester.id, existing_filter);
    state.filter_service.save_filter(search_plan.filter)?;

    match search_plan.favored_only {
        true => {
            let podcasts = PodcastService::search_podcasts_favored(
                search_plan.order,
                search_plan.title,
                search_plan.order_option,
                search_plan.user_id,
                search_plan.tag,
                &requester,
            )?;
            let podcasts = podcasts
                .into_iter()
                .map(|p| p.with_rewritten_urls(&rewriter))
                .collect();
            Ok(Json(podcasts))
        }
        false => {
            let podcasts = PodcastService::search_podcasts(
                search_plan.order,
                search_plan.title,
                search_plan.order_option,
                search_plan.tag,
                &requester,
            )?;
            let podcasts = podcasts
                .into_iter()
                .map(|p| p.with_rewritten_urls(&rewriter))
                .collect();
            Ok(Json(podcasts))
        }
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
    let id_num = parse_podcast_id::<CustomError>(&id).map_err(map_podcast_error)?;
    let rewriter = create_url_rewriter(&headers);

    let podcast = PodcastService::get_podcast(id_num)?;
    let tags = state.tag_service.get_tags_of_podcast(id_num, user.id)?;
    let favorite = PodcastService::get_favorite_state(user.id, id_num)?;
    let podcast_dto = map_podcast_with_context_to_dto(podcast.into(), favorite, tags, &user)
        .with_rewritten_urls(&rewriter);
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
    let rewriter = create_url_rewriter(&headers);
    let podcasts = PodcastService::get_podcasts(&requester)?
        .into_iter()
        .map(|p| p.with_rewritten_urls(&rewriter))
        .collect();

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
                log::debug!("Searching for podcast: {podcast}");
                res = PodcastService::find_podcast(&podcast).await;
            }
            Ok(Json(PodcastSearchReturn::Itunes(res)))
        }
        Podindex => {
            ensure_podindex_configured::<CustomError>(
                ENVIRONMENT_SERVICE.get_config().podindex_configured,
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
(status = 200, description = "Adds a podcast to the database.")),
tag="podcasts"
)]
#[debug_handler]
pub async fn add_podcast(
    Extension(requester): Extension<User>,
    Json(track_id): Json<PodcastAddModel>,
) -> Result<StatusCode, CustomError> {
    let current_count = DieselPodcastRepository::new(database())
        .count_by_added_by(requester.id)
        .map_err(CustomError::from)?;
    check_podcast_add_permission::<CustomError>(
        requester.is_privileged_user(),
        ENVIRONMENT_SERVICE.user_podcast_limit,
        current_count,
        1,
    )
    .map_err(map_podcast_error)?;

    let query: Vec<(&str, String)> = vec![
        ("id", track_id.track_id.to_string()),
        ("entity", "podcast".to_string()),
    ];

    let res = get_http_client(&ENVIRONMENT_SERVICE)
        .get("https://itunes.apple.com/lookup")
        .query(&query)
        .send()
        .await
        .unwrap();

    let res = res.json::<Value>().await.unwrap();

    PodcastService::handle_insert_of_podcast(
        PodcastInsertModel {
            feed_url: unwrap_string(&res["results"][0]["feedUrl"]),
            title: unwrap_string(&res["results"][0]["collectionName"]),
            id: unwrap_string(&res["results"][0]["collectionId"])
                .parse()
                .unwrap(),
            image_url: unwrap_string(&res["results"][0]["artworkUrl600"]),
        },
        None,
        Some(requester.id),
    )
    .await?;
    Ok(StatusCode::OK)
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
    Json(rss_feed): Json<PodcastRSSAddModel>,
) -> Result<Json<PodcastDto>, CustomError> {
    let current_count = DieselPodcastRepository::new(database())
        .count_by_added_by(requester.id)
        .map_err(CustomError::from)?;
    check_podcast_add_permission::<CustomError>(
        requester.is_privileged_user(),
        ENVIRONMENT_SERVICE.user_podcast_limit,
        current_count,
        1,
    )
    .map_err(map_podcast_error)?;
    let mut header_map = ReqwestHeaderMap::new();
    header_map.insert("User-Agent", COMMON_USER_AGENT.parse().unwrap());
    add_basic_auth_headers_conditionally(rss_feed.clone().rss_feed_url, &mut header_map);
    let result = get_http_client(&ENVIRONMENT_SERVICE)
        .get(rss_feed.clone().rss_feed_url)
        .headers(header_map)
        .send()
        .await
        .map_err(map_reqwest_error)?;

    let bytes = result.text().await.unwrap();

    let channel = Channel::read_from(bytes.as_bytes()).unwrap();
    let num = rand::rng().random_range(100..10000000);

    let inserted = PodcastService::handle_insert_of_podcast(
        PodcastInsertModel {
            feed_url: rss_feed.clone().rss_feed_url.clone(),
            title: channel.title.clone(),
            id: num,
            image_url: channel
                .image
                .clone()
                .map(|i| i.url)
                .unwrap_or(get_default_image()),
        },
        Some(channel),
        Some(requester.id),
    )
    .await?;
    let res: PodcastDto = map_podcast_to_dto(inserted.into());

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
    let document = OPML::from_str(&opml.content).unwrap();
    let outline_count = count_opml_feeds(&document.body.outlines);
    let current_count = DieselPodcastRepository::new(database())
        .count_by_added_by(requester.id)
        .map_err(CustomError::from)?;
    check_podcast_add_permission::<CustomError>(
        requester.is_privileged_user(),
        ENVIRONMENT_SERVICE.user_podcast_limit,
        current_count,
        outline_count,
    )
    .map_err(map_podcast_error)?;

    let user_id = requester.id;
    spawn_blocking(move || {
        for outline in document.body.outlines {
            let added_by = user_id;
            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                let rng = rand::rng();
                rt.block_on(insert_outline(outline.clone(), rng.clone(), Some(added_by)));
            });
        }
    });

    Ok(StatusCode::OK)
}

fn count_opml_feeds(outlines: &[Outline]) -> u32 {
    outlines
        .iter()
        .map(|o| {
            if o.outlines.is_empty() {
                if o.xml_url.is_some() { 1 } else { 0 }
            } else {
                count_opml_feeds(&o.outlines)
            }
        })
        .sum()
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
    let current_count = DieselPodcastRepository::new(database())
        .count_by_added_by(requester.id)
        .map_err(CustomError::from)?;
    check_podcast_add_permission::<CustomError>(
        requester.is_privileged_user(),
        ENVIRONMENT_SERVICE.user_podcast_limit,
        current_count,
        1,
    )
    .map_err(map_podcast_error)?;
    ensure_podindex_configured::<CustomError>(ENVIRONMENT_SERVICE.get_config().podindex_configured)
        .map_err(map_podcast_error)?;

    let added_by = Some(requester.id);
    spawn_blocking(move || {
        spawn_podindex_download(id.track_id, move |track_id| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                PodcastService::insert_podcast_from_podindex(track_id, added_by)
                    .await
                    .map(|_| ())
            })
        });
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
    let rewriter = create_url_rewriter(&headers);
    let res = PodcastEpisodeService::query_for_podcast(&podcast)?
        .into_iter()
        .map(|p| {
            let mut episode: PodcastEpisodeDto =
                (p, None::<User>, None::<FavoritePodcastEpisode>).into();
            rewriter.rewrite_in_place(&mut episode.local_url);
            rewriter.rewrite_in_place(&mut episode.local_image_url);
            episode
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
                log::error!("Error refreshing podcast: {e}");
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

    let id_num = parse_podcast_id::<CustomError>(&id).map_err(map_podcast_error)?;
    let podcast = PodcastService::get_podcast_by_id(id_num);
    thread::spawn(move || {
        match PodcastService::refresh_podcast(&podcast) {
            Ok(_) => {
                log::info!("Succesfully refreshed podcast.");
            }
            Err(e) => {
                log::error!("Error refreshing podcast: {e}");
            }
        }

        let download = PodcastService::schedule_episode_download(&podcast);

        if download.is_err() {
            log::error!("Error downloading podcast: {}", download.err().unwrap());
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
    PodcastService::update_favor_podcast(update_model.id, update_model.favored, requester.id)?;
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
    Path(id): Path<i32>,
    Extension(requester): Extension<User>,
    req: Json<PodcastUpdateNameRequest>,
) -> Result<StatusCode, CustomError> {
    require_admin::<CustomError>(requester.is_admin()).map_err(map_podcast_error)?;
    let found_podcast = match PodcastService::get_podcast(id) {
        Ok(p) => Ok(p),
        Err(..) => Err(CustomErrorInner::NotFound(ErrorSeverity::Debug)),
    }?;

    PodcastService::update_podcast_name(found_podcast.id, &req.name)?;

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
    let rewriter = create_url_rewriter(&headers);
    let podcasts = PodcastService::get_favored_podcasts(requester)?
        .into_iter()
        .map(|p| p.with_rewritten_urls(&rewriter))
        .collect();
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

    let id_num = parse_podcast_id::<CustomError>(&id).map_err(map_podcast_error)?;
    PodcastService::update_active_podcast(id_num)?;
    Ok(StatusCode::OK)
}

#[async_recursion(?Send)]
async fn insert_outline(podcast: Outline, mut rng: ThreadRng, added_by: Option<i32>) {
    if !podcast.outlines.is_empty() {
        for outline_nested in podcast.clone().outlines {
            insert_outline(outline_nested, rng.clone(), added_by).await;
        }
        return;
    }
    let feed_url = podcast.clone().xml_url;
    if feed_url.is_none() {
        return;
    }

    let feed_response = get_http_client(&ENVIRONMENT_SERVICE)
        .get(feed_url.unwrap())
        .send()
        .await;
    if feed_response.is_err() {
        ChatServerHandle::broadcast_opml_error(feed_response.err().unwrap().to_string());
        return;
    }
    let content = feed_response.unwrap().bytes().await.unwrap();

    let channel = Channel::read_from(&content[..]);

    match channel {
        Ok(channel) => {
            let image_url = match channel.image {
                Some(ref image) => image.url.clone(),
                None => {
                    log::info!(
                        "No image found for podcast. Downloading from {}",
                        ENVIRONMENT_SERVICE.server_url.clone().to_owned() + DEFAULT_IMAGE_URL
                    );
                    ENVIRONMENT_SERVICE.server_url.clone().to_owned() + "ui/default.jpg"
                }
            };

            let inserted_podcast = PodcastService::handle_insert_of_podcast(
                PodcastInsertModel {
                    feed_url: podcast.clone().xml_url.expect("No feed url"),
                    title: channel.clone().title.to_string(),
                    id: rng.random::<i32>(),
                    image_url,
                },
                Some(channel),
                added_by,
            )
            .await;
            match inserted_podcast {
                Ok(podcast) => {
                    ChatServerHandle::broadcast_opml_added(&podcast);
                }
                Err(e) => {
                    ChatServerHandle::broadcast_opml_error(e.to_string());
                }
            }
        }
        Err(e) => {
            ChatServerHandle::broadcast_opml_error(e.to_string());
        }
    }
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
    Path(id): Path<i32>,
    Extension(requester): Extension<User>,
    Json(data): Json<DeletePodcast>,
) -> Result<StatusCode, CustomError> {
    require_privileged::<CustomError>(requester.is_privileged_user()).map_err(map_podcast_error)?;

    let podcast = PodcastService::get_podcast(id)?;
    if data.delete_files {
        spawn_blocking(move || FileService::delete_podcast_files(&podcast))
            .await
            .expect(
                "Error deleting \
        files",
            );
    }
    WatchtimeService::delete_watchtime(id)?;
    PodcastEpisodeService::delete_episodes_of_podcast(id)?;
    state.tag_service.delete_podcast_tags(id)?;

    PodcastService::delete_podcast(id)?;
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
) -> Result<axum::http::response::Response<Body>, CustomError> {
    println!("Got a request: {:?}", req);
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

    *req.uri_mut() = episode.url.parse().unwrap();
    let reqwest_to_make = reqwest::Request::try_from(req).expect(
        "http::Uri to url::Url conversion \
    failed",
    );

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(50))
        .build()
        .unwrap();
    let mut resp = client.execute(reqwest_to_make).await.unwrap();

    if resp.status().is_redirection()
        && let Some(location) = resp.headers().get("Location")
    {
        let redirect_url: String = location.to_str().unwrap().parse().unwrap();
        resp = client
            .get(redirect_url)
            .headers(cloned_headers)
            .send()
            .await
            .expect("http::Uri to url::Url conversion failed");
    }

    let mut response_builder = Response::builder().status(resp.status());
    *response_builder.headers_mut().unwrap() = resp.headers().clone();
    Ok(response_builder
        .body(Body::from_stream(resp.bytes_stream()))
        .unwrap())
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
) -> Result<axum::http::response::Response<Body>, CustomError> {
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
    Path(id_num): Path<i32>,
    Extension(requester): Extension<User>,
    Json(mut settings): Json<PodcastSetting>,
) -> Result<Json<PodcastSetting>, CustomError> {
    require_privileged::<CustomError>(requester.is_privileged_user()).map_err(map_podcast_error)?;
    settings.podcast_id = id_num;
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
    Path(id): Path<i32>,
    Extension(requester): Extension<User>,
) -> Result<Json<PodcastSetting>, CustomError> {
    require_privileged::<CustomError>(requester.is_privileged_user()).map_err(map_podcast_error)?;
    let settings = state.podcast_settings_service.get_settings(id)?;

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
        id: 0,
        auto_download: false,
        auto_update: false,
        auto_cleanup: false,
        auto_cleanup_days: 0,
        podcast_prefill: 0,
        replace_invalid_characters: false,
        use_existing_filename: false,
        replacement_strategy: "remove".to_string(),
        episode_format: "".to_string(),
        podcast_format: sample_string.0.content,
        direct_paths: true,
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
            .put(&format!("/api/v1/podcasts/{}/name", &saved_podcast.id))
            .json(&PodcastUpdateNameRequest {
                name: "New Podcast Name".to_string(),
            })
            .await;
        assert_eq!(resp.status_code(), 200);
        assert_eq!(
            crate::services::podcast::service::PodcastService::get_podcast(saved_podcast.id)
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

        let persisted =
            crate::services::podcast::service::PodcastService::get_podcast(saved_podcast.id)
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
            podcast_id: 0,
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
            Path(1),
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
            Path(1),
            Extension(non_privileged.clone()),
            Json(PodcastSetting::default()),
        )
        .await;
        match update_settings_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for update_podcast_settings"),
        }

        let get_settings_result =
            super::get_podcast_settings(State(app_state()), Path(1), Extension(non_privileged))
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
        let state = crate::app_state::AppState::new();
        let mut user =
            crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder::new()
                .build();
        user.username = format!("proxy-test-user-{}", uuid::Uuid::new_v4());
        let api_key = format!("proxy-test-key-{}", uuid::Uuid::new_v4());
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
