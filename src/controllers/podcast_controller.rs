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
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{debug_handler, Extension, Json};
use axum_extra::extract::OptionalQuery;
use opml::{Outline, OPML};
use rand::rngs::ThreadRng;
use rand::Rng;
use rss::Channel;
use serde_json::{from_str, Value};
use std::thread;
use tokio::task::spawn_blocking;

use crate::models::filter::Filter;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcast_rssadd_model::PodcastRSSAddModel;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::service::file_service::{perform_podcast_variable_replacement, FileService};
use crate::utils::append_to_header::add_basic_auth_headers_conditionally;
use reqwest::header::HeaderMap;
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct PodcastSearchModel {
    order: Option<OrderCriteria>,
    title: Option<String>,
    order_option: Option<OrderOption>,
    favored_only: bool,
    tag: Option<String>,
}

#[derive(Serialize, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct PodcastSearchModelUtoipa {
    order: Option<String>,
    title: Option<String>,
    order_option: Option<String>,
    favored_only: bool,
    tag: Option<String>,
}

#[utoipa::path(
get,
path="/podcasts/filter",
responses(
(status = 200, description = "Gets the user specific filter.",body= Option<Filter>)),
tag="podcasts"
)]
pub async fn get_filter(
    Extension(requester): Extension<User>,
) -> Result<Json<Filter>, CustomError> {
    let filter = Filter::get_filter_by_username(&requester.username).await?;
    match filter {
        Some(f) => Ok(Json(f)),
        None => Err(CustomErrorInner::NotFound.into()),
    }
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
    Query(query): Query<PodcastSearchModelUtoipa>,
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<PodcastDto>>, CustomError> {
    let _order = query.order.map(|o| o.into()).unwrap_or(OrderCriteria::Asc);
    let _latest_pub = query
        .order_option
        .map(OrderOption::from_string)
        .unwrap_or(OrderOption::Title);
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
responses(
(status = 200, description = "Find a podcast by its collection id", body = PodcastDto)
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
responses(
(status = 200, description = "Gets all stored podcasts as a list", body = [PodcastDto])
),
tag="podcasts"
)]
pub async fn find_all_podcasts(
    requester: Extension<User>,
) -> Result<Json<Vec<PodcastDto>>, CustomError> {
    let username = &requester.username;

    let podcasts = PodcastService::get_podcasts(username)?;

    Ok(Json(podcasts))
}
use crate::models::itunes_models::PodcastSearchReturn;

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
            Ok(Json(PodcastSearchReturn::Itunes(res)))
        }
        Ok(Podindex) => {
            if !ENVIRONMENT_SERVICE.get_config().podindex_configured {
                return Err(
                    CustomErrorInner::BadRequest("Podindex is not configured".to_string()).into(),
                );
            }

            Ok(Json(PodcastSearchReturn::Podindex(
                PodcastService::find_podcast_on_podindex(&podcast).await?,
            )))
        }
        Err(_) => Err(CustomErrorInner::BadRequest("Invalid search type".to_string()).into()),
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
        None,
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
    let num = rand::rng().random_range(100..10000000);

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
            Some(channel),
        )
        .await?
        .into();
    }

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
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }
    let document = OPML::from_str(&opml.content).unwrap();

    spawn_blocking(move || {
        for outline in document.body.outlines {
            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                let rng = rand::rng();
                rt.block_on(insert_outline(outline.clone(), rng.clone()));
            });
        }
    });

    Ok(StatusCode::OK)
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
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    if !ENVIRONMENT_SERVICE.get_config().podindex_configured {
        return Err(CustomErrorInner::BadRequest("Podindex is not configured".to_string()).into());
    }

    spawn_blocking(move || match start_download_podindex(id.track_id) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error: {}", e)
        }
    });
    Ok(StatusCode::OK)
}

