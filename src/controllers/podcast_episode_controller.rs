use crate::constants::inner_constants::{PodcastType, MAIN_ROOM};
use crate::db::TimelineItem;
use crate::models::episode::Episode;
use crate::models::favorites::Favorite;
use crate::models::messages::BroadcastMessage;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::CustomError;
use actix_web::web::{Data, Json, Query};
use actix_web::{delete, get, post, put};
use actix_web::{web, HttpResponse};
use serde_json::from_str;
use utoipa::ToSchema;

use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::controllers::server::ChatServerHandle;
use crate::models::podcast_dto::PodcastDto;
use crate::models::settings::Setting;
use crate::service::file_service::perform_episode_variable_replacement;
use std::thread;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptionalId {
    last_podcast_episode: Option<String>,
}

impl OptionalId {}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastEpisodeWithHistory {
    pub podcast_episode: PodcastEpisodeDto,
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
    requester: web::ReqData<User>,
    last_podcast_episode: Query<OptionalId>,
) -> Result<HttpResponse, CustomError> {
    let last_podcast_episode = last_podcast_episode.into_inner();
    let id_num = from_str(&id).unwrap();
    let res = PodcastEpisodeService::get_podcast_episodes_of_podcast(
        id_num,
        last_podcast_episode.last_podcast_episode,
        requester.into_inner(),
    )?;
    let mapped_podcasts = res
        .into_iter()
        .map(|podcast| {
            let mapped_podcast_episode: PodcastEpisodeDto = podcast.0.into();
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
    podcast_episode: PodcastEpisodeDto,
    podcast: PodcastDto,
    history: Option<Episode>,
    favorite: Option<Favorite>,
}

#[get("/podcast/available/gpodder")]
pub async fn get_available_podcasts_not_in_webview(
    requester: web::ReqData<User>,
) -> Result<HttpResponse, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomError::Forbidden);
    }
    let found_episodes = Episode::find_episodes_not_in_webview()?;

    Ok(HttpResponse::Ok().json(found_episodes))
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
    requester: web::ReqData<User>,
    favored_only: Query<TimelineQueryParams>,
) -> Result<HttpResponse, CustomError> {
    let res = TimelineItem::get_timeline(
        requester.username.clone(),
        favored_only.into_inner(),
    )?;

    let mapped_timeline = res
        .data
        .iter()
        .map(|podcast_episode| {
            let (podcast_episode, podcast, history, favorite) = podcast_episode.clone();
            let mapped_podcast_episode: PodcastEpisodeDto = podcast_episode.clone();
            let podcast: PodcastDto = podcast.clone();

            TimeLinePodcastEpisode {
                podcast_episode: mapped_podcast_episode,
                podcast,
                history: history.clone(),
                favorite: favorite.clone(),
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
    requester: web::ReqData<User>,
) -> Result<HttpResponse, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomError::Forbidden);
    }

    thread::spawn(move || {
        let res = PodcastEpisode::get_podcast_episode_by_id(&id.into_inner()).unwrap();
        if let Some(podcast_episode) = res {
            let podcast = Podcast::get_podcast(podcast_episode.podcast_id).unwrap();
            PodcastEpisodeService::perform_download(&podcast_episode.clone(), podcast).unwrap();
            PodcastEpisode::update_deleted(&podcast_episode.clone().episode_id, false).unwrap();
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
    requester: web::ReqData<User>,
    lobby: Data<ChatServerHandle>,
) -> Result<HttpResponse, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomError::Forbidden);
    }

    let delted_podcast_episode: PodcastEpisodeDto =
        PodcastEpisodeService::delete_podcast_episode_locally(&id.into_inner())?.into();
    lobby
        .send_broadcast(
            MAIN_ROOM.parse().unwrap(),
            serde_json::to_string(&BroadcastMessage {
                podcast_episode: Some(delted_podcast_episode),
                podcast_episodes: None,
                type_of: PodcastType::DeletePodcastEpisode,
                podcast: None,
                message: "Deleted podcast episode locally".to_string(),
            })
            .unwrap(),
        )
        .await;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize)]
pub struct EpisodeFormatDto {
    pub content: String,
}

#[post("/episodes/formatting")]
pub async fn retrieve_episode_sample_format(
    sample_string: Json<EpisodeFormatDto>,
) -> Result<HttpResponse, CustomError> {
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
        episode_numbering_processed: false,
    };
    let settings = Setting {
        id: 0,
        auto_download: false,
        auto_update: false,
        auto_cleanup: false,
        auto_cleanup_days: 0,
        podcast_prefill: 0,
        replace_invalid_characters: false,
        use_existing_filename: false,
        replacement_strategy: "remove".to_string(),
        episode_format: sample_string.0.content,
        podcast_format: "test".to_string(),
        direct_paths: true,
    };
    let result = perform_episode_variable_replacement(settings, episode, None);

    match result {
        Ok(v) => Ok(HttpResponse::Ok().json(v)),
        Err(e) => Err(CustomError::BadRequest(e.to_string())),
    }
}
