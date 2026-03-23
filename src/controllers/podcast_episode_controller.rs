use crate::application::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use crate::application::usecases::watchtime::WatchtimeUseCase as WatchtimeService;
use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::app_state::AppState;
use crate::controllers::server::ChatServerHandle;
use crate::db::TimelineItem;
use crate::adapters::api::mappers::episode::map_episode_to_dto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::application::services::file::service::perform_episode_variable_replacement;
use crate::application::services::podcast::service::PodcastService;
use crate::utils::error::ErrorSeverity::Warning;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::url_builder::create_url_rewriter;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::{Extension, Json};
use podfetch_domain::user::User;
use podfetch_web::podcast::PodcastDto;
use podfetch_web::history::EpisodeDto;
pub use podfetch_web::podcast_episode::{
    EpisodeFormatDto, FavoritePut, OptionalId, PodcastChapterDto, TimelineQueryParams,
};
use podfetch_web::podcast_episode::{
    PodcastEpisodeControllerError, PodcastEpisodeWithHistory as WebPodcastEpisodeWithHistory,
    TimelineFavorite,
    TimeLinePodcastEpisode as WebTimeLinePodcastEpisode,
    TimeLinePodcastItem as WebTimeLinePodcastItem,
};
use podfetch_web::podcast_episode::{
    get_episode_with_history as web_get_episode_with_history,
    get_podcast_episodes_with_history as web_get_podcast_episodes_with_history,
    require_privileged as web_require_privileged,
};
use podfetch_web::settings::Setting;
use podfetch_web::subscription::GPodderAvailablePodcast;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub type PodcastEpisodeWithHistory = WebPodcastEpisodeWithHistory<PodcastEpisodeDto, EpisodeDto>;

fn map_podcast_episode_controller_error(
    error: PodcastEpisodeControllerError<CustomError>,
) -> CustomError {
    match error {
        PodcastEpisodeControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        PodcastEpisodeControllerError::NotFound => CustomErrorInner::NotFound(Warning).into(),
        PodcastEpisodeControllerError::BadRequest(message) => {
            CustomErrorInner::BadRequest(message, Warning).into()
        }
        PodcastEpisodeControllerError::Service(error) => error,
    }
}

#[utoipa::path(
    get,
    path="/podcasts/episodes/{id}/chapters",
    responses(
(status = 200, description = "Finds all chapters of the podcast episode.", body =
[PodcastChapterDto])),
    tag = "podcast_episodes"
)]
pub async fn find_all_chapters_of_podcast_episode(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<PodcastChapterDto>>, CustomError> {
    // no auth needed for this endpoint

    let chapters_of_podcast: Vec<PodcastChapterDto> = state
        .podcast_episode_chapter_service
        .get_chapters_by_episode_id(id)?
        .into_iter()
        .map(|v| PodcastChapterDto {
            id: v.id,
            title: v.title,
            start_time: v.start_time,
            end_time: v.end_time,
        })
        .collect();

    Ok(Json(chapters_of_podcast))
}

#[utoipa::path(
    get,
    path="/episodes/{id}",
    params(OptionalId),
    responses(
(status = 200, description = "Finds all podcast episodes of a given podcast id.", body =
[PodcastEpisodeWithHistory])),
    tag = "podcast_episodes"
)]
pub async fn get_podcast_episode_by_id(
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
    headers: HeaderMap,
) -> Result<Json<PodcastEpisodeWithHistory>, CustomError> {
    let rewriter = create_url_rewriter(&headers);
    let requester_username = requester.username.clone();
    let episode_with_history = web_get_episode_with_history(
        &id,
        &requester_username,
        |episode_id| {
            PodcastEpisodeService::get_podcast_episode_by_id(episode_id).map(|opt| {
                opt.map(|podcast_inner| {
                    let mut mapped_podcast_episode: PodcastEpisodeDto =
                        (podcast_inner, Some(requester), None).into();
                    rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_url);
                    rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_image_url);
                    mapped_podcast_episode
                })
            })
        },
        |episode_id, username| {
            WatchtimeService::get_watchtime(episode_id, username)
                .map(|episode| {
                    episode
                        .map(Into::into)
                        .as_ref()
                        .map(map_episode_to_dto)
                })
        },
    )
    .map_err(map_podcast_episode_controller_error)?;

    Ok(Json(episode_with_history))
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
    headers: HeaderMap,
) -> Result<Json<Vec<PodcastEpisodeWithHistory>>, CustomError> {
    let rewriter = create_url_rewriter(&headers);
    let mapped_podcasts = web_get_podcast_episodes_with_history(
        &id,
        &user.username,
        last_podcast_episode.last_podcast_episode.clone(),
        last_podcast_episode.only_unlistened,
        |podcast_id, last_episode, only_unlistened, _username| {
            PodcastEpisodeService::get_podcast_episodes_of_podcast(
                podcast_id,
                last_episode,
                only_unlistened,
                &user,
            )
            .map(|episodes| {
                episodes
                    .into_iter()
                    .map(|podcast_inner| {
                        let mut mapped_podcast_episode: PodcastEpisodeDto =
                            (podcast_inner.0, Some(user.clone()), podcast_inner.2).into();
                        rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_url);
                        rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_image_url);
                        (
                            mapped_podcast_episode,
                            podcast_inner
                                .1
                                .map(Into::into)
                                .as_ref()
                                .map(map_episode_to_dto),
                        )
                    })
                    .collect()
            })
        },
    )
    .map_err(map_podcast_episode_controller_error)?;
    Ok(Json(mapped_podcasts))
}

