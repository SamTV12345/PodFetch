use crate::models::dto_models::PodcastFavorUpdateModel;
use crate::models::models::{PodcastAddModel, PodcastInsertModel};
use crate::models::opml_model::OpmlModel;
use crate::models::search_type::SearchType::{ITUNES, PODINDEX};
use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::{DbConnection, DbPool, get_default_image, unwrap_string};
use actix::Addr;
use actix_web::web::{Data, Path};
use actix_web::{get, post, put, delete, HttpRequest, error, Error};
use actix_web::{web, HttpResponse, Responder};
use async_recursion::async_recursion;
use futures::{executor};
use opml::{Outline, OPML};
use rand::rngs::ThreadRng;
use rand::Rng;
use reqwest::blocking::{Client, ClientBuilder as SyncClientBuilder};
use reqwest::{ClientBuilder as AsyncClientBuilder};
use rss::Channel;
use serde_json::{from_str, Value};
use std::sync::{Mutex};
use std::thread;
use actix_web::dev::PeerAddr;
use actix_web::http::{Method};
use tokio::task::spawn_blocking;
use crate::constants::constants::{PodcastType};

use crate::models::user::User;
use crate::mutex::LockResultExt;
use crate::service::file_service::FileService;
use futures_util::{StreamExt};
use reqwest::header::HeaderMap;
use tokio::sync::mpsc;
use crate::models::filter::Filter;
use crate::models::podcasts::Podcast;
use crate::models::messages::BroadcastMessage;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::podcast_rssadd_model::PodcastRSSAddModel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcast_history_item::PodcastHistoryItem;
use crate::utils::append_to_header::add_basic_auth_headers_conditionally;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastSearchModel{
    order: Option<OrderCriteria>,
    title: Option<String>,
    order_option: Option<OrderOption>,
    favored_only: bool
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the user specific filter.",body= Option<Filter>)),
tag="podcasts"
)]
#[get("/podcasts/filter")]
pub async fn get_filter(conn: Data<DbPool>, requester:
Option<web::ReqData<User>>) -> Result<HttpResponse,CustomError>{
            let filter = Filter::get_filter_by_username(requester.unwrap().username.clone(),
                                                        &mut conn.get().unwrap()).await?;
            Ok(HttpResponse::Ok().json(filter))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the podcasts matching the searching criteria",body=
Vec<Podcast>)),
tag="podcasts"
)]
#[get("/podcasts/search")]
pub async fn search_podcasts(query: web::Query<PodcastSearchModel>, conn:Data<DbPool>,
                             _podcast_service: Data<Mutex<PodcastService>>,
                             _mapping_service:Data<Mutex<MappingService>>, requester: Option<web::ReqData<User>>)
                             ->Result<HttpResponse,CustomError>{

    let query = query.into_inner();
    let _order = query.order.unwrap_or(OrderCriteria::ASC);
    let _latest_pub = query.order_option.unwrap_or(OrderOption::Title);
    let only_favored;
    let opt_filter = Filter::get_filter_by_username(requester.clone().unwrap().username.clone(),
                                                    &mut conn.get().unwrap()).await?;

    match opt_filter {
        Some(filter)=>{
            only_favored = filter.only_favored;
        },
        None=>{
            only_favored = true
        }
    }

    let username = requester.unwrap().username.clone();
    let filter = Filter::new(username.clone(), query.title.clone(), _order.clone().to_bool(),Some
                (_latest_pub.clone()
                .to_string()),only_favored);
    Filter::save_filter(filter, &mut *conn.get().unwrap())?;

    match query.favored_only {
        true => {
            let podcasts = _podcast_service.lock().ignore_poison().search_podcasts_favored( _order
                                                                                               .clone(), query.title,
                                                                                   _latest_pub
                                                                                       .clone(),
                                                                                   _mapping_service.lock()
                                                                                       .ignore_poison(),
                                                                                   &mut conn.get().unwrap
                                                                                   (),username)?;
            Ok(HttpResponse::Ok().json(podcasts))
        }
        false => {
            let podcasts = _podcast_service.lock().ignore_poison().search_podcasts( _order.clone(),
                                                                                   _mapping_service.lock().ignore_poison(),
                                                                                   query.title,
                                                                                   _latest_pub
                                                                                        .clone(),
                                                                                   &mut conn.get
                                                                                   ().unwrap(),
                                                                                    username)?;
            Ok(HttpResponse::Ok().json(podcasts))
        }
    }

}



