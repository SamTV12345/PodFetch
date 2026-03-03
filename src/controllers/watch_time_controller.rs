use crate::models::episode::{Episode, EpisodeDto};
use crate::models::misc_models::{
    PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel,
};
use crate::models::user::User;
use axum::extract::Path;
use axum::http::HeaderMap;
use axum::{Extension, Json};
use reqwest::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::utils::error::ErrorSeverity::Debug;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::url_builder::{resolve_server_url_from_headers, rewrite_env_server_url_prefix};

#[utoipa::path(
post,
path="/podcasts/episode",
responses(
(status = 200, description = "Logs a watchtime request.")),
tag="watchtime"
)]
pub async fn log_watchtime(
    Extension(requester): Extension<User>,
    Json(podcast_watch): Json<PodcastWatchedPostModel>,
) -> Result<StatusCode, CustomError> {
    let podcast_episode_id = podcast_watch.podcast_episode_id.clone();
    Episode::log_watchtime(podcast_watch, requester.username.clone())?;
    log::debug!("Logged watchtime for episode: {podcast_episode_id}");
    Ok(StatusCode::OK)
}

#[utoipa::path(
get,
path="/podcasts/episode/lastwatched",
responses(
(status = 200, description = "Gets the last watched podcast episodes.", body= Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>)),
tag="watchtime"
)]
pub async fn get_last_watched(
    Extension(requester): Extension<User>,
    headers: HeaderMap,
) -> Result<Json<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
    let episodes = Episode::get_last_watched_episodes(&requester)?
        .into_iter()
        .map(|mut item| {
            item.podcast_episode.local_url =
                rewrite_env_server_url_prefix(&item.podcast_episode.local_url, &server_url);
            item.podcast_episode.local_image_url =
                rewrite_env_server_url_prefix(&item.podcast_episode.local_image_url, &server_url);
            item.podcast.image_url =
                rewrite_env_server_url_prefix(&item.podcast.image_url, &server_url);
            item.podcast.podfetch_feed =
                rewrite_env_server_url_prefix(&item.podcast.podfetch_feed, &server_url);
            item
        })
        .collect();
    Ok(Json(episodes))
}

#[utoipa::path(
get,
path="/podcasts/episode/{id}",
responses(
(status = 200, description = "Gets watchtime by id.", body=EpisodeDto)),
tag="watchtime"
)]
pub async fn get_watchtime(
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<Json<EpisodeDto>, CustomError> {
    let watchtime = Episode::get_watchtime(&id, &requester.username)?;
    match watchtime {
        None => Err(CustomErrorInner::NotFound(Debug).into()),
        Some(w) => Ok(Json(w.convert_to_episode_dto())),
    }
}

pub fn get_watchtime_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(log_watchtime))
        .routes(routes!(get_last_watched))
        .routes(routes!(get_watchtime))
}

#[cfg(test)]
mod tests {
    use crate::adapters::persistence::dbconfig::db::get_connection;
    use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl as pe_dsl;
    use crate::commands::startup::tests::handle_test_startup;
    use crate::models::episode::{EpisodeAction, EpisodeDto};
    use crate::models::podcast_episode::PodcastEpisode;
    use crate::models::podcasts::Podcast;
    use serde_json::json;
    use serial_test::serial;

    fn insert_episode(
        podcast_id: i32,
        episode_id: &str,
        guid: &str,
        title: &str,
    ) -> PodcastEpisode {
        use diesel::ExpressionMethods;
        use diesel::RunQueryDsl;

        diesel::insert_into(pe_dsl::podcast_episodes)
            .values((
                pe_dsl::podcast_id.eq(podcast_id),
                pe_dsl::episode_id.eq(episode_id.to_string()),
                pe_dsl::name.eq(title.to_string()),
                pe_dsl::url.eq(format!("https://example.com/{episode_id}.mp3")),
                pe_dsl::date_of_recording.eq("2026-03-01T00:00:00Z".to_string()),
                pe_dsl::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe_dsl::total_time.eq(1800),
                pe_dsl::description.eq("watchtime test".to_string()),
                pe_dsl::guid.eq(guid.to_string()),
                pe_dsl::deleted.eq(false),
                pe_dsl::episode_numbering_processed.eq(false),
            ))
            .get_result::<PodcastEpisode>(&mut get_connection())
            .unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_log_watchtime_and_fetch_by_episode_id() {
        let server = handle_test_startup().await;

        let podcast = Podcast::add_podcast_to_database(
            "Watchtime Podcast",
            "watchtime-podcast",
            "https://example.com/watchtime.xml",
            "http://localhost:8080/ui/default.jpg",
            "watchtime-podcast",
        )
        .unwrap();
        let podcast_episode = insert_episode(
            podcast.id,
            "watchtime-episode-1",
            "watchtime-guid-1",
            "Watchtime Episode 1",
        );

        let log_response = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": podcast_episode.episode_id,
                "time": 133
            }))
            .await;
        assert_eq!(log_response.status_code(), 200);

        let get_response = server
            .test_server
            .get("/api/v1/podcasts/episode/watchtime-episode-1")
            .await;
        assert_eq!(get_response.status_code(), 200);
        let episode = get_response.json::<EpisodeDto>();
        assert_eq!(episode.position, Some(133));
        assert_eq!(episode.started, Some(133));
        assert_eq!(episode.action, EpisodeAction::Play);

        let last_watched = server
            .test_server
            .get("/api/v1/podcasts/episode/lastwatched")
            .await;
        assert_eq!(last_watched.status_code(), 200);
        let payload = last_watched.json::<serde_json::Value>();
        assert_eq!(payload.as_array().unwrap().len(), 1);
        assert_eq!(
            payload[0]["podcastEpisode"]["episode_id"],
            json!("watchtime-episode-1")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_log_watchtime_for_unknown_episode_returns_not_found() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": "does-not-exist",
                "time": 10
            }))
            .await;

        assert_eq!(response.status_code(), 404);
    }
}
