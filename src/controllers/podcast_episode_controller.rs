use crate::constants::inner_constants::{PodcastType};
use crate::db::TimelineItem;
use crate::models::episode::Episode;
use crate::models::favorites::Favorite;
use crate::models::messages::BroadcastMessage;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::models::web_socket_message::Lobby;
use crate::mutex::LockResultExt;
use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::{map_r2d2_error, CustomError};
use crate::DbPool;
use actix::Addr;
use actix_web::web::{Data, Json, Query};
use actix_web::{delete, get, post, put};
use actix_web::{web, HttpResponse};
use serde_json::from_str;
use std::ops::DerefMut;
use std::sync::Mutex;
use std::thread;
use crate::models::settings::Setting;
use crate::service::file_service::{FileService, perform_episode_variable_replacement};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptionalId {
    last_podcast_episode: Option<String>,
}

impl OptionalId {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PodcastEpisodeWithHistory {
    pub podcast_episode: PodcastEpisode,
    pub podcast_history_item: Option<Episode>,
}

#[utoipa::path(
context_path = "/api/v1",
responses(
(status = 200, description = "Finds all podcast episodes of a given podcast id.", body =
[PodcastEpisode])),
tag = "podcast_episodes"
)]
#[get("/podcast/{id}/episodes")]
pub async fn find_all_podcast_episodes_of_podcast(
    id: web::Path<String>,
    requester: Option<web::ReqData<User>>,
    last_podcast_episode: Query<OptionalId>,
    mapping_service: Data<Mutex<MappingService>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    let mapping_service = mapping_service.lock().ignore_poison();

    let last_podcast_episode = last_podcast_episode.into_inner();
    let id_num = from_str(&id).unwrap();
    let res = PodcastEpisodeService::get_podcast_episodes_of_podcast(
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
        id_num,
        last_podcast_episode.last_podcast_episode,
        requester.unwrap().into_inner(),
    )?;
    let mapped_podcasts = res
        .into_iter()
        .map(|podcast| {
            let mapped_podcast_episode = mapping_service.map_podcastepisode_to_dto(&podcast.0);
            PodcastEpisodeWithHistory {
                podcast_episode: mapped_podcast_episode,
                podcast_history_item: podcast.1,
            }
        })
        .collect::<Vec<PodcastEpisodeWithHistory>>();
    Ok(HttpResponse::Ok().json(mapped_podcasts))
}

#[derive(Serialize, Deserialize)]
pub struct TimeLinePodcastEpisode {
    podcast_episode: PodcastEpisode,
    podcast: Podcast,
    history: Option<Episode>,
    favorite: Option<Favorite>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeLinePodcastItem {
    data: Vec<TimeLinePodcastEpisode>,
    total_elements: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineQueryParams {
    pub favored_only: bool,
    pub last_timestamp: Option<String>,
    pub not_listened: bool,
}

#[utoipa::path(
context_path = "/api/v1",
responses(
(status = 200, description = "Gets the current timeline of the user")),
tag = "podcasts"
)]
#[get("/podcasts/timeline")]
pub async fn get_timeline(
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>,
    mapping_service: Data<Mutex<MappingService>>,
    favored_only: Query<TimelineQueryParams>,
) -> Result<HttpResponse, CustomError> {
    let mapping_service = mapping_service.lock().ignore_poison().clone();

    let res = TimelineItem::get_timeline(
        requester.unwrap().username.clone(),
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
        favored_only.into_inner(),
    )?;

    let mapped_timeline = res
        .data
        .iter()
        .map(|podcast_episode| {
            let (podcast_episode, podcast, history, favorite) = podcast_episode.clone();
            let mapped_podcast_episode =
                mapping_service.map_podcastepisode_to_dto(&podcast_episode);

            TimeLinePodcastEpisode {
                podcast_episode: mapped_podcast_episode,
                podcast,
                history,
                favorite,
            }
        })
        .collect::<Vec<TimeLinePodcastEpisode>>();
    Ok(HttpResponse::Ok().json(TimeLinePodcastItem {
        data: mapped_timeline,
        total_elements: res.total_elements,
    }))
}