#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Find a podcast by its collection id", body = [Podcast])
),
tag="podcasts"
)]
#[get("/podcast/{id}")]
pub async fn find_podcast_by_id(
    id: Path<String>,
    mapping_service: Data<Mutex<MappingService>>,
    conn: Data<DbPool>
) -> Result<HttpResponse, CustomError> {
    let id_num = from_str::<i32>(&id).unwrap();
    let podcast = PodcastService::get_podcast(&mut conn.get().unwrap(), id_num)?;
    let mapping_service = mapping_service.lock().ignore_poison();
    let mapped_podcast = mapping_service.map_podcast_to_podcast_dto(&podcast);
    Ok(HttpResponse::Ok().json(mapped_podcast))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all stored podcasts as a list", body = [Podcast])
),
tag="podcasts"
)]
#[get("/podcasts")]
pub async fn find_all_podcasts(
    mapping_service: Data<Mutex<MappingService>>,
    conn: Data<DbPool>, requester: Option<web::ReqData<User>>
) -> Result<HttpResponse, CustomError> {
    let mapping_service = mapping_service
        .lock()
        .ignore_poison();
    let username = requester.unwrap().username.clone();
    let podcasts;


    podcasts = PodcastService::get_podcasts(&mut conn.get().unwrap(), username, mapping_service)?;
    Ok(HttpResponse::Ok().json(podcasts))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Finds a podcast from the itunes url.", body = [ItunesModel])
),
tag="podcasts"
)]
#[get("/podcasts/{type_of}/{podcast}/search")]
pub async fn find_podcast(
    podcast_col: Path<(i32, String)>,
    podcast_service: Data<Mutex<PodcastService>>,
    requester: Option<web::ReqData<User>>
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_privileged_user(){
        return Err(CustomError::Forbidden)
    }

    let (type_of, podcast) = podcast_col.into_inner();
    return match type_of.try_into() {
        Ok(ITUNES) => {
            let mut podcast_service = podcast_service
                .lock()
                .ignore_poison();
            log::debug!("Searching for podcast: {}", podcast);
            let res = podcast_service.find_podcast(&podcast).await;
            Ok(HttpResponse::Ok().json(res))
        }
        Ok(PODINDEX) => {
            let mut environment = EnvironmentService::new();

            if !environment.get_config().podindex_configured {
                return Ok(HttpResponse::BadRequest().json("Podindex is not configured"));
            }
            let mut podcast_service = podcast_service
                .lock()
                .expect("Error locking podcastservice");

            Ok(HttpResponse::Ok().json(podcast_service.find_podcast_on_podindex(&podcast).await?))
        }
        Err(_) => Err(CustomError::BadRequest("Invalid search type".to_string()))
    }
}

