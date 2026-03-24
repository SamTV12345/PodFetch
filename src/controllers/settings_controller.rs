use crate::application::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use crate::app_state::AppState;
use crate::application::services::episode_scan::service::EpisodeScanServiceImpl;
use crate::application::services::podcast::service::PodcastService;
use crate::adapters::api::url::resolve_server_url_from_headers;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::Response;
use axum::{Extension, Json};
use podfetch_domain::user::User;
use podfetch_web::settings::{
    self, Mode, OpmlError, OpmlPodcast, RescanError, Setting, SettingsControllerError,
    UpdateNameSettings,
};
use reqwest::StatusCode;

#[utoipa::path(
get,
path="/settings",
responses(
(status = 200, description = "Gets the current settings", body=Setting)),
tag="podcast_episodes"
)]
pub async fn get_settings(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
) -> Result<Json<Setting>, CustomError> {
    settings::get_settings(state.settings_service.as_ref(), requester.is_admin())
        .map(Json)
        .map_err(map_settings_controller_error)
}

#[utoipa::path(
    post,
    path="/settings/rescan-episodes",
    responses(
(status = 200, description = "Rescans all episodes for metadata")),
    tag="podcast_episodes"
)]
pub async fn rescan_episodes(Extension(requester): Extension<User>) -> Result<(), CustomError> {
    let scan_service = EpisodeScanServiceImpl::default();
    settings::rescan_episodes(&scan_service, requester.is_admin())
        .map(|stats| {
            log::info!(
                "Rescan complete: {} episodes scanned, {} chapters saved, {} skipped, {} errors",
                stats.episodes_scanned,
                stats.chapters_saved,
                stats.skipped,
                stats.errors
            );
        })
        .map_err(map_rescan_error)
}

#[utoipa::path(
put,
path="/settings",
request_body=Setting,
responses(
(status = 200, description = "Updates the current settings", body = Setting)),
tag="settings"
)]
pub async fn update_settings(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Json(settings): Json<Setting>,
) -> Result<Json<Setting>, CustomError> {
    settings::update_settings(
        state.settings_service.as_ref(),
        requester.is_admin(),
        settings,
    )
    .map(Json)
    .map_err(map_settings_controller_error)
}

#[utoipa::path(
put,
path="/settings/runcleanup",
responses(
(status = 200, description = "Runs a cleanup of old episodes")),
tag="settings"
)]
pub async fn run_cleanup(
    State(state): State<AppState>,
    requester: Extension<User>,
) -> Result<StatusCode, CustomError> {
    let settings =
        settings::cleanup_settings(state.settings_service.as_ref(), requester.is_admin())
            .map_err(map_settings_controller_error)?;
    PodcastEpisodeService::cleanup_old_episodes(settings.auto_cleanup_days);
    Ok(StatusCode::OK)
}

#[utoipa::path(
get,
path="/settings/opml/{type_of}",
responses(
(status = 200, description = "Gets the podcasts in opml format", body = String)),
tag="settings"
)]
pub async fn get_opml(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Path(type_of): Path<String>,
    headers: HeaderMap,
) -> Result<Response<String>, CustomError> {
    let podcasts_found = PodcastService::get_all_podcasts_raw()?
        .into_iter()
        .map(|podcast| OpmlPodcast {
            id: podcast.id,
            name: podcast.name,
            summary: podcast.summary,
            rssfeed: podcast.rssfeed,
        })
        .collect();
    let server_url = resolve_server_url_from_headers(&headers);
    let xml = settings::build_opml(
        podcasts_found,
        Mode::from(type_of),
        requester.api_key.as_deref(),
        state.environment.any_auth_enabled,
        &server_url,
    )
    .map_err(map_opml_error)?;
    let response = Response::builder()
        .header("Content-Type", "application/xml")
        .body(xml)
        .unwrap();
    Ok(response)
}

#[utoipa::path(
put,
path="/settings/name",
responses(
(status = 200, description = "Updates the name settings", body = Setting)),
tag="settings",
request_body=UpdateNameSettings
)]
pub async fn update_name(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Json(update_information): Json<UpdateNameSettings>,
) -> Result<Json<Setting>, CustomError> {
    settings::update_name(
        state.settings_service.as_ref(),
        requester.is_admin(),
        update_information,
    )
    .map(Json)
    .map_err(map_settings_controller_error)
}

use common_infrastructure::error::ErrorSeverity::{Critical, Debug, Error, Warning};
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

fn map_settings_controller_error(error: SettingsControllerError<CustomError>) -> CustomError {
    match error {
        SettingsControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        SettingsControllerError::NotFound => CustomErrorInner::NotFound(Debug).into(),
        SettingsControllerError::Service(error) => error,
    }
}

