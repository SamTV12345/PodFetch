use crate::db::TimelineItem;
use crate::models::episode::{Episode, EpisodeDto};
use crate::models::favorites::Favorite;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::{CustomError, CustomErrorInner};
use serde_json::from_str;
use utoipa::{IntoParams, ToSchema};

use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::controllers::server::ChatServerHandle;
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::gpodder_available_podcasts::GPodderAvailablePodcasts;
use crate::models::podcast_dto::PodcastDto;
use crate::models::settings::Setting;
use crate::service::file_service::perform_episode_variable_replacement;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::{Extension, Json};
use std::thread;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Serialize, Deserialize, Clone, IntoParams)]
pub struct OptionalId {
    last_podcast_episode: Option<String>,
    only_unlistened: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastEpisodeWithHistory {
    pub podcast_episode: PodcastEpisodeDto,
    pub podcast_history_item: Option<EpisodeDto>,
}

#[utoipa::path(
get,
path="/podcasts/{id}/episodes",
params(OptionalId),
responses(
(status = 200, description = "Finds all podcast episodes of a given podcast id.", body =
[PodcastEpisodeWithHistory])),
tag = "podcast_episodes"
)]
pub async fn find_all_podcast_episodes_of_podcast(
    Path(id): Path<String>,
    Extension(user): Extension<User>,
    last_podcast_episode: Query<OptionalId>,
) -> Result<Json<Vec<PodcastEpisodeWithHistory>>, CustomError> {
    let id_num = from_str(&id).unwrap();

    let res = PodcastEpisodeService::get_podcast_episodes_of_podcast(
        id_num,
        last_podcast_episode.last_podcast_episode.clone(),
        last_podcast_episode.only_unlistened,
        &user,
    )?;
    let mapped_podcasts = res
        .into_iter()
        .map(|podcast_inner| {
            let mapped_podcast_episode: PodcastEpisodeDto =
                (podcast_inner.0, Some(user.clone()), podcast_inner.2).into();
            PodcastEpisodeWithHistory {
                podcast_episode: mapped_podcast_episode,
                podcast_history_item: podcast_inner.1.map(|e| e.convert_to_episode_dto()),
            }
        })
        .collect::<Vec<PodcastEpisodeWithHistory>>();
    Ok(Json(mapped_podcasts))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TimeLinePodcastEpisode {
    podcast_episode: PodcastEpisodeDto,
    podcast: PodcastDto,
    history: Option<Episode>,
    favorite: Option<Favorite>,
}

#[utoipa::path(
    get,
    path="/podcasts/available/gpodder",
    responses(
(status = 200, description = "Finds all podcast not in webview", body =
[GPodderAvailablePodcasts])),
    tag = "gpodder"
)]
pub async fn get_available_podcasts_not_in_webview(
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<GPodderAvailablePodcasts>>, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }
    let found_episodes = Episode::find_episodes_not_in_webview()?;

    Ok(Json(found_episodes))
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimeLinePodcastItem {
    data: Vec<TimeLinePodcastEpisode>,
    total_elements: i64,
}

#[derive(Serialize, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TimelineQueryParams {
    pub favored_only: bool,
    pub last_timestamp: Option<String>,
    pub not_listened: bool,
    pub favored_episodes: bool,
}