#[utoipa::path(
context_path="/api/v1",
request_body=PodcastAddModel,
responses(
(status = 200, description = "Adds a podcast to the database.")),
tag="podcasts"
)]
#[post("/podcast/itunes")]
pub async fn add_podcast(
    track_id: web::Json<PodcastAddModel>,
    lobby: Data<Addr<Lobby>>,
    conn: Data<DbPool>, requester: Option<web::ReqData<User>>) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_privileged_user(){
        return Err(CustomError::Forbidden);
    }
    let client = AsyncClientBuilder::new().build().unwrap();

    let query:Vec<(&str, String)> = vec![
        ("id", track_id.track_id.to_string()),
        ("entity", "podcast".to_string())
    ];

    let res = client
              .get("https://itunes.apple.com/lookup")
              .query(&query)
              .send()
              .await
              .unwrap();

    let res = res.json::<Value>().await.unwrap();

    let mut podcast_service = PodcastService::new();
    let mapping_service = MappingService::new();
          podcast_service
              .handle_insert_of_podcast(&mut conn.get().unwrap(),
                                        PodcastInsertModel {
                                            feed_url: unwrap_string(&res["results"][0]["feedUrl"]),
                                            title: unwrap_string(&res["results"][0]["collectionName"]),
                                            id: unwrap_string(&res["results"][0]["collectionId"])
                                                .parse()
                                                .unwrap(),
                                            image_url: unwrap_string(&res["results"][0]["artworkUrl600"]),
                                        },
                                        mapping_service,
                                        lobby,
              )
              .await?;
    Ok(HttpResponse::Ok().into())
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Adds a podcast by its feed url",body=Podcast)),
tag="podcasts"
)]
#[post("/podcast/feed")]
pub async fn add_podcast_by_feed(
    rss_feed: web::Json<PodcastRSSAddModel>,
    lobby: Data<Addr<Lobby>>,
    podcast_service: Data<Mutex<PodcastService>>,
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>) -> Result<HttpResponse, CustomError> {
    let mut podcast_service = podcast_service
        .lock()
        .ignore_poison();

    if !requester.unwrap().is_privileged_user(){
        return Err(CustomError::Forbidden);
    }
    let client = AsyncClientBuilder::new().build().unwrap();
    let mut header_map = HeaderMap::new();
    add_basic_auth_headers_conditionally(rss_feed.clone().rss_feed_url, &mut header_map);
    let result = client.get(rss_feed.clone().rss_feed_url)
        .headers(header_map)
        .send()
        .await
        .map_err(map_reqwest_error)?;

    let bytes = result.bytes().await.unwrap();
    let channel = Channel::read_from(&*bytes).unwrap();
    let num = rand::thread_rng().gen_range(100..10000000);

    let res = podcast_service.handle_insert_of_podcast(
                        &mut conn.get().unwrap(),
                        PodcastInsertModel {
                            feed_url: rss_feed.clone().rss_feed_url.clone(),
                            title: channel.title.clone(),
                            id: num,
                            image_url: channel.image.map(|i| i.url)
                                .unwrap_or(get_default_image()),
                        },
                        MappingService::new(),
                        lobby,
                    ).await?;

    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
context_path="/api/v1",
request_body=OpmlModel,
responses(
(status = 200, description = "Adds all podcasts of an opml podcast list to the database.")),
tag="podcasts"
)]
#[post("/podcast/opml")]
pub async fn import_podcasts_from_opml(
    opml: web::Json<OpmlModel>,
    lobby: Data<Addr<Lobby>>,
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_privileged_user(){
        return Err(CustomError::Forbidden)
    }


   spawn_blocking(move || {
                        let rng = rand::thread_rng();
                        let environment = EnvironmentService::new();
                        let document = OPML::from_str(&opml.content).unwrap();

                        for outline in document.body.outlines {
                            let client = SyncClientBuilder::new().build().unwrap();
                            executor::block_on(insert_outline(outline.clone(), client.clone(), lobby.clone(), rng
                                .clone(), environment.clone(), conn.clone()));
                        }
                    });

    Ok(HttpResponse::Ok().into())
}

