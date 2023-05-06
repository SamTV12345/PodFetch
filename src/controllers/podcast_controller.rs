use crate::models::dto_models::PodcastFavorUpdateModel;
use crate::models::models::{PodCastAddModel, PodcastInsertModel};
use crate::models::opml_model::OpmlModel;
use crate::models::search_type::SearchType::{ITUNES, PODINDEX};
use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::{DbPool, unwrap_string};
use actix::Addr;
use actix_web::web::{Data, Path};
use actix_web::{get, post, put, delete, HttpRequest};
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
use std::time::Duration;
use actix_web::http::header::LOCATION;
use diesel::SqliteConnection;
use tokio::task::spawn_blocking;
use crate::constants::constants::{PodcastType, STANDARD_USER};
use crate::db::DB;
use crate::exception::exceptions::PodFetchError;
use crate::models::user::User;
use crate::mutex::LockResultExt;
use crate::service::file_service::FileService;
use awc::Client as AwcClient;
use crate::models::itunes_models::Podcast;
use crate::models::messages::BroadcastMessage;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::podcast_rssadd_model::PodcastRSSAddModel;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastSearchModel{
    order: Option<OrderCriteria>,
    title: Option<String>,
    order_option: Option<OrderOption>,
    favored_only: bool
}

#[get("/podcasts/search")]
pub async fn search_podcasts(query: web::Query<PodcastSearchModel>, conn:Data<DbPool>,
                             podcast_service: Data<Mutex<PodcastService>>,
                             mapping_service:Data<Mutex<MappingService>>)
    ->impl Responder{
    let query = query.into_inner();
    let order = query.order.unwrap_or(OrderCriteria::ASC);
    let latest_pub = query.order_option.unwrap_or(OrderOption::Title);
    match query.favored_only {
        true => {
            let podcasts = podcast_service.lock().ignore_poison().search_podcasts_favored( order, query.title,
                                                                                   latest_pub,
                                                                                   mapping_service.lock()
                                                                                       .ignore_poison(),
                                                                                   &mut conn.get().unwrap
                                                                                   ()).unwrap();
            HttpResponse::Ok().json(podcasts)
        }
        false => {
            let podcasts = podcast_service.lock().ignore_poison().search_podcasts( order, query.title,
                                                                                   latest_pub,
                                                                                   mapping_service.lock()
                                                                                       .ignore_poison(),
                                                                                   &mut conn.get().unwrap
                                                                                   ()).unwrap();
            HttpResponse::Ok().json(podcasts)
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
) -> impl Responder {
    let id_num = from_str::<i32>(&id).unwrap();
    let podcast = PodcastService::get_podcast(&mut conn.get().unwrap(), id_num)
        .expect("Error getting podcast");
    let mapping_service = mapping_service.lock().ignore_poison();
    let mapped_podcast = mapping_service.map_podcast_to_podcast_dto(&podcast);
    HttpResponse::Ok().json(mapped_podcast)
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
    conn: Data<DbPool>, rq: HttpRequest
) -> impl Responder {
    let mapping_service = mapping_service
        .lock()
        .ignore_poison();
    let err_username = User::get_username_from_req_header(&rq);

    if err_username.is_err(){
        return HttpResponse::Unauthorized().json("Unauthorized");
    }

    let username = err_username.unwrap();
    let podcasts;

    match username {
        Some(u)=>{
             podcasts = PodcastService::get_podcasts(&mut conn.get().unwrap(), u, mapping_service).unwrap();
        },
        None => {
             podcasts = PodcastService::get_podcasts(&mut conn.get().unwrap(), STANDARD_USER
                .to_string(), mapping_service).unwrap();

        }
    }
    HttpResponse::Ok().json(podcasts)
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
) -> impl Responder {
    let (type_of, podcast) = podcast_col.into_inner();
    match type_of.try_into() {
        Ok(ITUNES) => {
            let mut podcast_service = podcast_service
                .lock()
                .ignore_poison();
            log::debug!("Searching for podcast: {}", podcast);
            let res = podcast_service.find_podcast(&podcast).await;
            HttpResponse::Ok().json(res)
        }
        Ok(PODINDEX) => {
            let mut environment = EnvironmentService::new();

            if !environment.get_config().podindex_configured {
                return HttpResponse::BadRequest().json("Podindex is not configured");
            }
            let mut podcast_service = podcast_service
                .lock()
                .expect("Error locking podcastservice");

            HttpResponse::Ok().json(podcast_service.find_podcast_on_podindex(&podcast).await)
        }
        Err(_) => HttpResponse::BadRequest().json("Invalid search type"),
    }
}

#[utoipa::path(
context_path="/api/v1",
request_body=PodCastAddModel,
responses(
(status = 200, description = "Adds a podcast to the database.")),
tag="podcasts"
)]
#[post("/podcast/itunes")]
pub async fn add_podcast(
    track_id: web::Json<PodCastAddModel>,
    lobby: Data<Addr<Lobby>>,
    conn: Data<DbPool>, rq: HttpRequest) -> impl Responder {

    return match  User::get_username_from_req_header(&rq){
      Ok(username)=>{

          match User::check_if_admin_or_uploader(&username, &mut conn.get().unwrap()){
              Some(err)=>{
                  return err;
              }
              None=>{
              }
          }
          let client = AsyncClientBuilder::new().build().unwrap();
          let res = client
              .get("https://itunes.apple.com/lookup?id=".to_owned() + &track_id.track_id
                  .to_string()+ "&entity=podcast")
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
              .await.expect("Error handling insert of podcast");
          HttpResponse::Ok()
      },
        Err(e)=>{
            return HttpResponse::BadRequest().json(e.to_string()).into();
        }
    }.into();
}

