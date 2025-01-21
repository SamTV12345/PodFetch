use crate::constants::inner_constants::{
    BASIC_AUTH, COMMON_USER_AGENT, DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE, OIDC_AUTH,
};
use crate::models::dto_models::PodcastFavorUpdateModel;
use crate::models::misc_models::{PodcastAddModel, PodcastInsertModel};
use crate::models::opml_model::OpmlModel;
use crate::models::search_type::SearchType::{ITunes, Podindex};
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::{get_default_image, unwrap_string};
use async_recursion::async_recursion;
use opml::{Outline, OPML};
use rand::rngs::ThreadRng;
use rand::Rng;
use rss::Channel;
use serde_json::{from_str, Value};
use std::thread;
use axum::{Extension, Json, Router};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{Request, Response, StatusCode};
use axum::routing::{delete, get, post, put};
use futures::executor::block_on;
use tokio::task::spawn_blocking;

use crate::models::filter::Filter;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcast_rssadd_model::PodcastRSSAddModel;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::service::file_service::{perform_podcast_variable_replacement, FileService};
use crate::utils::append_to_header::add_basic_auth_headers_conditionally;
use futures_util::StreamExt;
use reqwest::header::HeaderMap;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastSearchModel {
    order: Option<OrderCriteria>,
    title: Option<String>,
    order_option: Option<OrderOption>,
    favored_only: bool,
    tag: Option<String>,
}

#[utoipa::path(
get,
path="/podcasts/filter",
context_path="/api/v1",
responses(
(status = 200, description = "Gets the user specific filter.",body= Option<Filter>)),
tag="podcasts"
)]
pub async fn get_filter(Extension(requester): Extension<User>) -> Result<Json<Filter>,
    CustomError> {
    let filter = Filter::get_filter_by_username(&requester.username).await?;
    match filter {
        Some(f)=> Ok(Json(f)),
        None=> Err(CustomErrorInner::NotFound.into())
    }
}

#[utoipa::path(
get,
path="/podcasts/search",
context_path="/api/v1",
responses(
(status = 200, description = "Gets the podcasts matching the searching criteria",body=
Vec<PodcastDto>)),
tag="podcasts"
)]
pub async fn search_podcasts(
    Query(query): Query<PodcastSearchModel>,
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<PodcastDto>>, CustomError> {
    let _order = query.order.unwrap_or(OrderCriteria::Asc);
    let _latest_pub = query.order_option.unwrap_or(OrderOption::Title);
    let tag = query.tag;

    let opt_filter = Filter::get_filter_by_username(&requester.username).await?;

    let only_favored = match opt_filter {
        Some(filter) => filter.only_favored,
        None => true,
    };

    let username = requester.username.clone();
    let filter = Filter::new(
        username.clone(),
        query.title.clone(),
        _order.clone().to_bool(),
        Some(_latest_pub.clone().to_string()),
        only_favored,
    );
    Filter::save_filter(filter)?;

    match query.favored_only {
        true => {
            let podcasts;
            {
                podcasts = PodcastService::search_podcasts_favored(
                    _order.clone(),
                    query.title,
                    _latest_pub.clone(),
                    username,
                    tag,
                )?;
            }
            Ok(Json(podcasts))
        }
        false => {
            let podcasts;
            {
                podcasts = PodcastService::search_podcasts(
                    _order.clone(),
                    query.title,
                    _latest_pub.clone(),
                    username,
                    tag,
                )?;
            }
            Ok(Json(podcasts))
        }
    }
}

#[utoipa::path(
get,
path="/podcasts/{id}",
context_path="/api/v1",
responses(
(status = 200, description = "Find a podcast by its collection id", body = [PodcastDto])
),
tag="podcasts"
)]
pub async fn find_podcast_by_id(
    Path(id): Path<String>,
    Extension(user): Extension<User>,
) -> Result<Json<PodcastDto>, CustomError> {
    let id_num = from_str::<i32>(&id).unwrap();
    let username = &user.username;

    let podcast = PodcastService::get_podcast(id_num)?;
    let tags = Tag::get_tags_of_podcast(id_num, username)?;
    let favorite = Favorite::get_favored_podcast_by_username_and_podcast_id(username, id_num)?;
    let podcast_dto: PodcastDto = (podcast, favorite, tags).into();
    Ok(Json(podcast_dto))
}

