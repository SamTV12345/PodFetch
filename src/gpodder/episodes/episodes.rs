use std::ops::Deref;
use actix_web::{HttpResponse, Responder, web};

use actix_web::{get,post};
use actix_web::web::Data;
use crate::db::DB;
use crate::DbPool;
use crate::models::episode::{Episode, EpisodeAction, EpisodeDto};
use crate::models::models::PodcastWatchedPostModel;
use std::borrow::Borrow;
use crate::utils::time::get_current_timestamp;

#[derive(Serialize, Deserialize)]
pub struct EpisodeActionResponse{
    actions: Vec<Episode>,
    timestamp: i64
}

#[get("/episodes/{username}.json")]
pub async fn get_episode_actions(username: web::Path<String>, pool: Data<DbPool>) -> impl Responder {
    let actions = Episode::get_actions_by_username(username.clone(), &mut *pool.get().unwrap()).await;
    println!("actions: {:?}", actions);
    HttpResponse::Ok().json(EpisodeActionResponse{
        actions,
        timestamp: get_current_timestamp()
    })
}


#[post("/episodes/{username}.json")]
pub async fn upload_episode_actions(username: web::Path<String>, podcast_episode:
web::Json<Vec<EpisodeDto>>, conn: Data<DbPool>) -> impl
Responder {

    podcast_episode.iter().for_each(|episode| {
        let episode = Episode::convert_to_episode(episode, username.clone());
        Episode::insert_episode(episode.borrow(), &mut *conn.get().unwrap()).expect("TODO: panic message");

        if EpisodeAction::from_string(&episode.clone().action) == EpisodeAction::Play{
            let mut episode_url = episode.clone().episode;
            let mut test = episode.episode.split("?");
            let res = test.next();
            if res.is_some(){
                episode_url = res.unwrap().parse().unwrap()
            }
            let podcast_episode  = DB::query_podcast_episode_by_url(&mut *conn.get().unwrap(),
                                                                  &*episode_url);
            println!("Tres {:?}",podcast_episode.clone().unwrap());
            if podcast_episode.clone().unwrap().is_none(){
                return;
            }

            let model = PodcastWatchedPostModel{
                podcast_episode_id: podcast_episode.clone().unwrap().unwrap().episode_id,
                time: episode.position.unwrap() as i32,
            };
            DB::log_watchtime(&mut *conn.get().unwrap(), model, "admin".to_string())
                .expect("TODO: panic message");
            println!("episode: {:?}", episode);
        }

    });
    HttpResponse::Ok().json(EpisodeActionResponse{
        actions: vec![],
        timestamp: 0
    })
}