#[post("/podcast/feed")]
pub async fn add_podcast_by_feed(
    rss_feed: web::Json<PodcastRSSAddModel>,
    lobby: Data<Addr<Lobby>>,
    podcast_service: Data<Mutex<PodcastService>>,
    conn: Data<DbPool>, rq: HttpRequest) -> impl Responder {
    let mut podcast_service = podcast_service
        .lock()
        .ignore_poison();
    return match User::get_username_from_req_header(&rq) {
        Ok(username) => {
            match User::check_if_admin_or_uploader(&username, &mut conn.get().unwrap()) {
                Some(err) => {
                    return err;
                }
                None => {
                    let client = AsyncClientBuilder::new().build().unwrap();
                    let result = client.get(rss_feed.clone().rss_feed_url).send().await.unwrap();
                    let bytes = result.bytes().await.unwrap();
                    let channel = Channel::read_from(&*bytes).unwrap();
                    let num = rand::thread_rng().gen_range(100..10000000);

                    let res = podcast_service.handle_insert_of_podcast(
                        &mut conn.get().unwrap(),
                        PodcastInsertModel {
                            feed_url: rss_feed.clone().rss_feed_url.clone(),
                            title: channel.title.clone(),
                            id: num,
                            image_url: channel.image.map(|i| i.url).unwrap_or("".to_string()),
                        },
                        MappingService::new(),
                        lobby,
                    ).await.expect("Error handling insert of podcast");

                    HttpResponse::Ok().json(res)
                }
            }
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(e.to_string()).into();
        }
    }.into()
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
    rq: HttpRequest
) -> impl Responder {
    return match  User::get_username_from_req_header(&rq) {
        Ok(username) => {
            match User::check_if_admin_or_uploader(&username, &mut conn.get().unwrap()) {
                Some(err) => {
                    return err;
                }
                None => {
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

                    HttpResponse::Ok()
                }
            }
        }
        Err(e)=>{
            return HttpResponse::BadRequest().json(e.to_string()).into();
        }
    }.into();

}

#[utoipa::path(
context_path="/api/v1",
request_body=PodCastAddModel,
responses(
(status = 200, description = "Adds a podindex podcast to the database")),
tag="podcasts"
)]
#[post("/podcast/podindex")]
pub async fn add_podcast_from_podindex(
    id: web::Json<PodCastAddModel>,
    lobby: Data<Addr<Lobby>>,
    conn: Data<DbPool>,
    rq: HttpRequest
) -> impl Responder {
    let mut environment = EnvironmentService::new();

    if !environment.get_config().podindex_configured {
        return HttpResponse::BadRequest().json("Podindex is not configured");
    }

    return match  User::get_username_from_req_header(&rq) {
        Ok(username) => {
            match User::check_if_admin_or_uploader(&username, &mut conn.get().unwrap()) {
                Some(err) => {
                    return err;
                }
                None => {
                    spawn_blocking(move || {
                        match start_download_podindex(id.track_id, lobby, &mut conn.get().unwrap()) {
                            Ok(_) => {},
                            Err(e) => {
                                log::error!("Error: {}", e)
                            }
                        }
                    });
                    HttpResponse::Ok().into()
                }
            }
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(e.to_string()).into();
        }
    }
}

fn start_download_podindex(id: i32, lobby: Data<Addr<Lobby>>, conn: &mut SqliteConnection)
    ->Result<Podcast, PodFetchError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut podcast_service = PodcastService::new();
        podcast_service
            .insert_podcast_from_podindex(conn, id, lobby)
            .await
    })
}

#[utoipa::path(
context_path="/api/v1",
params(("podcast", description="The podcast episode query parameter.")),
responses(
(status = 200, description = "Queries for a podcast episode by a query string ")),
tag="podcasts"
)]
#[get("/podcasts/{podcast}/query")]
pub async fn query_for_podcast(
    podcast: Path<String>,
    podcast_service: Data<Mutex<PodcastEpisodeService>>,
) -> impl Responder {
    let mut podcast_service = podcast_service.lock()
        .ignore_poison();
    let res = podcast_service.query_for_podcast(&podcast);

    HttpResponse::Ok().json(res)
}

