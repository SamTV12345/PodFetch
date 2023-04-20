use crate::db::DB;
use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use actix_web::web::{Data, Query};
use actix_web::{get, HttpRequest, put};
use actix_web::{web, HttpResponse, Responder};
use serde_json::from_str;
use std::sync::Mutex;
use std::thread;
use crate::controllers::watch_time_controller::get_username;
use crate::DbPool;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::mutex::LockResultExt;

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
    conn: Data<DbPool>
) -> impl Responder {
    let mapping_service = mapping_service.lock() .ignore_poison();

    let last_podcast_episode = last_podcast_episode.into_inner();
    let id_num = from_str(&id).unwrap();
    let res = PodcastEpisodeService::get_podcast_episodes_of_podcast(&mut conn.get().unwrap(), id_num,
                                                                     last_podcast_episode.last_podcast_episode)
        .unwrap();
    let mapped_podcasts = res
        .into_iter()
        .map(|podcast| mapping_service.map_podcastepisode_to_dto(&podcast))
        .collect::<Vec<_>>();
    HttpResponse::Ok().json(mapped_podcasts)
}

#[derive(Serialize, Deserialize)]
pub struct TimeLinePodcastEpisode {
    podcast_episode: PodcastEpisode,
    podcast: Podcast,
}

#[get("/podcasts/timeline")]
pub async fn get_timeline(conn: Data<DbPool>, req: HttpRequest, mapping_service:
Data<Mutex<MappingService>>) ->
                                                                                             impl
Responder {
    let mapping_service = mapping_service.lock().ignore_poison();
    let username = get_username(req);
    if username.is_err(){
        return HttpResponse::BadRequest().json("Username not found");
    }


    let res = DB::get_timeline(username.unwrap(),&mut conn.get().unwrap());

    let mapped_timeline = res.iter().map(|podcast_episode| {
        let (podcast_episode, podcast) = podcast_episode;
        let mapped_podcast_episode = mapping_service.map_podcastepisode_to_dto(podcast_episode);

        TimeLinePodcastEpisode{
            podcast_episode: mapped_podcast_episode,
            podcast: podcast.clone()
        }
    }).collect::<Vec<TimeLinePodcastEpisode>>();
    HttpResponse::Ok().json(mapped_timeline)
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
pub async fn download_podcast_episodes_of_podcast(id: web::Path<String>, conn: Data<DbPool>) ->
                                                                                             impl
Responder {
    thread::spawn(move || {
        let mut db = DB::new().unwrap();
        let res = DB::get_podcast_episode_by_id(&mut conn.get().unwrap(), &id.into_inner())
            .unwrap();
        match res {
            Some(podcast_episode) => {
                let podcast = DB::get_podcast(&mut conn.get().unwrap(),podcast_episode
                    .podcast_id).unwrap();
                PodcastEpisodeService::perform_download(
                    &podcast_episode,
                    &mut db,
                    podcast_episode.clone(),
                    podcast,
                );
            }
            None => {}
        }
    });

    HttpResponse::Ok().json("Download started")
}