pub type TimeLinePodcastEpisode =
    WebTimeLinePodcastEpisode<PodcastEpisodeDto, PodcastDto, EpisodeDto, TimelineFavorite>;

#[utoipa::path(
    get,
    path="/podcasts/available/gpodder",
    responses(
(status = 200, description = "Finds all podcast not in webview", body =
[GPodderAvailablePodcast])),
    tag = "gpodder"
)]
pub async fn get_available_podcasts_not_in_webview(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<GPodderAvailablePodcast>>, CustomError> {
    web_require_privileged::<CustomError>(requester.is_privileged_user())
        .map_err(map_podcast_episode_controller_error)?;
    let found_episodes = state
        .subscription_service
        .get_available_gpodder_podcasts()?;

    Ok(Json(found_episodes))
}

pub type TimeLinePodcastItem = WebTimeLinePodcastItem<TimeLinePodcastEpisode>;

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
    headers: HeaderMap,
) -> Result<Json<TimeLinePodcastItem>, CustomError> {
    let res = TimelineItem::get_timeline(requester, favored_only)?;
    let rewriter = create_url_rewriter(&headers);

    let mapped_timeline = res
        .data
        .iter()
        .map(|podcast_episode| {
            let (podcast_episode, podcast_extracted, history, favorite) = podcast_episode.clone();

            let mut mapped_podcast_episode = podcast_episode;
            rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_url);
            rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_image_url);
            let mapped_podcast = podcast_extracted.with_rewritten_urls(&rewriter);

            TimeLinePodcastEpisode {
                podcast_episode: mapped_podcast_episode,
                podcast: mapped_podcast,
                history,
                favorite: favorite.map(Into::into),
            }
        })
        .collect::<Vec<TimeLinePodcastEpisode>>();
    Ok(Json(TimeLinePodcastItem {
        data: mapped_timeline,
        total_elements: res.total_elements,
    }))
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
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(requester): Extension<User>,
    Json(fav): Json<FavoritePut>,
) -> Result<StatusCode, CustomError> {
    println!("User id is {}, Episode id is {}", requester.id, id.clone());
    state
        .favorite_podcast_episode_service
        .set_favorite(&requester.username, id, fav.favored)?;

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
    web_require_privileged::<CustomError>(requester.is_privileged_user())
        .map_err(map_podcast_episode_controller_error)?;

    tokio::task::spawn_blocking(
        move || match PodcastEpisodeService::get_podcast_episode_by_id(&id) {
            Ok(Some(podcast_episode)) => match PodcastService::get_podcast(podcast_episode.podcast_id) {
                Ok(podcast_found) => {
                    if let Err(err) =
                        PodcastEpisodeService::perform_download(&podcast_episode, &podcast_found)
                    {
                        log::error!(
                            "Error downloading episode {}: {}",
                            podcast_episode.episode_id,
                            err
                        );
                        return;
                    }
                    if let Err(err) =
                        PodcastEpisodeService::update_deleted(&podcast_episode.episode_id, false)
                    {
                        log::error!(
                            "Error updating deleted status for episode {}: {}",
                            podcast_episode.episode_id,
                            err
                        );
                    }
                    ChatServerHandle::broadcast_podcast_episode_offline_available(
                        &podcast_episode,
                        &podcast_found,
                    );
                }
                Err(err) => {
                    log::error!(
                        "Could not load podcast {} for episode {}: {}",
                        podcast_episode.podcast_id,
                        podcast_episode.episode_id,
                        err
                    );
                }
            },
            Ok(None) => {
                log::error!("Episode with id {} not found", id);
            }
            Err(err) => {
                log::error!("Error retrieving episode {}: {}", id, err);
            }
        },
    );

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
    web_require_privileged::<CustomError>(requester.is_privileged_user())
        .map_err(map_podcast_episode_controller_error)?;

    let delted_podcast_episode = tokio::task::spawn_blocking(move || {
        PodcastEpisodeService::delete_podcast_episode_locally(&id)
    })
    .await
    .unwrap()?;

    ChatServerHandle::broadcast_podcast_episode_deleted_locally(&delted_podcast_episode);

    Ok(StatusCode::NO_CONTENT)
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
    let result = perform_episode_variable_replacement(settings.into(), episode, None)?;

    Ok(result)
}