#[utoipa::path(
context_path="/api/v1",
request_body=PodcastAddModel,
responses(
(status = 200, description = "Adds a podindex podcast to the database")),
tag="podcasts"
)]
#[post("/podcast/podindex")]
pub async fn add_podcast_from_podindex(
    id: web::Json<PodcastAddModel>,
    lobby: Data<Addr<Lobby>>,
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>
) -> Result<HttpResponse, CustomError>{
    let mut environment = EnvironmentService::new();
    if !requester.unwrap().is_privileged_user(){
        return Err(CustomError::Forbidden);
    }

    if !environment.get_config().podindex_configured {
        return Err(CustomError::BadRequest("Podindex is not configured".to_string()));
    }

    spawn_blocking(move || {
                        match start_download_podindex(id.track_id, lobby, &mut conn.get().unwrap()) {
                            Ok(_) => {},
                            Err(e) => {
                                log::error!("Error: {}", e)
                            }
                        }
                    });
    Ok(HttpResponse::Ok().into())
}

fn start_download_podindex(id: i32, lobby: Data<Addr<Lobby>>, conn: &mut DbConnection)
    ->Result<Podcast, CustomError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let podcast = rt.block_on(async {
        let mut podcast_service = PodcastService::new();
        return podcast_service
            .insert_podcast_from_podindex(conn, id, lobby)
            .await;
    });
    return podcast
}

#[utoipa::path(
context_path="/api/v1",
params(("podcast", description="The podcast episode query parameter.")),
responses(
(status = 200, description = "Queries for a podcast episode by a query string", body = Vec<PodcastEpisode>)),
tag="podcasts",)]
#[get("/podcasts/{podcast}/query")]
pub async fn query_for_podcast(
    podcast: Path<String>,
    podcast_service: Data<Mutex<PodcastEpisodeService>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    let mut podcast_service = podcast_service.lock()
        .ignore_poison();
    let res = podcast_service.query_for_podcast(&podcast,&mut conn.get().unwrap())?;

    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Refreshes all podcasts")),
tag="podcasts"
)]
#[post("/podcast/all")]
pub async fn refresh_all_podcasts(lobby:Data<Addr<Lobby>>, podcast_service:
Data<Mutex<PodcastService>>, conn: Data<DbPool>, requester: Option<web::ReqData<User>>)->impl Responder {
    if !requester.unwrap().is_privileged_user(){
        return HttpResponse::Unauthorized().json("Unauthorized");
    }

    let podcasts = Podcast::get_all_podcasts(&mut conn.get().unwrap());
    thread::spawn(move || {
    for podcast in podcasts.unwrap() {
        podcast_service.lock()
            .ignore_poison()
            .refresh_podcast(podcast.clone(), lobby.clone(), &mut conn.get()
                .unwrap()).unwrap();
        lobby.clone().do_send(BroadcastMessage {
            podcast_episode: None,
            type_of: PodcastType::RefreshPodcast,
            message: format!("Refreshed podcast: {}", podcast.name),
            podcast: Option::from(podcast.clone()),
            podcast_episodes: None,
        });
        }
    });
    HttpResponse::Ok().into()
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Refreshes a podcast episode")),
tag="podcasts"
)]
#[post("/podcast/{id}/refresh")]
pub async fn download_podcast(
    id: Path<String>,
    lobby: Data<Addr<Lobby>>,
    podcast_service: Data<Mutex<PodcastService>>,
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_privileged_user(){
        return Err(CustomError::Forbidden);
    }

    let id_num = from_str::<i32>(&id).unwrap();
    let mut podcast_service = podcast_service.lock()
        .ignore_poison();
    let podcast = podcast_service.get_podcast_by_id(&mut conn.get().unwrap(),id_num);
    thread::spawn(move || {
        let mut podcast_service = PodcastService::new();
        podcast_service.refresh_podcast(podcast.clone(), lobby, &mut conn.get().unwrap()).unwrap();
    });
    Ok(HttpResponse::Ok().json("Refreshing podcast"))
}