#[utoipa::path(
get,
path="/podcasts",
context_path="/api/v1",
responses(
(status = 200, description = "Gets all stored podcasts as a list", body = [PodcastDto])
),
tag="podcasts"
)]
pub async fn find_all_podcasts(requester: Extension<User>) -> Result<Json<Vec<PodcastDto>>, CustomError> {
    let username = &requester.username;

    let podcasts = PodcastService::get_podcasts(username)?;

    Ok(Json(podcasts))
}
use crate::models::itunes_models::ItunesModel;

#[utoipa::path(
get,
path="/podcasts/{type_of}/{podcast}/search",
context_path="/api/v1",
responses(
(status = 200, description = "Finds a podcast from the itunes url.", body = [ItunesModel])
),
tag="podcasts"
)]
pub async fn find_podcast(
    Path(podcast_col): Path<(i32, String)>,
    Extension(requester): Extension<User>,
) -> Result<Json<Value>, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let (type_of, podcast) = podcast_col;
    match type_of.try_into() {
        Ok(ITunes) => {
            let res;
            {
                log::debug!("Searching for podcast: {}", podcast);
                res = PodcastService::find_podcast(&podcast).await;
            }
            Ok(Json(res))
        }
        Ok(Podindex) => {
            if !ENVIRONMENT_SERVICE.get_config().podindex_configured {
                return Err(CustomErrorInner::BadRequest("Podindex is not configured".to_string())
                    .into());
            }

            Ok(Json(PodcastService::find_podcast_on_podindex(&podcast).await?))
        }
        Err(_) => Err(CustomErrorInner::BadRequest("Invalid search type".to_string()).into()),
    }
}

#[utoipa::path(
post,
path="/podcast/itunes",
context_path="/api/v1",
request_body=PodcastAddModel,
responses(
(status = 200, description = "Adds a podcast to the database.")),
tag="podcasts"
)]
pub async fn add_podcast(
    track_id: Json<PodcastAddModel>,
    State(lobby): State<ChatServerHandle>,
    requester: Extension<User>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let query: Vec<(&str, String)> = vec![
        ("id", track_id.track_id.to_string()),
        ("entity", "podcast".to_string()),
    ];

    let res = get_http_client()
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
        lobby,
        None,
    )
    .await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
post,
path="/podcast/feed",
context_path="/api/v1",
responses(
(status = 200, description = "Adds a podcast by its feed url",body=PodcastDto)),
tag="podcasts"
)]
pub async fn add_podcast_by_feed(
    Json(rss_feed): Json<PodcastRSSAddModel>,
    State(lobby): State<ChatServerHandle>,
    Extension(requester): Extension<User>,
) -> Result<Json<PodcastDto>, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }
    let mut header_map = HeaderMap::new();
    header_map.insert("User-Agent", COMMON_USER_AGENT.parse().unwrap());
    add_basic_auth_headers_conditionally(rss_feed.clone().rss_feed_url, &mut header_map);
    let result = get_http_client()
        .get(rss_feed.clone().rss_feed_url)
        .headers(header_map)
        .send()
        .await
        .map_err(map_reqwest_error)?;

    let bytes = result.text().await.unwrap();

    let channel = Channel::read_from(bytes.as_bytes()).unwrap();
    let num = rand::thread_rng().gen_range(100..10000000);

    let res: PodcastDto;
    {
        res = PodcastService::handle_insert_of_podcast(
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
            lobby,
            Some(channel),
        )
        .await?
        .into();
    }

    Ok(Json(res))
}

#[utoipa::path(
post,
path="/podcast/opml",
context_path="/api/v1",
request_body=OpmlModel,
responses(
(status = 200, description = "Adds all podcasts of an opml podcast list to the database.")),
tag="podcasts"
)]
pub async fn import_podcasts_from_opml(
    Json(opml): Json<OpmlModel>,
    State(lobby): State<ChatServerHandle>,
    requester: Extension<User>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }
    let document = OPML::from_str(&opml.content).unwrap();

    spawn_blocking(move || {
        for outline in document.body.outlines {
            let moved_lobby = lobby.clone();
            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                let rng = rand::thread_rng();
                rt.block_on(insert_outline(outline.clone(), moved_lobby, rng.clone()));
            });
        }
    });

    Ok(StatusCode::OK)
}

