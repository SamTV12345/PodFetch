use std::thread;
use actix_web::{HttpResponse, Responder, web};
use actix_web::web::Query;
use serde_json::from_str;
use crate::db::DB;
use crate::service::mapping_service::MappingService;
use actix_web::{get, put};
use crate::service::podcast_episode_service::PodcastEpisodeService;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptionalId {
    last_podcast_episode: Option<String>,
}

impl OptionalId {
    pub fn new() -> Self {
        OptionalId { last_podcast_episode: None }
    }
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Finds all podcast episodes of a given podcast id.", body =
[PodcastEpisode])),
tag="podcast_episodes"
)]
#[get("/podcast/{id}/episodes")]
pub async fn find_all_podcast_episodes_of_podcast(id: web::Path<String>, last_podcast_episode :
Query<OptionalId>)
                                                  -> impl Responder {
    let last_podcast_episode = last_podcast_episode.into_inner();
    let id_num = from_str(&id).unwrap();
    let mut db = DB::new().unwrap();
    let mappingservice = MappingService::new();
    let res  = db.get_podcast_episodes_of_podcast(id_num,last_podcast_episode
        .last_podcast_episode ).unwrap();
    let mapped_podcasts = res
        .into_iter()
        .map(|podcast| mappingservice.map_podcastepisode_to_dto(&podcast)).collect::<Vec<_>>();
    HttpResponse::Ok().json(mapped_podcasts)
}

/**
    * id is the episode id (uuid)
 */
#[put("/podcast/{id}/episodes/download")]
pub async fn download_podcast_episodes_of_podcast(id: web::Path<String>) -> impl Responder {

    thread::spawn(move || {
        let mut db = DB::new().unwrap();
        let res  = db.get_podcast_episode_by_id(&id.into_inner()).unwrap();
        match res {
            Some(podcast_episode) => {
                let podcast = db.get_podcast(podcast_episode.podcast_id).unwrap();
                PodcastEpisodeService::perform_download(&podcast_episode, &mut db, podcast_episode
                    .clone(), podcast);
            }
            None => {
            }
        }
    });

    HttpResponse::Ok().json("Download started")
}