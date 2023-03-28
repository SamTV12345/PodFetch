use std::sync::{Mutex};
use std::thread;
use actix::Addr;
use actix_web::{HttpResponse, Responder, web};
use serde_json::{from_str, Value};
use crate::db::DB;
use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use actix_web::{get, post, put};
use actix_web::web::{Data, Path};
use crate::models::models::{PodCastAddModel, PodcastInsertModel};
use crate::unwrap_string;
use reqwest::{ClientBuilder as AsyncClientBuilder};
use tokio::task::spawn_blocking;
use crate::models::dto_models::PodcastFavorUpdateModel;
use crate::models::search_type::SearchType::{ITUNES, PODINDEX};
use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::rust_service::PodcastService;


#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Find a podcast by its collection id", body = [Podcast])
),
tag="podcasts"
)]
#[get("/podcast/{id}")]
    pub async fn find_podcast_by_id( id: Path<String>, db: Data<Mutex<DB>>, mapping_service: Data<Mutex<MappingService>>) -> impl Responder {
        let id_num = from_str::<i32>(&id).unwrap();
        let podcast = db.lock().expect("Error acquiring lock").get_podcast(id_num)
            .expect("Error getting podcast");
        let mapping_service = mapping_service.lock().expect("Error acquiring lock");
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
pub async fn find_all_podcasts(db: Data<Mutex<DB>>, mapping_service:Data<Mutex<MappingService>>) -> impl Responder {

    let mapping_service = mapping_service.lock().expect("Error acquiring lock");
    let podcasts = db.lock().expect("Error acquiring lock").get_podcasts().unwrap();

    let mapped_podcasts = podcasts
        .into_iter()
        .map(|podcast| mapping_service.map_podcast_to_podcast_dto(&podcast)).collect::<Vec<_>>();
    HttpResponse::Ok().json(mapped_podcasts)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Finds a podcast from the itunes url.", body = [ItunesModel])
),
tag="podcasts"
)]
#[get("/podcasts/{type_of}/{podcast}/search")]
pub async fn find_podcast(podcast_col: Path<(i32, String)>,
                          podcast_service: Data<Mutex<PodcastService>>) -> impl Responder {

    let (type_of, podcast) = podcast_col.into_inner();
    match type_of.try_into() {
        Ok(ITUNES) => {
            let mut podcast_service = podcast_service.lock().expect("Error locking podcastservice");
            log::debug!("Searching for podcast: {}", podcast);
            let res = podcast_service.find_podcast(&podcast).await;
            HttpResponse::Ok().json(res)
        }
        Ok(PODINDEX) => {
            let mut environment = EnvironmentService::new();

            if !environment.get_config().podindex_configured{
                return HttpResponse::BadRequest().json("Podindex is not configured");
            }
            let mut podcast_service = podcast_service.lock().expect("Error locking podcastservice");

            HttpResponse::Ok().json(podcast_service.find_podcast_on_podindex(&podcast).await)
        }
        Err(_)=> {
            HttpResponse::BadRequest().json("Invalid search type")
        }
    }
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Adds a podcast to the database.")),
tag="podcasts"
)]
#[post("/podcast/itunes")]
pub async fn add_podcast(track_id: web::Json<PodCastAddModel>,
                         lobby: Data<Addr<Lobby>>) ->
                                                                                             impl
Responder {
    let client = AsyncClientBuilder::new().build().unwrap();
    let res = client.get("https://itunes.apple.com/lookup?id=".to_owned()+&track_id
        .track_id
        .to_string())
        .send().await.unwrap();

    let res  = res.json::<Value>().await.unwrap();

    let mut podcast_service = PodcastService::new();
    let mapping_service= MappingService::new();
    podcast_service.handle_insert_of_podcast(PodcastInsertModel{
        feed_url: unwrap_string(&res["results"][0]["feedUrl"]),
        title: unwrap_string(&res["results"][0]["collectionName"]),
        id: unwrap_string(&res["results"][0]["collectionId"]).parse().unwrap(),
        image_url: unwrap_string(&res["results"][0]["artworkUrl600"])
    }, mapping_service, lobby).await;
    HttpResponse::Ok()
}

#[post("/podcast/podindex")]
pub async fn add_podcast_from_podindex(id: web::Json<PodCastAddModel>, lobby: Data<Addr<Lobby>>)
    ->impl Responder {
    let mut environment = EnvironmentService::new();

    if !environment.get_config().podindex_configured{
        return HttpResponse::BadRequest();
    }

    spawn_blocking(move || {
           start_download_podindex(id.track_id, lobby);
    });
    HttpResponse::Ok()
 }

fn start_download_podindex(id: i32, lobby: Data<Addr<Lobby>>){
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async{
        let mut podcast_service = PodcastService::new();
        podcast_service.insert_podcast_from_podindex(id, lobby).await;
    });
}

#[get("/podcasts/{podcast}/query")]
pub async fn query_for_podcast(podcast: Path<String>, podcast_service: Data<Mutex<PodcastEpisodeService>>) -> impl Responder {
    let mut podcast_service = podcast_service.lock().unwrap();
    let res = podcast_service.query_for_podcast(&podcast);

    HttpResponse::Ok().json(res)
}

#[post("/podcast/{id}/refresh")]
pub async fn download_podcast(id: Path<String>, lobby: Data<Addr<Lobby>>, podcast_service: Data<Mutex<PodcastService>>) -> impl Responder {
        let id_num = from_str::<i32>(&id).unwrap();
        let mut podcast_service = podcast_service.lock().unwrap();
        let podcast = podcast_service.get_podcast_by_id(id_num);
        thread::spawn(move || {
            let mut podcast_service = PodcastService::new();
            podcast_service.refresh_podcast(podcast.clone(), lobby);
        });
        HttpResponse::Ok().json("Refreshing podcast")
}

#[put("/podcast/favored")]
pub async fn favorite_podcast(update_model: web::Json<PodcastFavorUpdateModel> ,
                              podcast_service_mutex: Data<Mutex<PodcastService>>) -> impl Responder {
    let mut podcast_service = podcast_service_mutex.lock().unwrap();
    podcast_service.update_favor_podcast( update_model.id, update_model.favored);
    HttpResponse::Ok().json("Favorited podcast")
}

#[get("/podcasts/favored")]
pub async fn get_favored_podcasts(podcast_service_mutex: Data<Mutex<PodcastService>>) -> impl Responder {
    let mut podcast_service = podcast_service_mutex.lock().unwrap();
    let podcasts = podcast_service.get_favored_podcasts();
    HttpResponse::Ok().json(podcasts)
}

#[put("/podcast/{id}/active")]
pub async fn update_active_podcast(id: Path<String>, podcast_service_mutex: Data<Mutex<PodcastService>>) -> impl Responder {
    let id_num = from_str::<i32>(&id).unwrap();
    let mut podcast_service = podcast_service_mutex.lock().unwrap();
    podcast_service.update_active_podcast(id_num);
    HttpResponse::Ok().json("Updated active podcast")
}