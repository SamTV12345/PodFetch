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
use crate::utils::error::CustomError;

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
    PodcastHistoryItem::log_watchtime(&mut conn.get().unwrap(),podcast_watch.0, requester.unwrap().username
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

    let episodes = Episode::get_last_watched_episodes(designated_username, &mut conn.get().unwrap(),
    )?;

    let mut episodes_with_logs = last_watched.iter().map(|e|{
        let episode = episodes.iter().find(|e1| e1.episode_id == e.episode_id);
        match episode {
            Some(episode) => {
                if episode.watched_time>e.watched_time{
                    return episode
                }
                e
            },
            None => {
                e
            }
        }
    }).collect::<Vec<&PodcastWatchedEpisodeModelWithPodcastEpisode>>();

    episodes.iter().for_each(|x|{
        if !episodes_with_logs.iter().any(|e| e.episode_id == x.episode_id){
            episodes_with_logs.push(x);
        }
    });
    episodes_with_logs.sort_by(|a,b| a.date.cmp(&b.date).reverse());
    Ok(HttpResponse::Ok().json(episodes_with_logs))
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
    let watchtime = PodcastHistoryItem::get_watchtime(&mut conn.get().unwrap(),&id,
                                                      designated_username)?;
    Ok(HttpResponse::Ok().json(watchtime))
}