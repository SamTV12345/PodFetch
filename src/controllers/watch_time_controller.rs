use crate::db::DB;
use crate::models::models::{PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel};
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest};
use std::sync::{Mutex};
use crate::constants::constants::STANDARD_USER;
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
                           rq: HttpRequest
) ->
                                                                                             impl
Responder {

    let res = get_username(rq);
    if res.is_err(){
        return res.err().unwrap()
    }

    let designated_username = res.unwrap();
    let podcast_episode_id = podcast_watch.0.podcast_episode_id.clone();
    DB::log_watchtime(&mut conn.get().unwrap(),podcast_watch.0, designated_username)
        .expect("Error logging watchtime");
    log::debug!("Logged watchtime for episode: {}", podcast_episode_id);
    HttpResponse::Ok().into()
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the last watched podcasts.")),
tag="watchtime"
)]
#[get("/podcast/episode/lastwatched")]
pub async fn get_last_watched(db: Data<Mutex<DB>>, conn: Data<DbPool>, rq: HttpRequest) -> impl
Responder {

    let res = get_username(rq);
    if res.is_err(){
        return res.err().unwrap()
    }

    let designated_username = res.unwrap();
    let mut db = db.lock().ignore_poison();
    let last_watched = db.get_last_watched_podcasts(&mut conn.get().unwrap(), designated_username
        .clone()).unwrap();
    let episodes = Episode::get_last_watched_episodes(designated_username, &mut conn.get().unwrap
        (),
    );

    let episodes_with_logs = last_watched.iter().map(|e|{
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
    HttpResponse::Ok().json(episodes_with_logs)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets watchtime by id.")),
tag="watchtime"
)]
#[get("/podcast/episode/{id}")]
pub async fn get_watchtime(id: web::Path<String>, conn: Data<DbPool>, rq: HttpRequest) -> impl Responder {
    let res = get_username(rq);
    if res.is_err(){
        return res.err().unwrap()
    }

    let designated_username = res.unwrap();
    let watchtime = DB::get_watchtime(&mut conn.get().unwrap(),&id, designated_username).unwrap();
    HttpResponse::Ok().json(watchtime)
}


pub fn get_username(rq: HttpRequest) -> Result<String, HttpResponse> {
    let res = User::get_username_from_req_header(&rq);
    if res.is_err() {
        return Err(HttpResponse::Unauthorized().body("Unauthorized"))
    }
    let designated_username: String;

    match res.unwrap(){
        Some(username) => {
            designated_username = username;
        },
        None => {
            designated_username = STANDARD_USER.to_string();
        }
    }
    Ok(designated_username)
}