#[post("/podcast/all")]
pub async fn refresh_all_podcasts(lobby:Data<Addr<Lobby>>, podcast_service:
Data<Mutex<PodcastService>>, conn: Data<DbPool>)->impl Responder {
    let podcasts = DB::get_all_podcasts(&mut conn.get().unwrap());
    thread::spawn(move || {
    for podcast in podcasts.unwrap() {
        podcast_service.lock()
            .ignore_poison()
            .refresh_podcast(podcast.clone(), lobby.clone(), &mut conn.get()
            .unwrap());
        lobby.clone().do_send(BroadcastMessage {
            podcast_episode: None,
            type_of: PodcastType::RefreshPodcast,
            message: format!("Refreshed podcast: {}", podcast.name),
            podcast: Option::from(podcast.clone()),
            podcast_episodes: None,
        });
        }
    });
    HttpResponse::Ok()
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
    conn: Data<DbPool>
) -> impl Responder {
    let id_num = from_str::<i32>(&id).unwrap();
    let mut podcast_service = podcast_service.lock()
        .ignore_poison();
    let podcast = podcast_service.get_podcast_by_id(&mut conn.get().unwrap(),id_num);
    thread::spawn(move || {
        let mut podcast_service = PodcastService::new();
        podcast_service.refresh_podcast(podcast.clone(), lobby, &mut conn.get().unwrap());
    });
    HttpResponse::Ok().json("Refreshing podcast")
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
    rq:HttpRequest
) -> impl Responder {
    let mut podcast_service = podcast_service_mutex.lock()
        .ignore_poison();
    let username = User::get_username_from_req_header(&rq).map_err(|e| {
        log::error!("Error: {}", e);
        HttpResponse::InternalServerError().json("Error getting username from request header")
    }).unwrap();
    return match username {
        Some(username) => {
            podcast_service.update_favor_podcast(update_model.id, update_model.favored, username);
            HttpResponse::Ok().json("Favorited podcast")
        },
        None => {
            podcast_service.update_favor_podcast(update_model.id, update_model.favored,
                                                 STANDARD_USER.to_string());
            HttpResponse::Ok().json("Favorited podcast")
        }
    }
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Finds all favored podcasts.")),
tag="podcasts"
)]
#[get("/podcasts/favored")]
pub async fn get_favored_podcasts(
    podcast_service_mutex: Data<Mutex<PodcastService>>,rq: HttpRequest,
) -> impl Responder {

    let found_username = User::get_username_from_req_header(&rq).map_err(|e| {
        log::error!("Error: {}", e);
        HttpResponse::InternalServerError().json("Error getting username from request header")
    }).unwrap();
    let mut podcast_service = podcast_service_mutex.lock().ignore_poison();
    let podcasts = podcast_service.get_favored_podcasts(found_username);
    HttpResponse::Ok().json(podcasts)
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
    conn: Data<DbPool>
) -> impl Responder {
    let id_num = from_str::<i32>(&id).unwrap();
    PodcastService::update_active_podcast(&mut conn.get().unwrap(), id_num);
    HttpResponse::Ok().json("Updated active podcast")
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

#[derive(Deserialize)]
pub struct DeletePodcast {
    pub delete_files: bool
}

#[delete("/podcast/{id}")]
pub async fn delete_podcast(data: web::Json<DeletePodcast>, db: Data<DbPool>, id: Path<i32>)
                            ->impl Responder{
    let podcast = DB::get_podcast(&mut *db.get().unwrap(), id.clone()).expect("Error \
        finding podcast");
    if data.delete_files{
        FileService::delete_podcast_files(&podcast.directory_id);
    }

    DB::delete_watchtime(&mut *db.get().unwrap(), id.clone()).expect("Error deleting \
    watchtime");
    DB::delete_episodes_of_podcast(&mut *db.get().unwrap(), id.clone()).expect("Error deleting \
    episodes of podcast");
    DB::delete_podcast(&mut *db.get().unwrap(), id.clone());
    HttpResponse::Ok()
}
#[derive(Debug, Deserialize)]
pub struct Params {
    url: String,
}

#[get("/proxy/podcast")]
pub(crate) async fn proxy_podcast(
    req: HttpRequest,
    payload: web::Payload,
    params: web::Query<Params>
) -> HttpResponse {
    let new_url = params.url.clone();

    let forwarded_req = AwcClient::new()
        .request_from(new_url.as_str(), req.head())
        .timeout(Duration::from_secs(10))
        .no_decompress();

    let res = forwarded_req
        .send_stream(payload)
        .await;

    if res.is_err() {
        return HttpResponse::InternalServerError().json("Error proxying podcast");
    }

    let unwrapped_res = res.unwrap();
    let mut client_resp = HttpResponse::build(unwrapped_res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in unwrapped_res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
    }

    let streaming_res = client_resp.streaming(unwrapped_res);
    if streaming_res.status()==400{
        return HttpResponse::TemporaryRedirect().append_header((LOCATION,new_url)).finish()
    }
    streaming_res
}
