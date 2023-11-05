use std::collections::HashMap;
use std::ops::DerefMut;
use crate::models::misc_models::{PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel};
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse};
use std::sync::{Mutex};
use crate::config::dbconfig::establish_connection;
use crate::DbPool;
use crate::models::episode::Episode;
use crate::models::podcast_history_item::PodcastHistoryItem;
use crate::models::user::User;
use crate::mutex::LockResultExt;
use crate::service::mapping_service::MappingService;
use crate::utils::error::{CustomError, map_r2d2_error};

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Logs a watchtime request.")),
tag="watchtime"
)]
#[post("/podcast/episode")]
pub async fn log_watchtime(podcast_watch: web::Json<PodcastWatchedPostModel>, conn: Data<DbPool>,
                           requester: Option<web::ReqData<User>>) -> Result<HttpResponse, CustomError> {


    let podcast_episode_id = podcast_watch.0.podcast_episode_id.clone();
    PodcastHistoryItem::log_watchtime(conn.get().map_err(map_r2d2_error)?.deref_mut(),podcast_watch.0, requester.unwrap().username
        .clone())?;
    log::debug!("Logged watchtime for episode: {}", podcast_episode_id);
    Ok(HttpResponse::Ok().body("Watchtime logged."))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the last watched podcast episodes.")),
tag="watchtime"
)]
#[get("/podcast/episode/lastwatched")]
pub async fn get_last_watched(conn: Data<DbPool>, requester:
Option<web::ReqData<User>>, mapping_service:Data<Mutex<MappingService>>) -> Result<HttpResponse, CustomError> {

    let designated_username = requester.unwrap().username.clone();
    let last_watched = PodcastHistoryItem::get_last_watched_podcasts(&mut establish_connection(),
                                                                     designated_username
        .clone(), mapping_service.lock().ignore_poison().clone()).unwrap();

    let episodes = Episode::get_last_watched_episodes(designated_username,
                                                          conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;
    let mut last_watched_episodes:HashMap<String,
        PodcastWatchedEpisodeModelWithPodcastEpisode> = HashMap::from_iter(last_watched.iter().map(|e| (e
                                                                                             .episode_id
                                                                                     .clone(), e.clone())));

    episodes.iter().for_each(|v|{
      match last_watched_episodes.contains_key(&v.episode_id){
            true => {
                let e1 = last_watched_episodes.get(&v.episode_id).unwrap();
                if e1.date<v.date{
                    last_watched_episodes.insert(v.episode_id.clone(), v.clone());
                }
            },
            false => {
                last_watched_episodes.insert(v.episode_id.clone(), v.clone());
            }
      }
    });

    let mut extracted_values = last_watched_episodes
        .values()
        .cloned()
        .collect::<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>>();
    extracted_values.sort_by(|a,b| a.date.cmp(&b.date).reverse());
    Ok(HttpResponse::Ok().json(extracted_values))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets watchtime by id.")),
tag="watchtime"
)]
#[get("/podcast/episode/{id}")]
pub async fn get_watchtime(id: web::Path<String>, conn: Data<DbPool>, requester:
Option<web::ReqData<User>>) -> Result<HttpResponse, CustomError> {
    let designated_username = requester.unwrap().username.clone();
    let watchtime = PodcastHistoryItem::get_watchtime(conn.get().map_err(map_r2d2_error)?.deref_mut(),&id,
                                                      designated_username)?;
    Ok(HttpResponse::Ok().json(watchtime))
}