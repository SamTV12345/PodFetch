use actix_web::{web, HttpResponse};
use std::ops::DerefMut;

use actix_web::web::Data;
use actix_web::{get, post};

use crate::models::episode::{Episode, EpisodeAction, EpisodeDto};
use crate::models::podcast_history_item::PodcastHistoryItem;
use crate::models::session::Session;
use crate::utils::error::{map_r2d2_error, CustomError};
use crate::utils::time::get_current_timestamp;
use crate::DbPool;
use chrono::NaiveDateTime;

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
}

#[get("/episodes/{username}.json")]
pub async fn get_episode_actions(
    username: web::Path<String>,
    pool: Data<DbPool>,
    opt_flag: Option<web::ReqData<Session>>,
    since: web::Query<EpisodeSinceRequest>,
) -> Result<HttpResponse, CustomError> {
    match opt_flag {
        Some(flag) => {
            let username = username.clone();
            if flag.username != username.clone() {
                return Err(CustomError::Forbidden);
            }

            let since_date = NaiveDateTime::from_timestamp_opt(since.since, 0);
            let mut actions = Episode::get_actions_by_username(
                username.clone(),
                &mut pool.get().unwrap(),
                since_date,
            )
            .await;
            let watch_logs = PodcastHistoryItem::get_watch_logs_by_username(
                username.clone(),
                &mut pool.get().unwrap(),
                since_date.unwrap(),
            )?
            .iter()
            .map(|watch_log| Episode {
                id: 0,
                username: watch_log.clone().0.username,
                device: "".to_string(),
                podcast: watch_log.clone().2.rssfeed,
                episode: watch_log.clone().1.url,
                timestamp: watch_log.clone().0.date,
                guid: None,
                action: EpisodeAction::Play.to_string(),
                started: Option::from(watch_log.clone().0.watched_time),
                position: Option::from(watch_log.clone().0.watched_time),
                total: Option::from(watch_log.clone().1.total_time),
                cleaned_url: "".to_string(),
            })
            .collect::<Vec<Episode>>();

            actions.append(&mut watch_logs.clone().to_vec());
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
    conn: Data<DbPool>,
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
                        conn.get().map_err(map_r2d2_error).unwrap().deref_mut(),
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