fn start_download_podindex(id: i32) -> Result<Podcast, CustomError> {
    let rt = Runtime::new().unwrap();

    rt.block_on(async { PodcastService::insert_podcast_from_podindex(id).await })
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
) -> Result<Json<Vec<PodcastEpisodeDto>>, CustomError> {
    let res = PodcastEpisodeService::query_for_podcast(&podcast)?
        .into_iter()
        .map(|p| (p, None::<User>, None::<FavoritePodcastEpisode>).into())
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
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let podcasts = Podcast::get_all_podcasts()?;
    thread::spawn(move || {
        for podcast in podcasts {
            let refresh_result = PodcastService::refresh_podcast(&podcast);
            if let Err(e) = refresh_result {
                log::error!("Error refreshing podcast: {}", e);
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
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let id_num = from_str::<i32>(&id).unwrap();
    let podcast = PodcastService::get_podcast_by_id(id_num);
    thread::spawn(move || {
        match PodcastService::refresh_podcast(&podcast) {
            Ok(_) => {
                log::info!("Succesfully refreshed podcast.");
            }
            Err(e) => {
                log::error!("Error refreshing podcast: {}", e);
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
    PodcastService::update_favor_podcast(
        update_model.id,
        update_model.favored,
        &requester.username,
    )?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PodcastUpdateNameRequest {
    name: String,
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
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden.into());
    }
    let found_podcast = match PodcastService::get_podcast(id) {
        Ok(p) => Ok(p),
        Err(..) => Err(CustomErrorInner::NotFound),
    }?;

    Podcast::update_podcast_name(found_podcast.id, &req.name)?;

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
) -> Result<Json<Vec<PodcastDto>>, CustomError> {
    let podcasts = PodcastService::get_favored_podcasts(requester.username)?;
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
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let id_num = from_str::<i32>(&id).unwrap();
    PodcastService::update_active_podcast(id_num)?;
    Ok(StatusCode::OK)
}

#[async_recursion(?Send)]
async fn insert_outline(podcast: Outline, mut rng: ThreadRng) {
    if !podcast.outlines.is_empty() {
        for outline_nested in podcast.clone().outlines {
            insert_outline(outline_nested, rng.clone()).await;
        }
        return;
    }
    let feed_url = podcast.clone().xml_url;
    if feed_url.is_none() {
        return;
    }

    let feed_response = get_http_client().get(feed_url.unwrap()).send().await;
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
use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::controllers::podcast_episode_controller::EpisodeFormatDto;
use crate::controllers::server::ChatServerHandle;
use crate::controllers::websocket_controller::RSSAPiKey;
use crate::models::episode::Episode;
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::favorites::Favorite;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_settings::PodcastSetting;
use crate::models::settings::Setting;
use crate::models::tag::Tag;
use crate::models::tags_podcast::TagsPodcast;
use crate::utils::environment_variables::is_env_var_present_and_true;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::utils::error::{map_reqwest_error, CustomError, CustomErrorInner};
use crate::utils::http_client::get_http_client;
use crate::utils::rss_feed_parser::PodcastParsed;

#[derive(Deserialize, ToSchema)]
pub struct DeletePodcast {
    pub delete_files: bool,
}

#[utoipa::path(
delete,
path="/podcasts/{id}",
request_body=DeletePodcast,
responses(
(status = 200, description = "Deletes a podcast by id")),
tag="podcasts"
)]
pub async fn delete_podcast(
    Path(id): Path<i32>,
    Extension(requester): Extension<User>,
    Json(data): Json<DeletePodcast>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let podcast = Podcast::get_podcast(id)?;
    if data.delete_files {
        spawn_blocking(move || FileService::delete_podcast_files(&podcast))
            .await
            .expect(
                "Error deleting \
        files",
            );
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

use axum::response::Response;

#[utoipa::path(
get,
path="/proxy/podcast",
responses(
(status = 200, description = "Proxies a podcast so people can stream podcasts from the remote \
server")),
tag="podcasts"
)]
pub(crate) async fn proxy_podcast(
    Query(params): Query<Params>,
    OptionalQuery(api_key): OptionalQuery<RSSAPiKey>,
    req: axum::extract::Request,
) -> Result<axum::http::response::Response<Body>, CustomError> {
    let mut req = req.map(|body| reqwest::Body::wrap_stream(body.into_data_stream()));
    let headers = req.headers_mut();

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
    for header in ["host", "referer", "sec-fetch-site"] {
        headers.remove(header);
    }

    add_basic_auth_headers_conditionally(episode.url.clone(), headers);

    *req.uri_mut() = episode.url.parse().unwrap();
    let reqwest_to_make = reqwest::Request::try_from(req).expect(
        "http::Uri to url::Url conversion \
    failed",
    );

    let client = reqwest::Client::new();
    let resp = client.execute(reqwest_to_make).await.unwrap();

    let mut response_builder = Response::builder().status(resp.status());
    *response_builder.headers_mut().unwrap() = resp.headers().clone();
    Ok(response_builder
        .body(Body::from_stream(resp.bytes_stream()))
        .unwrap())
}

#[utoipa::path(
    put,
    path="/podcasts/{id}/settings",
    responses(
(status = 200, description = "Updates the settings of a podcast by id")),
    tag="podcasts"
)]
pub async fn update_podcast_settings(
    Path(id_num): Path<i32>,
    Extension(requester): Extension<User>,
    Json(mut settings): Json<PodcastSetting>,
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
        None => Err(CustomErrorInner::NotFound.into()),
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
    let result = perform_podcast_variable_replacement(settings, podcast, None);

    match result {
        Ok(v) => Ok(Json(v)),
        Err(e) => Err(CustomErrorInner::BadRequest(e.to_string()).into()),
    }
}

pub fn get_podcast_router() -> OpenApiRouter {
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
}

#[cfg(test)]
pub mod tests {
    use crate::commands::startup::tests::handle_test_startup;
    use crate::controllers::podcast_controller::PodcastUpdateNameRequest;
    use crate::controllers::podcast_episode_controller::EpisodeFormatDto;
    use crate::models::podcasts::Podcast;
    use serial_test::serial;

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
        let saved_podcast = Podcast::add_podcast_to_database(
            "collection",
            "The homelab podcast",
            "https://example.com/feed",
            "https://example.com/image\
                                         .jpg",
            "test123",
        )
        .unwrap();
        let resp = ts_server
            .test_server
            .put(&format!("/api/v1/podcasts/{}/name", &saved_podcast.id))
            .json(&PodcastUpdateNameRequest {
                name: "New Podcast Name".to_string(),
            })
            .await;
        assert_eq!(
            Podcast::get_podcast(saved_podcast.id).unwrap().name,
            "New Podcast Name"
        );

        assert_eq!(resp.status_code(), 200);
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
}