fn map_rescan_error(error: RescanError<CustomError>) -> CustomError {
    match error {
        RescanError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        RescanError::UnsupportedFormat => {
            CustomErrorInner::Conflict("Unsupported file format".to_string(), ErrorSeverity::Error)
                .into()
        }
        RescanError::Service(error) => error,
    }
}

fn map_opml_error(error: OpmlError) -> CustomError {
    match error {
        OpmlError::ApiKeyRequired => {
            CustomErrorInner::UnAuthorized("Please generate an api key".to_string(), Critical)
                .into()
        }
        OpmlError::Xml(_) => CustomErrorInner::Unknown(Error).into(),
    }
}

pub fn get_settings_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_settings))
        .routes(routes!(update_settings))
        .routes(routes!(run_cleanup))
        .routes(routes!(get_opml))
        .routes(routes!(update_name))
        .routes(routes!(rescan_episodes))
}

#[cfg(test)]
mod tests {
    use crate::adapters::persistence::dbconfig::db::get_connection;
    use crate::adapters::persistence::dbconfig::schema::settings::dsl as s_dsl;
    use crate::app_state::AppState;
    use crate::commands::startup::tests::handle_test_startup;
    use common_infrastructure::error::CustomErrorInner;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use axum::Extension;
    use axum::Json;
    use axum::extract::State;
    use diesel::RunQueryDsl;
    use podfetch_web::settings::{ReplacementStrategy, Setting};
    use serde_json::json;
    use serial_test::serial;

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    fn non_admin_user() -> podfetch_domain::user::User {
        UserTestDataBuilder::new().build()
    }

    fn app_state() -> AppState {
        AppState::new()
    }

    #[tokio::test]
    #[serial]
    async fn test_get_settings_returns_defaults() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/api/v1/settings").await;
        assert_eq!(response.status_code(), 200);

        let settings = response.json::<Setting>();
        assert_eq!(settings.id, 1);
        assert!(settings.auto_download);
        assert!(settings.auto_update);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_name_updates_settings_values() {
        let server = handle_test_startup().await;

        let update_response = server
            .test_server
            .put("/api/v1/settings/name")
            .json(&json!({
                "useExistingFilename": true,
                "replaceInvalidCharacters": true,
                "replacementStrategy": "replace-with-dash",
                "episodeFormat": "{episodeTitle}-{episodeDate}",
                "podcastFormat": "{podcastTitle}",
                "directPaths": true
            }))
            .await;
        assert_eq!(update_response.status_code(), 200);
        let updated = update_response.json::<Setting>();
        assert!(updated.use_existing_filename);
        assert!(updated.replace_invalid_characters);
        assert_eq!(updated.replacement_strategy, "replace-with-dash");
        assert_eq!(updated.episode_format, "{episodeTitle}-{episodeDate}");
        assert_eq!(updated.podcast_format, "{podcastTitle}");
        assert!(updated.direct_paths);

        let get_response = server.test_server.get("/api/v1/settings").await;
        assert_eq!(get_response.status_code(), 200);
        let persisted = get_response.json::<Setting>();
        assert_eq!(persisted.replacement_strategy, "replace-with-dash");
        assert_eq!(persisted.episode_format, "{episodeTitle}-{episodeDate}");
        assert_eq!(persisted.podcast_format, "{podcastTitle}");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_opml_local_returns_xml() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/api/v1/settings/opml/local").await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.maybe_content_type().unwrap(), "application/xml");
        let xml = response.text();
        assert!(xml.contains("<opml"));
        assert!(xml.contains("PodFetch Feed Export"));
    }

