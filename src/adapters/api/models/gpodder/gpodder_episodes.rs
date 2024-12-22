use actix_web::{web, HttpResponse};

use actix_web::{get, post};

use crate::utils::error::CustomError;
use crate::utils::time::get_current_timestamp;
use chrono::{DateTime};
use crate::adapters::api::models::episode::episode::EpisodeDto;
use crate::adapters::api::models::user::session::SessionDto;
use crate::application::services::episode::episode_service::EpisodeService;
use crate::domain::models::episode::episode::Episode;

#[derive(Serialize, Deserialize)]
pub struct EpisodeActionResponse {
    actions: Vec<EpisodeDto>,
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
    opt_flag: Option<web::ReqData<SessionDto>>,
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
            let mut actions = EpisodeService::get_actions_by_username(
                &username,
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
    mut podcast_episode: web::Json<Vec<EpisodeDto>>,
    opt_flag: Option<web::ReqData<SessionDto>>,
) -> Result<HttpResponse, CustomError> {
    match opt_flag {
        Some(flag) => {
            if flag.username != username.clone() {
                return Ok(HttpResponse::Unauthorized().finish());
            }
            let mut inserted_episodes: Vec<Episode> = vec![];
            podcast_episode.iter_mut().for_each(|mut episode| {
                episode.username = username.clone();
                let episode = episode.into();
                inserted_episodes.push(
                    EpisodeService::insert_episode(
                        &episode.clone(),
                    )?
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