#[utoipa::path(
post,
path="/podcast/podindex",
context_path="/api/v1",
request_body=PodcastAddModel,
responses(
(status = 200, description = "Adds a podindex podcast to the database")),
tag="podcasts"
)]
pub async fn add_podcast_from_podindex(
    Json(id): Json<PodcastAddModel>,
    State(lobby): State<ChatServerHandle>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    if !ENVIRONMENT_SERVICE.get_config().podindex_configured {
        return Err(CustomErrorInner::BadRequest("Podindex is not configured".to_string()).into());
    }

    spawn_blocking(move || match start_download_podindex(id.track_id, lobby) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error: {}", e)
        }
    });
    Ok(StatusCode::OK)
}

fn start_download_podindex(id: i32, State(lobby): ChatServerHandle) -> Result<Podcast,
    CustomError> {
    let rt = Runtime::new().unwrap();

    rt.block_on(async { PodcastService::insert_podcast_from_podindex(id, lobby).await })
}

#[utoipa::path(
get,
path="/podcasts/{podcast}/query",
context_path="/api/v1",
params(("podcast", description="The podcast episode query parameter.")),
responses(
(status = 200, description = "Queries for a podcast episode by a query string", body = Vec<PodcastEpisode>)),
tag="podcasts",)]
pub async fn query_for_podcast(podcast: Path<String>) -> Result<Json<Vec<PodcastEpisode>>, CustomError> {
    let res = PodcastEpisodeService::query_for_podcast(&podcast)?;

    Ok(Json(res))
}

#[utoipa::path(
post,
path="/podcast/all",
context_path="/api/v1",
responses(
(status = 200, description = "Refreshes all podcasts")),
tag="podcasts"
)]
pub async fn refresh_all_podcasts(
    State(lobby): State<ChatServerHandle>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let podcasts = Podcast::get_all_podcasts()?;
    thread::spawn(move || {
        for podcast in podcasts {
            PodcastService::refresh_podcast(podcast.clone(), lobby.clone()).unwrap();
            lobby.broadcast_podcast_refreshed(&podcast);
        }
    });
    Ok(StatusCode::OK)
}

#[utoipa::path(
post,
path="/podcast/{id}/refresh",
context_path="/api/v1",
responses(
(status = 200, description = "Refreshes a podcast episode")),
tag="podcasts"
)]
pub async fn download_podcast(
    Path(id): Path<String>,
    State(lobby): State<ChatServerHandle>,
    Extension(requester): Extension<User>,
) -> Result<impl Into<String>, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let id_num = from_str::<i32>(&id).unwrap();
    let podcast = PodcastService::get_podcast_by_id(id_num);
    thread::spawn(move || {
        match PodcastService::refresh_podcast(podcast.clone(), lobby.clone()) {
            Ok(_) => {
                log::info!("Succesfully refreshed podcast.");
            }
            Err(e) => {
                log::error!("Error refreshing podcast: {}", e);
            }
        }

        let download = PodcastService::schedule_episode_download(podcast.clone(), Some(lobby));

        if download.is_err() {
            log::error!("Error downloading podcast: {}", download.err().unwrap());
        }
    });

    Ok("Refreshing podcast")
}

