use crate::app_state::AppState;
use crate::application::services::podcast::service::PodcastService;
use crate::application::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use crate::controllers::websocket_controller::RSSAPiKey;
use common_infrastructure::error::ErrorSeverity::{Info, Warning};
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use axum::extract::{Request, State};
use axum::http::Uri;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::OptionalQuery;

pub async fn check_permissions_for_files(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<RSSAPiKey>,
    req: Request,
    next: Next,
) -> Result<Response, CustomError> {
    let request = query.and_then(|rss_api_key| rss_api_key.api_key);
    check_auth(req.uri().clone(), request, &state)?;
    Ok(next.run(req).await)
}

fn retrieve_podcast_or_podcast_episode(
    path: &str,
    encoded_path: &str,
) -> Result<(), CustomError> {
    let podcast_episode = PodcastEpisodeService::get_podcast_episodes_by_url(path)?;
    match podcast_episode {
        Some(podcast_episode) => {
            if podcast_episode.file_image_path.is_none() {
                return Ok(());
            }

            if let Some(image) = &podcast_episode.file_image_path
                && image == path
            {
                return Ok(());
            }

            let _ = podcast_episode;
            Ok(())
        }
        None => {
            let podcast = PodcastService::find_podcast_by_image_path(encoded_path)?;
            match podcast {
                Some(podcast) => {
                    let _ = podcast;
                    Ok(())
                }
                None => Err(CustomErrorInner::NotFound(Info).into()),
            }
        }
    }
}

fn check_auth(
    uri: Uri,
    api_key: Option<String>,
    state: &AppState,
) -> Result<(), CustomError> {
    match ENVIRONMENT_SERVICE.any_auth_enabled {
        true => {
            let api_key = &match api_key {
                Some(api_key) => api_key,
                None => {
                    return Err(CustomErrorInner::BadRequest(
                        "No query parameters found".to_string(),
                        Info,
                    )
                    .into());
                }
            };

            let api_key_exists = state.user_auth_service.is_api_key_valid(api_key);

            if !api_key_exists {
                return Err(CustomErrorInner::Forbidden(Warning).into());
            }
            let requested_path = uri
                .path()
                .to_string()
                .replace(ENVIRONMENT_SERVICE.server_url.as_str(), "");
            let requested_path = &requested_path[1..];
            let decoded_path = urlencoding::decode(requested_path).map_err(|_| {
                CustomErrorInner::BadRequest("Error while decoding URL".to_string(), Warning)
            })?;
            let decoded_path = decoded_path.as_ref();
            retrieve_podcast_or_podcast_episode(decoded_path, requested_path)
        }
        false => {
            let requested_path = uri
                .path()
                .to_string()
                .replace(ENVIRONMENT_SERVICE.server_url.as_str(), "");
            let requested_path = &requested_path[1..];
            let decoded_path = urlencoding::decode(requested_path).map_err(|_| {
                CustomErrorInner::BadRequest("Error while decoding URL".to_string(), Warning)
            })?;
            let decoded_path = decoded_path.as_ref();
            retrieve_podcast_or_podcast_episode(decoded_path, requested_path)
        }
    }
}