#[utoipa::path(
get,
path="/podcasts/timeline",
params(TimelineQueryParams),
responses(
(status = 200, description = "Gets the current timeline of the user", body = TimeLinePodcastItem)),
tag = "podcasts"
)]
pub async fn get_timeline(
    Extension(requester): Extension<User>,
    Query(favored_only): Query<TimelineQueryParams>,
) -> Result<Json<TimeLinePodcastItem>, CustomError> {
    let res = TimelineItem::get_timeline(requester, favored_only)?;

    let mapped_timeline = res
        .data
        .iter()
        .map(|podcast_episode| {
            let (podcast_episode, podcast_extracted, history, favorite) = podcast_episode.clone();

            TimeLinePodcastEpisode {
                podcast_episode,
                podcast: podcast_extracted,
                history,
                favorite,
            }
        })
        .collect::<Vec<TimeLinePodcastEpisode>>();
    Ok(Json(TimeLinePodcastItem {
        data: mapped_timeline,
        total_elements: res.total_elements,
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct FavoritePut {
    pub favored: bool,
}

/**
 * id is the episode id (uuid)
 */
#[utoipa::path(
put,
path="/podcasts/{id}/episodes/favor",
    responses(
(status = 200, description = "Likes a given podcast episode.", body=FavoritePut)),
    tag = "podcast_episodes"
)]
pub async fn like_podcast_episode(
    Path(id): Path<i32>,
    Extension(requester): Extension<User>,
    Json(fav): Json<FavoritePut>,
) -> Result<StatusCode, CustomError> {
    println!("User id is {}, Episode id is {}", requester.id, id.clone());
    FavoritePodcastEpisode::like_podcast_episode(id, &requester, fav.favored)?;

    Ok(StatusCode::OK)
}

/**
 * id is the episode id (uuid)
 */
#[utoipa::path(
put,
path="/podcasts/{id}/episodes/download",
responses(
(status = 200, description = "Starts the download of a given podcast episode")),
tag = "podcast_episodes"
)]
pub async fn download_podcast_episodes_of_podcast(
    Extension(requester): Extension<User>,
    Path(id): Path<String>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    thread::spawn(move || {
        let res = PodcastEpisode::get_podcast_episode_by_id(&id).unwrap();
        if let Some(podcast_episode) = res {
            let podcast_found = Podcast::get_podcast(podcast_episode.podcast_id).unwrap();
            PodcastEpisodeService::perform_download(&podcast_episode.clone(), &podcast_found)
                .unwrap();
            PodcastEpisode::update_deleted(&podcast_episode.clone().episode_id, false).unwrap();
            ChatServerHandle::broadcast_podcast_episode_offline_available(
                &podcast_episode,
                &podcast_found,
            );
        }
    });

    Ok(StatusCode::from_u16(200).unwrap())
}

/**
 * id is the episode id (uuid)
 */
#[utoipa::path(
delete,
path="/episodes/{id}/download",
responses(
(status = 204, description = "Removes the download of a given podcast episode. This very episode \
won't be included in further checks/downloads unless done by user.")),
tag = "podcast_episodes"
)]
pub async fn delete_podcast_episode_locally(
    id: Path<String>,
    requester: Extension<User>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let delted_podcast_episode = tokio::task::spawn_blocking(move || {
        PodcastEpisodeService::delete_podcast_episode_locally(&id)
    })
    .await
    .unwrap()?;

    ChatServerHandle::broadcast_podcast_episode_deleted_locally(&delted_podcast_episode);

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct EpisodeFormatDto {
    pub content: String,
}

#[utoipa::path(
    post,
    path="/episodes/formatting",
    responses(
(status = 204, description = "Retrieve episode sample format")),
    tag = "podcast_episodes"
)]
pub async fn retrieve_episode_sample_format(
    sample_string: Json<EpisodeFormatDto>,
) -> Result<String, CustomError> {
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
        description: "My description".to_string(),
        download_time: None,
        guid: "081923123".to_string(),
        deleted: false,
        file_episode_path: None,
        file_image_path: None,
        episode_numbering_processed: false,
        download_location: None,
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
    let result = perform_episode_variable_replacement(settings, episode, None)?;

    Ok(result)
}

pub fn get_podcast_episode_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(find_all_podcast_episodes_of_podcast))
        .routes(routes!(get_available_podcasts_not_in_webview))
        .routes(routes!(get_timeline))
        .routes(routes!(like_podcast_episode))
        .routes(routes!(download_podcast_episodes_of_podcast))
        .routes(routes!(delete_podcast_episode_locally))
        .routes(routes!(retrieve_episode_sample_format))
}