#[utoipa::path(
context_path="/api/v1",
request_body=PodcastFavorUpdateModel,
responses(
(status = 200, description = "Updates favoring a podcast.", body=String)),
tag="podcasts"
)]
#[put("/podcast/favored")]
pub async fn favorite_podcast(
    update_model: web::Json<PodcastFavorUpdateModel>,
    podcast_service_mutex: Data<Mutex<PodcastService>>,
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>
) -> Result<HttpResponse, CustomError> {

    let mut podcast_service = podcast_service_mutex.lock()
        .ignore_poison();

    podcast_service.update_favor_podcast(update_model.id, update_model.favored,
                                         requester.unwrap().username.clone(), &mut conn.get()
            .unwrap())?;
    Ok(HttpResponse::Ok().json("Favorited podcast"))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Finds all favored podcasts.")),
tag="podcasts"
)]
#[get("/podcasts/favored")]
pub async fn get_favored_podcasts(
    podcast_service_mutex: Data<Mutex<PodcastService>>,requester: Option<web::ReqData<User>>,
    mapping_service: Data<Mutex<MappingService>>,
    conn: Data<DbPool>
) -> Result<HttpResponse, CustomError> {
    let mut podcast_service = podcast_service_mutex.lock().ignore_poison();
    let podcasts = podcast_service.get_favored_podcasts(requester.unwrap().username.clone(),
                                                        mapping_service.lock().ignore_poison()
                                                            .clone(), &mut conn.get().unwrap())?;
    Ok(HttpResponse::Ok().json(podcasts))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Updates the active state of a podcast. If inactive the podcast \
will not be refreshed automatically.")),
tag="podcasts"
)]
#[put("/podcast/{id}/active")]
pub async fn update_active_podcast(
    id: Path<String>,
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_privileged_user(){
        return Err(CustomError::Forbidden);
    }

    let id_num = from_str::<i32>(&id).unwrap();
    PodcastService::update_active_podcast(&mut conn.get().unwrap(), id_num)?;
    Ok(HttpResponse::Ok().json("Updated active podcast"))
}

#[async_recursion(?Send)]
async fn insert_outline(
    podcast: Outline,
    client: Client,
    lobby: Data<Addr<Lobby>>,
    mut rng: ThreadRng,
    environment: EnvironmentService,
    conn: Data<DbPool>
) {
    if podcast.outlines.len() > 0 {
        for outline_nested in podcast.clone().outlines {
            insert_outline(
                outline_nested,
                client.clone(),
                lobby.clone(),
                rng.clone(),
                environment.clone(),
                conn.clone()
            )
            .await;
        }
        return;
    }
    let feed_url = podcast.clone().xml_url.expect("No feed url");

    let content = client.get(feed_url).send().unwrap().bytes().unwrap();

    let channel = Channel::read_from(&content[..]);

    match channel{
        Ok(channel)=>{
            let mut podcast_service = PodcastService::new();
            let mapping_service = MappingService::new();

            let image_url = match channel.image {
                Some(image) => image.url,
                None => {
                    println!("No image found for podcast. Downloading from {}",environment.server_url
                        .clone().to_owned() + "ui/default.jpg");
                    environment.server_url.clone().to_owned() + "ui/default.jpg"
                },
            };

            let inserted_podcast = podcast_service
                .handle_insert_of_podcast(
                    &mut conn.get().unwrap(),
                    PodcastInsertModel {
                        feed_url: podcast.clone().xml_url.expect("No feed url"),
                        title: channel.title,
                        id: rng.gen::<i32>(),
                        image_url,
                    },
                    mapping_service,
                    lobby.clone(),
                )
                .await;
            match inserted_podcast {
                Ok(podcast)=>{
                    lobby.do_send(BroadcastMessage{
                        type_of: PodcastType::OpmlAdded,
                        message: "Refreshed podcasts".to_string(),
                        podcast: Option::from(podcast),
                        podcast_episodes: None,
                        podcast_episode: None,
                    })
                }
                Err(e)=>{
                    lobby.do_send(BroadcastMessage{
                        type_of: PodcastType::OpmlErrored,
                        message: e.to_string(),
                        podcast: None,
                        podcast_episodes: None,
                        podcast_episode: None,
                    })
                }
            }
        }
        Err(e)=>{
            lobby.do_send(BroadcastMessage{
                type_of: PodcastType::OpmlErrored,
                message: e.to_string(),
                podcast: None,
                podcast_episodes: None,
                podcast_episode: None,
            })
        }
    }


}
use utoipa::ToSchema;
use crate::utils::error::{CustomError, map_reqwest_error};

