use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use actix_web::web::{Data, Query};
use actix_web::{get, put};
use actix_web::{web, HttpResponse};
use serde_json::from_str;
use std::sync::Mutex;
use std::thread;
use crate::db::TimelineItem;
use crate::DbPool;
use crate::models::favorites::Favorite;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::mutex::LockResultExt;
use crate::utils::error::CustomError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptionalId {
    last_podcast_episode: Option<String>,
}

impl OptionalId {}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Finds all podcast episodes of a given podcast id.", body =
[PodcastEpisode])),
tag="podcast_episodes"
)]
#[get("/podcast/{id}/episodes")]
pub async fn find_all_podcast_episodes_of_podcast(
    id: web::Path<String>,
    last_podcast_episode: Query<OptionalId>,
    mapping_service: Data<Mutex<MappingService>>,
    conn: Data<DbPool>) -> Result<HttpResponse, CustomError> {
    let mapping_service = mapping_service.lock() .ignore_poison();

    let last_podcast_episode = last_podcast_episode.into_inner();
    let id_num = from_str(&id).unwrap();
    let res = PodcastEpisodeService::get_podcast_episodes_of_podcast(&mut conn.get().unwrap(), id_num,
                                                                     last_podcast_episode.last_podcast_episode)?;
    let mapped_podcasts = res
        .into_iter()
        .map(|podcast| mapping_service
            .map_podcastepisode_to_dto(&podcast))
        .collect::<Vec<PodcastEpisode>>();
    Ok(HttpResponse::Ok()
        .json(mapped_podcasts))
}

#[derive(Serialize, Deserialize)]
pub struct TimeLinePodcastEpisode {
    podcast_episode: PodcastEpisode,
    podcast: Podcast,
    favorite: Option<Favorite>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeLinePodcastItem{
    data: Vec<TimeLinePodcastEpisode>,
    total_elements: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineQueryParams {
    pub favored_only: bool,
    pub last_timestamp: Option<String>
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the current timeline of the user")),
tag="podcasts"
)]
#[get("/podcasts/timeline")]
pub async fn get_timeline(conn: Data<DbPool>,  requester: Option<web::ReqData<User>>, mapping_service:
Data<Mutex<MappingService>>, favored_only: Query<TimelineQueryParams>) -> Result<HttpResponse,
    CustomError> {
    let mapping_service = mapping_service.lock().ignore_poison().clone();


    let res = TimelineItem::get_timeline(requester.unwrap().username.clone(), &mut conn.get().unwrap(),
                                   favored_only.into_inner());

    let mapped_timeline = res.data.iter().map(|podcast_episode| {
        let (podcast_episode, podcast, favorite) = podcast_episode.clone();
        let mapped_podcast_episode = mapping_service.map_podcastepisode_to_dto(&podcast_episode);

        return TimeLinePodcastEpisode {
            podcast_episode: mapped_podcast_episode,
            podcast,
            favorite
        };
        }).collect::<Vec<TimeLinePodcastEpisode>>();
    Ok(HttpResponse::Ok().json(TimeLinePodcastItem{
        data: mapped_timeline,
        total_elements: res.total_elements
    }))
}

/**
 * id is the episode id (uuid)
 */
#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Starts the download of a given podcast episode")),
tag="podcast_episodes"
)]
#[put("/podcast/{id}/episodes/download")]
pub async fn download_podcast_episodes_of_podcast(id: web::Path<String>, conn: Data<DbPool>,
                                                  requester: Option<web::ReqData<User>>) -> Result<HttpResponse, CustomError>{
    if !requester.unwrap().is_privileged_user(){
        return Err(CustomError::Forbidden)
    }

    thread::spawn(move || {
        let res = PodcastEpisode::get_podcast_episode_by_id(&mut conn.get().unwrap(), &id
            .into_inner()).unwrap();
        match res {
            Some(podcast_episode) => {
                let podcast = Podcast::get_podcast(&mut conn.get().unwrap(),podcast_episode
                    .podcast_id).unwrap();
                PodcastEpisodeService::perform_download(
                    &podcast_episode,
                    podcast_episode.clone(),
                    podcast,
                    &mut conn.get().unwrap()
                ).unwrap();
            }
            None => {
            }
        }
    });

    Ok(HttpResponse::Ok().json("Download started"))
}