/**
 * id is the episode id (uuid)
 */
#[utoipa::path(
context_path = "/api/v1",
responses(
(status = 200, description = "Starts the download of a given podcast episode")),
tag = "podcast_episodes"
)]
#[put("/podcast/{id}/episodes/download")]
pub async fn download_podcast_episodes_of_podcast(
    id: web::Path<String>,
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_privileged_user() {
        return Err(CustomError::Forbidden);
    }

    thread::spawn(move || {
        let res = PodcastEpisode::get_podcast_episode_by_id(
            conn.get().map_err(map_r2d2_error).unwrap().deref_mut(),
            &id.into_inner(),
        )
        .unwrap();
        if let Some(podcast_episode) = res {
            let podcast = Podcast::get_podcast(
                conn.get().map_err(map_r2d2_error).unwrap().deref_mut(),
                podcast_episode.podcast_id,
            )
            .unwrap();
            PodcastEpisodeService::perform_download(
                &podcast_episode.clone(),
                podcast,
                conn.get().map_err(map_r2d2_error).unwrap().deref_mut(),
            )
            .unwrap();
            PodcastEpisode::update_deleted(
                conn.get().map_err(map_r2d2_error).unwrap().deref_mut(),
                &podcast_episode.clone().episode_id,
                false,
            )
            .unwrap();
        }
    });

    Ok(HttpResponse::Ok().json("Download started"))
}

/**
 * id is the episode id (uuid)
 */
#[utoipa::path(
context_path = "/api/v1",
responses(
(status = 204, description = "Removes the download of a given podcast episode. This very episode \
won't be included in further checks/downloads unless done by user.")),
tag = "podcast_episodes"
)]
#[delete("/episodes/{id}/download")]
pub async fn delete_podcast_episode_locally(
    id: web::Path<String>,
    requester: Option<web::ReqData<User>>,
    db: Data<DbPool>,
    lobby: Data<Addr<Lobby>>,
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_privileged_user() {
        return Err(CustomError::Forbidden);
    }

    let delted_podcast_episode = PodcastEpisodeService::delete_podcast_episode_locally(
        &id.into_inner(),
        &mut db.get().unwrap(),
    )?;
    lobby.do_send(BroadcastMessage {
        podcast_episode: Some(delted_podcast_episode),
        podcast_episodes: None,
        type_of: PodcastType::DeletePodcastEpisode,
        podcast: None,
        message: "Deleted podcast episode locally".to_string(),
    });

    Ok(HttpResponse::NoContent().finish())
}


#[post("/episodes/formatting")]
pub async fn retrieve_episode_sample_format(sample_string: Json<String>) -> Result<HttpResponse, CustomError> {
    // Sample episode for formatting
     let episode: PodcastEpisode = PodcastEpisode {
         id: 0,
        podcast_id: 0,
        episode_id: "0218342".to_string(),
        name: "My Homelab".to_string(),
        url: "http://podigee.com/rss/123".to_string(),
        date_of_recording: "2023-12-24".to_string(),
        image_url: "http://podigee.com/rss/123/image".to_string(),
        total_time: 1200,
        local_url: "http://localhost:8912/podcasts/123".to_string(),
        local_image_url: "http://localhost:8912/podcasts/123/image".to_string(),
        description: "My description".to_string(),
        status: "D".to_string(),
        download_time: None,
        guid: "081923123".to_string(),
        deleted: false,
        file_episode_path: None,
        file_image_path: None,
    };
    let settings = Setting{
        id: 0,
        auto_download: false,
        auto_update: false,
        auto_cleanup: false,
        auto_cleanup_days: 0,
        podcast_prefill: 0,
        replace_invalid_characters: false,
        use_existing_filename: false,
        replacement_strategy: "".to_string(),
        episode_format: sample_string.0,
        podcast_format: "test".to_string(),
        direct_paths: true,
    };
    let result = perform_episode_variable_replacement(settings, episode);

    match result {
        Ok(v)=>Ok(HttpResponse::Ok().json(v)),
        Err(e)=>Err(CustomError::BadRequest(e.to_string()))
    }
}