#[derive(Deserialize,ToSchema)]
pub struct DeletePodcast {
    pub delete_files: bool
}

#[utoipa::path(
context_path="/api/v1",
request_body=DeletePodcast,
responses(
(status = 200, description = "Deletes a podcast by id")),
tag="podcasts"
)]
#[delete("/podcast/{id}")]
pub async fn delete_podcast(data: web::Json<DeletePodcast>, db: Data<DbPool>, id: Path<i32>, requester: Option<web::ReqData<User>>)
                            -> Result<HttpResponse, CustomError>{
    if !requester.unwrap().is_privileged_user(){
        return Err(CustomError::Forbidden);
    }


    let podcast = Podcast::get_podcast(&mut *db.get().unwrap(), id.clone()).expect("Error \
        finding podcast");
    if data.delete_files{
        FileService::delete_podcast_files(&podcast.directory_name);
    }

    PodcastHistoryItem::delete_watchtime(&mut *db.get().unwrap(), id.clone()).expect("Error deleting \
    watchtime");
    PodcastEpisode::delete_episodes_of_podcast(&mut *db.get().unwrap(), id.clone()).expect("Error \
    deleting \
    episodes of podcast");
    Podcast::delete_podcast(&mut *db.get().unwrap(), id.clone())?;
    Ok(HttpResponse::Ok().into())
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    episode_id: String,
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Proxies a podcast so people can stream podcasts from the remote \
server")),
tag="podcasts"
)]
#[get("/proxy/podcast")]
pub(crate) async fn proxy_podcast(
    mut payload: web::Payload,
    params: web::Query<Params>,
    peer_addr: Option<PeerAddr>,
    method: Method,
    rq: HttpRequest,
    pool: Data<DbPool>
) -> Result<HttpResponse, Error> {
    let conn = &mut *pool.get().unwrap();

    let opt_res = PodcastEpisodeService::get_podcast_episode_by_id(conn, &params.episode_id)?;
    if opt_res.clone().is_none(){
        return Ok(HttpResponse::NotFound().finish());
    }
    let episode = opt_res.unwrap();
    let (tx, rx) = mpsc::unbounded_channel();

    actix_web::rt::spawn(async move {
        while let Some(chunk) = payload.next().await {
            tx.send(chunk).unwrap();
        }
    });

    let mut header_map = HeaderMap::new();

    for x in rq.headers() {
        if x.0 == "host"||x.0 == "referer"||x.0 == "sec-fetch-site"||x.0 == "sec-fetch-mode" {
            continue;
        }
        header_map.append(x.0.clone(), x.1.clone());
    }

    add_basic_auth_headers_conditionally(episode.clone().url,&mut header_map);
    // Required to not generate a 302 redirect
    header_map.append("sec-fetch-mode", "no-cors".parse().unwrap());
    header_map.append("sec-fetch-site", "cross-site".parse().unwrap());

    let forwarded_req = reqwest::Client::new()
        .request(method, episode.url)
        .headers(header_map)
        .fetch_mode_no_cors()
        .body(reqwest::Body::wrap_stream(UnboundedReceiverStream::new(rx)));

    let forwarded_req = match peer_addr {
        Some(PeerAddr(addr)) => forwarded_req.header("x-forwarded-for", addr.ip().to_string()),
        None => forwarded_req,
    };

    let res = forwarded_req
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut client_resp = HttpResponse::build(res.status());
    for (header_name, header_value) in res.headers().iter() {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
    }

    Ok(client_resp.streaming(res.bytes_stream()))
}