pub fn get_podcast_episode_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(find_all_podcast_episodes_of_podcast))
        .routes(routes!(get_available_podcasts_not_in_webview))
        .routes(routes!(get_timeline))
        .routes(routes!(like_podcast_episode))
        .routes(routes!(get_podcast_episode_by_id))
        .routes(routes!(download_podcast_episodes_of_podcast))
        .routes(routes!(delete_podcast_episode_locally))
        .routes(routes!(retrieve_episode_sample_format))
        .routes(routes!(find_all_chapters_of_podcast_episode))
}

#[cfg(test)]
mod tests {
    use crate::adapters::persistence::dbconfig::db::get_connection;
    use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl as pe_dsl;
    use crate::adapters::persistence::dbconfig::schema::subscriptions::dsl as subs_dsl;
    use crate::app_state::AppState;
    use crate::commands::startup::tests::handle_test_startup;
    use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
    use crate::models::podcast_episode::PodcastEpisode;
    use crate::application::services::favorite_podcast_episode::service::FavoritePodcastEpisodeService;
    use crate::utils::error::CustomErrorInner;
    use crate::utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use axum::Extension;
    use axum::extract::{Path, State};
    use chrono::Utc;
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;
    use podfetch_domain::user::User;
    use serde_json::json;
    use serial_test::serial;
    use uuid::Uuid;

