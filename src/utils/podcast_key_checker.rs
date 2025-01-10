use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::controllers::websocket_controller::RSSAPiKey;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::utils::error::{CustomError, CustomErrorInner};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::web::Query;
use actix_web::{Error, HttpMessage};

pub enum PodcastOrPodcastEpisodeResource {
    Podcast(Podcast),
    PodcastEpisode(PodcastEpisode),
}

pub async fn check_permissions_for_files(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let request = req
        .extract::<Query<Option<RSSAPiKey>>>()
        .await?
        .into_inner()
        .map(|rss_api_key| rss_api_key.api_key);
    let extracted_podcast = check_auth(&req, request)?;
    req.extensions_mut().insert(extracted_podcast);
    next.call(req).await
}

fn retrieve_podcast_or_podcast_episode(
    path: &str,
) -> Result<PodcastOrPodcastEpisodeResource, CustomError> {
    let podcast_episode = PodcastEpisode::get_podcast_episodes_by_url(path)?;
    match podcast_episode {
        Some(podcast_episode) => Ok(PodcastOrPodcastEpisodeResource::PodcastEpisode(
            podcast_episode,
        )),
        None => {
            let podcast = Podcast::find_by_path(path)?;
            match podcast {
                Some(podcast) => Ok(PodcastOrPodcastEpisodeResource::Podcast(podcast)),
                None => Err(CustomErrorInner::NotFound.into()),
            }
        }
    }
}

fn check_auth(
    req: &ServiceRequest,
    api_key: Option<String>,
) -> Result<PodcastOrPodcastEpisodeResource, CustomError> {
    match ENVIRONMENT_SERVICE.any_auth_enabled {
        true => {
            let api_key = &match api_key {
                Some(api_key) => api_key,
                None => {
                    return Err(CustomErrorInner::BadRequest(
                        "No query parameters found".to_string(),
                    )
                    .into())
                }
            };

            let api_key_exists = User::check_if_api_key_exists(api_key);

            if !api_key_exists {
                return Err(CustomErrorInner::Forbidden.into());
            }

            let requested_path = req
                .path()
                .to_string()
                .replace(ENVIRONMENT_SERVICE.server_url.as_str(), "");
            retrieve_podcast_or_podcast_episode(&requested_path)
        }
        false => {
            let requested_path = req
                .path()
                .to_string()
                .replace(ENVIRONMENT_SERVICE.server_url.as_str(), "");
            retrieve_podcast_or_podcast_episode(&requested_path)
        }
    }
}