#[utoipa::path(
put,
path="/podcast/favored",
context_path="/api/v1",
request_body=PodcastFavorUpdateModel,
responses(
(status = 200, description = "Updates favoring a podcast.", body=String)),
tag="podcasts"
)]
pub async fn favorite_podcast(
    update_model: Json<PodcastFavorUpdateModel>,
    requester: Extension<User>,
) -> Result<StatusCode, CustomError> {
    PodcastService::update_favor_podcast(
        update_model.id,
        update_model.favored,
        &requester.username,
    )?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
get,
path="/podcasts/favored",
context_path="/api/v1",
responses(
(status = 200, description = "Finds all favored podcasts.")),
tag="podcasts"
)]
pub async fn get_favored_podcasts(
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<PodcastDto>>, CustomError> {
    let podcasts = PodcastService::get_favored_podcasts(requester.username)?;
    Ok(Json(podcasts))
}

#[utoipa::path(
put,
path="/podcast/{id}/active",
context_path="/api/v1",
responses(
(status = 200, description = "Updates the active state of a podcast. If inactive the podcast \
will not be refreshed automatically.")),
tag="podcasts"
)]
pub async fn update_active_podcast(
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let id_num = from_str::<i32>(&id).unwrap();
    PodcastService::update_active_podcast(id_num)?;
    Ok(StatusCode::OK)
}

#[async_recursion(?Send)]
async fn insert_outline(podcast: Outline, lobby: ChatServerHandle, mut rng: ThreadRng) {
    if !podcast.outlines.is_empty() {
        for outline_nested in podcast.clone().outlines {
            insert_outline(outline_nested, lobby.clone(), rng.clone()).await;
        }
        return;
    }
    let feed_url = podcast.clone().xml_url;
    if feed_url.is_none() {
        return;
    }

    let feed_response = get_http_client().get(feed_url.unwrap()).send().await;
    if feed_response.is_err() {
        lobby.broadcast_opml_error(feed_response.err().unwrap().to_string());
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
                    id: rng.gen::<i32>(),
                    image_url,
                },
                lobby.clone(),
                Some(channel),
            )
            .await;
            match inserted_podcast {
                Ok(podcast) => {
                    lobby.broadcast_opml_added(&podcast);
                }
                Err(e) => {
                    lobby.broadcast_opml_error(e.to_string());
                }
            }
        }
        Err(e) => {
            lobby.broadcast_opml_error(e.to_string());
        }
    }
}
use crate::models::episode::Episode;
use crate::models::tag::Tag;
use utoipa::ToSchema;
use crate::controllers::podcast_episode_controller::EpisodeFormatDto;
use crate::controllers::server::ChatServerHandle;
use crate::controllers::websocket_controller::RSSAPiKey;
use crate::models::favorites::Favorite;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_settings::PodcastSetting;
use crate::models::settings::Setting;
use crate::models::tags_podcast::TagsPodcast;
use crate::utils::environment_variables::is_env_var_present_and_true;

use crate::utils::error::{map_reqwest_error, CustomError, CustomErrorInner};
use crate::utils::http_client::get_http_client;
use crate::utils::rss_feed_parser::PodcastParsed;

#[derive(Deserialize, ToSchema)]
pub struct DeletePodcast {
    pub delete_files: bool,
}

#[utoipa::path(
delete,
path="/podcast/{id}",
context_path="/api/v1",
request_body=DeletePodcast,
responses(
(status = 200, description = "Deletes a podcast by id")),
tag="podcasts"
)]
pub async fn delete_podcast(
    Json(data): Json<DeletePodcast>,
    Path(id): Path<i32>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let podcast = Podcast::get_podcast(id)?;
    if data.delete_files {
        spawn_blocking(move ||FileService::delete_podcast_files(&podcast)).await.expect("Error deleting \
        files");
    }
    Episode::delete_watchtime(id)?;
    PodcastEpisode::delete_episodes_of_podcast(id)?;
    TagsPodcast::delete_tags_by_podcast_id(id)?;

    Podcast::delete_podcast(id)?;
    Ok(StatusCode::OK)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    episode_id: String,
}

