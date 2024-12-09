use actix_web::{web, HttpResponse};
use std::ops::DerefMut;

use actix_web::web::Data;
use actix_web::{get, post};

use crate::models::episode::{Episode, EpisodeDto};
use crate::models::session::Session;
use crate::utils::error::{map_r2d2_error, CustomError};
use crate::utils::time::get_current_timestamp;
use crate::DbPool;
use chrono::{DateTime};

#[derive(Serialize, Deserialize)]
pub struct EpisodeActionResponse {
    actions: Vec<Episode>,
    timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct EpisodeActionPostResponse {
    update_urls: Vec<String>,
    timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct EpisodeSinceRequest {
    since: i64,
    podcast: Option<String>,
    device: Option<String>,
    aggregate: Option<String>,
}

#[get("/episodes/{username}.json")]
pub async fn get_episode_actions(
    username: web::Path<String>,
    opt_flag: Option<web::ReqData<Session>>,
    since: web::Query<EpisodeSinceRequest>,
) -> Result<HttpResponse, CustomError> {
    match opt_flag {
        Some(flag) => {
            let username = username.clone();
            if flag.username != username.clone() {
                return Err(CustomError::Forbidden);
            }

            let since_date = DateTime::from_timestamp(since.since, 0)
                .map(|v| v.naive_utc());
            let mut actions = Episode::get_actions_by_username(
                username.clone(),
                since_date,
                since.device.clone(),
                since.aggregate.clone(),
                since.podcast.clone(),
            )
            .await;


            if let Some(device) = since.device.clone() {
                actions.iter_mut().for_each(|a| {
                    a.device = device.clone();
                });
            }


            Ok(HttpResponse::Ok().json(EpisodeActionResponse {
                actions,
                timestamp: get_current_timestamp(),
            }))
        }
        None => Err(CustomError::Forbidden),
    }
}

#[post("/episodes/{username}.json")]
pub async fn upload_episode_actions(
    username: web::Path<String>,
    podcast_episode: web::Json<Vec<EpisodeDto>>,
    opt_flag: Option<web::ReqData<Session>>,
) -> Result<HttpResponse, CustomError> {
    match opt_flag {
        Some(flag) => {
            if flag.username != username.clone() {
                return Ok(HttpResponse::Unauthorized().finish());
            }
            let mut inserted_episodes: Vec<Episode> = vec![];
            podcast_episode.iter().for_each(|episode| {
                let episode = Episode::convert_to_episode(episode, username.clone());
                inserted_episodes.push(
                    Episode::insert_episode(
                        &episode.clone(),
                    )
                    .unwrap(),
                );
            });
            Ok(HttpResponse::Ok().json(EpisodeActionPostResponse {
                update_urls: vec![],
                timestamp: get_current_timestamp(),
            }))
        }
        None => Ok(HttpResponse::Unauthorized().finish()),
    }
}
