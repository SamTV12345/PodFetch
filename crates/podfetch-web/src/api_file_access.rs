use crate::app_state::AppState;
use crate::file_access::{FileAccessError, check_permissions_for_files as check_file_access};
use crate::rss::RSSAPiKey;
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::OptionalQuery;
use common_infrastructure::error::ErrorSeverity::{Info, Warning};
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;

pub async fn check_permissions_for_files(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<RSSAPiKey>,
    req: Request,
    next: Next,
) -> Result<Response, CustomError> {
    let request = query.and_then(|rss_api_key| rss_api_key.api_key);
    check_file_access(
        req.uri().path(),
        request,
        ENVIRONMENT_SERVICE.any_auth_enabled,
        &ENVIRONMENT_SERVICE.server_url,
        |api_key| state.user_auth_service.is_api_key_valid(api_key),
        |path| {
            PodcastEpisodeService::get_podcast_episodes_by_url(path)
                .map(|episode| episode.and_then(|e| e.file_image_path))
        },
        |encoded_path| {
            PodcastService::find_podcast_by_image_path(encoded_path)
                .map(|podcast| podcast.is_some())
        },
    )
    .map_err(map_file_access_error)?;
    Ok(next.run(req).await)
}

fn map_file_access_error(error: FileAccessError<CustomError>) -> CustomError {
    match error {
        FileAccessError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        FileAccessError::NotFound => CustomErrorInner::NotFound(Info).into(),
        FileAccessError::BadRequest(message) => {
            CustomErrorInner::BadRequest(message, Warning).into()
        }
        FileAccessError::Service(error) => error,
    }
}