#[utoipa::path(
get,
path="/proxy/podcast",
context_path="/api/v1",
responses(
(status = 200, description = "Proxies a podcast so people can stream podcasts from the remote \
server")),
tag="podcasts"
)]
pub(crate) async fn proxy_podcast(
    Query(params): Query<Params>,
    Query(api_key): Query<Option<RSSAPiKey>>,
    rq: Request<axum::body::Body>,
) -> Result<Body, CustomError> {
    let is_auth_enabled =
        is_env_var_present_and_true(BASIC_AUTH) || is_env_var_present_and_true(OIDC_AUTH);

    if is_auth_enabled {
        if api_key.is_none() {
            return Err(CustomErrorInner::Forbidden.into());
        }
        let api_key = api_key.unwrap().api_key;

        let api_key_exists = User::check_if_api_key_exists(&api_key);

        if !api_key_exists {
            return Err(CustomErrorInner::Forbidden.into());
        }
    }

    let opt_res = PodcastEpisodeService::get_podcast_episode_by_id(&params.episode_id)?;
    if opt_res.is_none() {
        return Err(CustomErrorInner::NotFound.into());
    }
    let episode = opt_res.unwrap();
    let mut header_map = HeaderMap::new();
    for (header, value) in rq.headers().iter() {
        if header == "host" || header == "referer" || header == "sec-fetch-site" || header == "sec-fetch-mode" {
            continue;
        }
        header_map.insert(header.clone(), value.clone());
    }

    add_basic_auth_headers_conditionally(episode.url.clone(), &mut header_map);

    let client = reqwest::Client::new();
    let res = client
        .request(rq.method().clone(), &episode.url)
        .headers(header_map)
        .body(rq.body())
        .send()
        .await
        .map_err(|e| CustomErrorInner::Unknown)?;

    let stream = res.bytes_stream();
    Ok(Body::from_stream(stream))
}

#[utoipa::path(
    get,
    path="/podcasts/{id}/settings",
    context_path="/api/v1",
    responses(
(status = 200, description = "Updates the settings of a podcast by id")),
    tag="podcasts"
)]
pub async fn update_podcast_settings(
    Path(id_num): Path<i32>,
    Json(mut settings): Json<PodcastSetting>,
    Extension(requester): Extension<User>,
) -> Result<Json<PodcastSetting>, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }
    settings.podcast_id = id_num;
    let updated_podcast = PodcastSetting::update_settings(&settings)?;

    Ok(Json(updated_podcast))
}

#[utoipa::path(
    get,
    path="/podcasts/{id}/settings",
    context_path="/api/v1",
    responses(
(status = 200, description = "Gets the settings of a podcast by id")),
    tag="podcasts"
)]
pub async fn get_podcast_settings(
    Path(id): Path<i32>,
    Extension(requester): Extension<User>,
) -> Result<Json<PodcastSetting>, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }
    let settings = PodcastSetting::get_settings(id)?;

    match settings {
        None => {
            Err(CustomErrorInner::NotFound.into())
        }
        Some(s) => {
            Ok(Json(s))
        }
    }
}

#[utoipa::path(
    post,
    path="/podcasts/formatting",
    context_path="/api/v1",
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
    let result = perform_podcast_variable_replacement(settings, podcast, None);

    match result {
        Ok(v) => Ok(Json(v)),
        Err(e) => Err(CustomErrorInner::BadRequest(e.to_string()).into()),
    }
}

pub fn get_podcast_router() -> Router {
    Router::new()
                .route("/podcasts/filter", get(get_filter))
                .route("/podcasts/search", get(search_podcasts))
                .route("/podcasts/{id}", get(find_podcast_by_id))
                .route("/podcasts", get(find_all_podcasts))
                .route("/podcasts/{type_of}/{podcast}/search", get(find_podcast))
                .route("/podcast/itunes", post(add_podcast))
                .route("/podcast/feed", post(add_podcast_by_feed))
                .route("/podcast/opml", post(import_podcasts_from_opml))
                .route("/podcast/podindex", post(add_podcast_from_podindex))
                .route("/podcasts/favored", get(get_favored_podcasts))
                .route("/podcast/all", post(refresh_all_podcasts))
                .route("/podcast/{id}/refresh", post(download_podcast))
                .route("/podcast/favored", put(favorite_podcast))
                .route("/podcast/{id}/active", put(update_active_podcast))
                .route("/podcast/{id}", delete(delete_podcast))
                .route("/proxy/podcast", get(proxy_podcast))
                .route("/podcasts/{id}/settings", get(update_podcast_settings))
                .route("/podcasts/{id}/settings", get(get_podcast_settings))
                .route("/podcasts/formatting", post(retrieve_podcast_sample_format))
                .route("/podcasts/{id}/query", get(query_for_podcast))
}