    #[tokio::test]
    #[serial]
    async fn test_update_settings_persists_values() {
        let server = handle_test_startup().await;

        let update_response = server
            .test_server
            .put("/api/v1/settings")
            .json(&json!({
                "id": 1,
                "autoDownload": false,
                "autoUpdate": false,
                "autoCleanup": true,
                "autoCleanupDays": 14,
                "podcastPrefill": 25,
                "replaceInvalidCharacters": true,
                "useExistingFilename": true,
                "replacementStrategy": "replace-with-dash",
                "episodeFormat": "{episodeTitle}",
                "podcastFormat": "{podcastTitle}",
                "directPaths": true
            }))
            .await;
        assert_eq!(update_response.status_code(), 200);
        let updated = update_response.json::<Setting>();
        assert!(!updated.auto_download);
        assert!(!updated.auto_update);
        assert!(updated.auto_cleanup);
        assert_eq!(updated.auto_cleanup_days, 14);
        assert!(updated.direct_paths);

        let get_response = server.test_server.get("/api/v1/settings").await;
        assert_eq!(get_response.status_code(), 200);
        let persisted = get_response.json::<Setting>();
        assert!(!persisted.auto_download);
        assert!(!persisted.auto_update);
        assert!(persisted.auto_cleanup);
        assert_eq!(persisted.auto_cleanup_days, 14);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_settings_rejects_invalid_payload() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .put("/api/v1/settings")
            .json(&json!({
                "id": 1,
                "autoDownload": "nope",
                "autoUpdate": true,
                "autoCleanup": true,
                "autoCleanupDays": 7,
                "podcastPrefill": 10,
                "replaceInvalidCharacters": false,
                "useExistingFilename": false,
                "replacementStrategy": "remove",
                "episodeFormat": "{episodeTitle}",
                "podcastFormat": "{podcastTitle}",
                "directPaths": false
            }))
            .await;

        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_update_name_rejects_invalid_replacement_strategy() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .put("/api/v1/settings/name")
            .json(&json!({
                "useExistingFilename": true,
                "replaceInvalidCharacters": true,
                "replacementStrategy": "invalid-value",
                "episodeFormat": "{episodeTitle}",
                "podcastFormat": "{podcastTitle}",
                "directPaths": false
            }))
            .await;

        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_run_cleanup_and_rescan_endpoints_return_ok() {
        let server = handle_test_startup().await;

        let cleanup_response = server.test_server.put("/api/v1/settings/runcleanup").await;
        assert_eq!(cleanup_response.status_code(), 200);

        let rescan_response = server
            .test_server
            .post("/api/v1/settings/rescan-episodes")
            .await;
        assert_eq!(rescan_response.status_code(), 200);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_settings_returns_not_found_when_settings_missing() {
        let server = handle_test_startup().await;

        diesel::delete(s_dsl::settings)
            .execute(&mut get_connection())
            .unwrap();

        let response = server.test_server.get("/api/v1/settings").await;
        assert_eq!(response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_admin_settings_handlers_return_forbidden_for_non_admin_user() {
        let user = non_admin_user();

        let get_result = super::get_settings(State(app_state()), Extension(user.clone())).await;
        match get_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for get_settings"),
        }

        let run_cleanup_result =
            super::run_cleanup(State(app_state()), Extension(user.clone())).await;
        match run_cleanup_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for run_cleanup"),
        }

        let update_result = super::update_settings(
            State(app_state()),
            Extension(user.clone()),
            Json(Setting {
                id: 1,
                auto_download: true,
                auto_update: true,
                auto_cleanup: false,
                auto_cleanup_days: 10,
                podcast_prefill: 12,
                replace_invalid_characters: false,
                use_existing_filename: false,
                replacement_strategy: "remove".to_string(),
                episode_format: "{episodeTitle}".to_string(),
                podcast_format: "{podcastTitle}".to_string(),
                direct_paths: false,
            }),
        )
        .await;
        match update_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for update_settings"),
        }

        let update_name_result = super::update_name(
            State(app_state()),
            Extension(user.clone()),
            Json(super::UpdateNameSettings {
                use_existing_filename: true,
                replace_invalid_characters: true,
                replacement_strategy: ReplacementStrategy::ReplaceWithDash,
                episode_format: "{episodeTitle}".to_string(),
                podcast_format: "{podcastTitle}".to_string(),
                direct_paths: false,
            }),
        )
        .await;
        match update_name_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for update_name"),
        }

        let rescan_result = super::rescan_episodes(Extension(user)).await;
        match rescan_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for rescan_episodes"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_settings_endpoints_return_client_error_for_wrong_http_methods() {
        let server = handle_test_startup().await;

        let post_settings = server.test_server.post("/api/v1/settings").await;
        assert_client_error_status(post_settings.status_code().as_u16());

        let get_update_name = server.test_server.get("/api/v1/settings/name").await;
        assert_client_error_status(get_update_name.status_code().as_u16());

        let post_run_cleanup = server.test_server.post("/api/v1/settings/runcleanup").await;
        assert_client_error_status(post_run_cleanup.status_code().as_u16());

        let get_rescan = server
            .test_server
            .get("/api/v1/settings/rescan-episodes")
            .await;
        assert_client_error_status(get_rescan.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_settings_endpoints_return_not_found_for_invalid_paths() {
        let server = handle_test_startup().await;

        let wrong_base = server.test_server.get("/api/v1/setting").await;
        assert_eq!(wrong_base.status_code(), 404);

        let typo_route = server.test_server.get("/api/v1/settings/runcleanups").await;
        assert_eq!(typo_route.status_code(), 404);

        let extra_segment = server
            .test_server
            .get("/api/v1/settings/opml/local/extra")
            .await;
        assert_eq!(extra_segment.status_code(), 404);
    }
}