    fn admin_username() -> String {
        ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "postgres".to_string())
    }

    fn unique_name(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }

    fn non_admin_user() -> User {
        UserTestDataBuilder::new().build()
    }

    fn app_state() -> AppState {
        AppState::new()
    }

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    fn insert_episode(
        podcast_id: i32,
        episode_id: &str,
        guid: &str,
        title: &str,
    ) -> PodcastEpisode {
        diesel::insert_into(pe_dsl::podcast_episodes)
            .values((
                pe_dsl::podcast_id.eq(podcast_id),
                pe_dsl::episode_id.eq(episode_id.to_string()),
                pe_dsl::name.eq(title.to_string()),
                pe_dsl::url.eq(format!("https://example.com/{episode_id}.mp3")),
                pe_dsl::date_of_recording.eq("2026-03-01T00:00:00Z".to_string()),
                pe_dsl::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe_dsl::total_time.eq(1800),
                pe_dsl::description.eq("podcast episode test".to_string()),
                pe_dsl::guid.eq(guid.to_string()),
                pe_dsl::deleted.eq(false),
                pe_dsl::episode_numbering_processed.eq(false),
            ))
            .get_result::<PodcastEpisode>(&mut get_connection())
            .unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_retrieve_episode_sample_format() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .post("/api/v1/episodes/formatting")
            .json(&json!({
                "content": "{episodeTitle}-{episodeDate}"
            }))
            .await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.text(), "My Homelab-2023-12-24");
    }

    #[tokio::test]
    #[serial]
    async fn test_find_all_podcast_episodes_and_get_single_by_id() {
        let server = handle_test_startup().await;

        let podcast = crate::application::services::podcast::service::PodcastService::add_podcast_to_database(
            "Episode Query Podcast",
            "episode-query",
            "https://example.com/episode-query.xml",
            "http://localhost:8080/ui/default.jpg",
            "episode-query",
        )
        .unwrap();
        let inserted_episode = insert_episode(
            podcast.id,
            "episode-query-1",
            "episode-query-guid-1",
            "Episode Query 1",
        );

        let list_response = server
            .test_server
            .get(&format!("/api/v1/podcasts/{}/episodes", podcast.id))
            .await;
        assert_eq!(list_response.status_code(), 200);
        let payload = list_response.json::<serde_json::Value>();
        assert_eq!(payload.as_array().unwrap().len(), 1);
        assert_eq!(
            payload[0]["podcastEpisode"]["episode_id"],
            json!("episode-query-1")
        );

        let get_response = server
            .test_server
            .get(&format!("/api/v1/episodes/{}", inserted_episode.episode_id))
            .await;
        assert_eq!(get_response.status_code(), 200);
        let single_payload = get_response.json::<serde_json::Value>();
        assert_eq!(
            single_payload["podcastEpisode"]["episode_id"],
            json!("episode-query-1")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_podcast_episode_by_id_returns_not_found_for_unknown_id() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .get("/api/v1/episodes/does-not-exist")
            .await;
        assert_eq!(response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_like_podcast_episode_persists_favorite() {
        let server = handle_test_startup().await;

        let podcast = crate::application::services::podcast::service::PodcastService::add_podcast_to_database(
            "Like Podcast",
            "like-podcast",
            "https://example.com/like.xml",
            "http://localhost:8080/ui/default.jpg",
            "like-podcast",
        )
        .unwrap();
        let inserted_episode = insert_episode(
            podcast.id,
            "like-episode-1",
            "like-guid-1",
            "Like Episode 1",
        );

        let response = server
            .test_server
            .put(&format!(
                "/api/v1/podcasts/{}/episodes/favor",
                inserted_episode.id
            ))
            .json(&json!({"favored": true}))
            .await;
        assert_eq!(response.status_code(), 200);

        let favorite = FavoritePodcastEpisodeService::default_service()
            .get_by_username_and_episode_id(&admin_username(), inserted_episode.id)
            .unwrap();
        assert!(favorite.is_some());
        assert!(favorite.unwrap().favorite);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_available_podcasts_not_in_webview_uses_local_subscriptions() {
        let server = handle_test_startup().await;

        diesel::insert_into(subs_dsl::subscriptions)
            .values((
                subs_dsl::username.eq(admin_username()),
                subs_dsl::device.eq("phone".to_string()),
                subs_dsl::podcast.eq("https://example.com/not-present.xml".to_string()),
                subs_dsl::created.eq(Utc::now().naive_utc()),
                subs_dsl::deleted.eq::<Option<chrono::NaiveDateTime>>(None),
            ))
            .execute(&mut get_connection())
            .unwrap();

        let response = server
            .test_server
            .get("/api/v1/podcasts/available/gpodder")
            .await;
        assert_eq!(response.status_code(), 200);

        let available = response.json::<serde_json::Value>();
        assert_eq!(available.as_array().unwrap().len(), 1);
        assert_eq!(available[0]["device"], json!("phone"));
        assert_eq!(
            available[0]["podcast"],
            json!("https://example.com/not-present.xml")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_timeline_returns_items_without_external_calls() {
        let server = handle_test_startup().await;

        let podcast = crate::application::services::podcast::service::PodcastService::add_podcast_to_database(
            "Timeline Podcast",
            "timeline-podcast",
            "https://example.com/timeline.xml",
            "http://localhost:8080/ui/default.jpg",
            "timeline-podcast",
        )
        .unwrap();
        insert_episode(
            podcast.id,
            "timeline-episode-1",
            "timeline-guid-1",
            "Timeline Episode 1",
        );

        let response = server
            .test_server
            .get("/api/v1/podcasts/timeline?favoredOnly=false&notListened=false&favoredEpisodes=false")
            .await;
        assert_eq!(response.status_code(), 200);
        let payload = response.json::<serde_json::Value>();
        assert!(payload["totalElements"].as_i64().unwrap() >= 1);
        assert!(!payload["data"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_find_all_chapters_of_episode_returns_empty_when_none_exist() {
        let server = handle_test_startup().await;

        let podcast = crate::application::services::podcast::service::PodcastService::add_podcast_to_database(
            "Chapter Podcast",
            "chapter-podcast",
            "https://example.com/chapter.xml",
            "http://localhost:8080/ui/default.jpg",
            "chapter-podcast",
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            "chapter-episode-1",
            "chapter-guid-1",
            "Chapter Episode 1",
        );

        let response = server
            .test_server
            .get(&format!(
                "/api/v1/podcasts/episodes/{}/chapters",
                episode.id
            ))
            .await;
        assert_eq!(response.status_code(), 200);
        let payload = response.json::<serde_json::Value>();
        assert!(payload.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_podcast_episode_locally_marks_episode_deleted() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let slug = format!("delete-local-podcast-{unique}");

        let podcast = crate::application::services::podcast::service::PodcastService::add_podcast_to_database(
            &unique_name("Delete Local Podcast"),
            &slug,
            &format!("https://example.com/{slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &slug,
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            &format!("delete-local-episode-{unique}"),
            &format!("delete-local-guid-{unique}"),
            "Delete Local Episode",
        );

        let response = server
            .test_server
            .delete(&format!("/api/v1/episodes/{}/download", episode.episode_id))
            .await;
        assert_eq!(response.status_code(), 204);

        let persisted = pe_dsl::podcast_episodes
            .filter(pe_dsl::episode_id.eq(episode.episode_id))
            .first::<PodcastEpisode>(&mut get_connection())
            .unwrap();
        assert!(persisted.deleted);
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_podcast_episode_locally_returns_not_found_for_unknown_id() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .delete("/api/v1/episodes/does-not-exist/download")
            .await;
        assert_eq!(response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_like_podcast_episode_rejects_invalid_payload() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let slug = format!("invalid-like-podcast-{unique}");

        let podcast = crate::application::services::podcast::service::PodcastService::add_podcast_to_database(
            &unique_name("Invalid Like Podcast"),
            &slug,
            &format!("https://example.com/{slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &slug,
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            &format!("invalid-like-episode-{unique}"),
            &format!("invalid-like-guid-{unique}"),
            "Invalid Like Episode",
        );

        let response = server
            .test_server
            .put(&format!("/api/v1/podcasts/{}/episodes/favor", episode.id))
            .json(&json!({ "favored": "yes" }))
            .await;
        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_admin_episode_handlers_return_forbidden_for_non_admin_user() {
        let non_admin = non_admin_user();

        let available_result = super::get_available_podcasts_not_in_webview(
            State(app_state()),
            Extension(non_admin.clone()),
        )
        .await;
        match available_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for get_available_podcasts_not_in_webview"),
        }

        let download_result = super::download_podcast_episodes_of_podcast(
            Extension(non_admin.clone()),
            Path("episode-id".to_string()),
        )
        .await;
        match download_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for download_podcast_episodes_of_podcast"),
        }

        let delete_result = super::delete_podcast_episode_locally(
            Path("episode-id".to_string()),
            Extension(non_admin),
        )
        .await;
        match delete_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for delete_podcast_episode_locally"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_download_unknown_episode_is_noop_and_returns_ok() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .put("/api/v1/podcasts/does-not-exist/episodes/download")
            .await;
        assert_eq!(response.status_code(), 200);
    }
}

