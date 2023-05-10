use std::cmp::Ordering;
use crate::db::DB;
use crate::models::models::{PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel};
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder};
use std::sync::{Mutex};
use crate::DbPool;
use crate::models::episode::Episode;
use crate::models::user::User;
use crate::mutex::LockResultExt;

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Logs a watchtime request.")),
tag="watchtime"
)]
#[post("/podcast/episode")]
pub async fn log_watchtime(podcast_watch: web::Json<PodcastWatchedPostModel>, conn: Data<DbPool>,
                           requester: Option<web::ReqData<User>>) ->
                                                                                             impl
Responder {


    let podcast_episode_id = podcast_watch.0.podcast_episode_id.clone();
    DB::log_watchtime(&mut conn.get().unwrap(),podcast_watch.0, requester.unwrap().username.clone())
        .expect("Error logging watchtime");
    log::debug!("Logged watchtime for episode: {}", podcast_episode_id);
    HttpResponse::Ok()
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the last watched podcasts.")),
tag="watchtime"
)]
#[get("/podcast/episode/lastwatched")]
pub async fn get_last_watched(db: Data<Mutex<DB>>, conn: Data<DbPool>, requester: Option<web::ReqData<User>>) -> impl
Responder {

    let designated_username = requester.unwrap().username.clone();
    let mut db = db.lock().ignore_poison();
    let last_watched = db.get_last_watched_podcasts(&mut conn.get().unwrap(), designated_username
        .clone()).unwrap();

    let episodes = Episode::get_last_watched_episodes(designated_username, &mut conn.get().unwrap
        (),
    );

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
        if episodes_with_logs.iter().find(|e| e.episode_id == x.episode_id).is_none(){
            episodes_with_logs.push(x);
        }
    });
    episodes_with_logs.sort_by(|a,b| a.date.cmp(&b.date).reverse());
    HttpResponse::Ok().json(episodes_with_logs)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets watchtime by id.")),
tag="watchtime"
)]
#[get("/podcast/episode/{id}")]
pub async fn get_watchtime(id: web::Path<String>, conn: Data<DbPool>, requester: Option<web::ReqData<User>>) -> impl Responder {
    let designated_username = requester.unwrap().username.clone();
    let watchtime = DB::get_watchtime(&mut conn.get().unwrap(),&id, designated_username).unwrap();
    HttpResponse::Ok().json(watchtime)
}