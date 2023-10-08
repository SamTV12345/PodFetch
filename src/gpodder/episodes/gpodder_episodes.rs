use std::ops::DerefMut;
use actix_web::{HttpResponse, web};

use actix_web::{get,post};
use actix_web::web::Data;

use crate::DbPool;
use crate::models::episode::{Episode, EpisodeAction, EpisodeDto};
use chrono::NaiveDateTime;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcast_history_item::PodcastHistoryItem;
use crate::models::session::Session;
use crate::utils::error::{CustomError, map_r2d2_error};
use crate::utils::time::{get_current_timestamp};

#[derive(Serialize, Deserialize)]
pub struct EpisodeActionResponse{
    actions: Vec<Episode>,
    timestamp: i64
}


#[derive(Serialize, Deserialize)]
pub struct EpisodeActionPostResponse{
    update_urls: Vec<String>,
    timestamp: i64
}

#[derive(Serialize, Deserialize)]
pub struct EpisodeSinceRequest{
    since: i64
}

#[get("/episodes/{username}.json")]
pub async fn get_episode_actions(username: web::Path<String>, pool: Data<DbPool>,
                                 opt_flag: Option<web::ReqData<Session>>,
                                 since: web::Query<EpisodeSinceRequest>) -> Result<HttpResponse,
    CustomError> {
    match opt_flag {
        Some(flag) => {
            let username = username.clone();
            if flag.username != username.clone() {
                return Err(CustomError::Forbidden)
            }

            let since_date = NaiveDateTime::from_timestamp_opt(since.since, 0);
            let mut actions = Episode::get_actions_by_username(username.clone(), &mut pool.get().unwrap(), since_date)
                .await;
            let watch_logs = PodcastHistoryItem::get_watch_logs_by_username(username.clone(), &mut pool.get()
                .unwrap(), since_date.unwrap())?.iter().map(|watch_log| {
                Episode{
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
                }
            }).collect::<Vec<Episode>>();

            actions.append(&mut watch_logs.clone().to_vec());
            Ok(HttpResponse::Ok().json(EpisodeActionResponse {
                actions,
                timestamp: get_current_timestamp()
            }))
        }
        None => {
           Err(CustomError::Forbidden)
        }
    }
}


#[post("/episodes/{username}.json")]
pub async fn upload_episode_actions(username: web::Path<String>, podcast_episode: web::Json<Vec<EpisodeDto>>,opt_flag: Option<web::ReqData<Session>>, conn: Data<DbPool>)
    -> Result<HttpResponse,CustomError>{
    match opt_flag {
        Some(flag) => {
            if flag.username != username.clone() {
                 return Ok(HttpResponse::Unauthorized().finish());
            }
            let mut inserted_episodes: Vec<Episode> = vec![];
            podcast_episode.iter().for_each(|episode| {
                let episode = Episode::convert_to_episode(episode, username.clone());
                inserted_episodes.push(Episode::insert_episode(&episode.clone(), &mut conn
                    .get()
                    .unwrap())
                    .expect("Unable to insert episode"));

                if EpisodeAction::from_string(&episode.clone().action) == EpisodeAction::Play {
                    let mut episode_url = episode.clone().episode;
                    // Sometimes podcast provider like to check which browser access their podcast
                    let mut first_split = episode.episode.split('?');
                    let res = first_split.next();

                    if let Some(unwrapped_res) = res {
                        episode_url = unwrapped_res.parse().unwrap()
                    };

                    let podcast_episode = PodcastEpisode::query_podcast_episode_by_url(conn.get()
                                                                                           .map_err(map_r2d2_error).unwrap().deref_mut(),
                                                                           &episode_url);
                    if podcast_episode.clone().unwrap().is_none() {
                    }
                }
            });
            Ok(HttpResponse::Ok().json(EpisodeActionPostResponse {
                update_urls: vec![],
                timestamp: get_current_timestamp()
            }))
        }
        None => {
            Ok(HttpResponse::Unauthorized().finish())
        }
    }
}