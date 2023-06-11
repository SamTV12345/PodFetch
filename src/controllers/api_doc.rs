use crate::models;
use crate::models::models::{
    PodCastAddModel, PodcastHistoryItem, PodcastWatchedEpisodeModel, PodcastWatchedPostModel,
};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::HttpResponse;
use futures::future::LocalBoxFuture;
use models::itunes_models::{ItunesModel};
use crate::models::podcast_episode::PodcastEpisode;
use models::podcasts::Podcast;
use std::future;
use std::future::Ready;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use crate::models::dto_models::PodcastFavorUpdateModel;
use crate::controllers::podcast_episode_controller::__path_find_all_podcast_episodes_of_podcast;
use crate::controllers::watch_time_controller::*;
use crate::controllers::podcast_controller::*;
use crate::controllers::notification_controller::*;
use crate::controllers::settings_controller::*;
use crate::models::settings::Setting;

#[derive(OpenApi)]
#[openapi(
paths(
    find_podcast,
    add_podcast,
    import_podcasts_from_opml,
    add_podcast_from_podindex,
    find_podcast_by_id,
    query_for_podcast,
    download_podcast,
    favorite_podcast,
    get_favored_podcasts,
    find_all_podcasts,
    get_settings,
    update_settings,
    run_cleanup,
    find_all_podcast_episodes_of_podcast,
    log_watchtime,
    get_last_watched,
    get_unread_notifications,
    dismiss_notifications,
    get_watchtime
),
components(
schemas(Podcast, PodcastEpisode, ItunesModel, PodcastHistoryItem,PodcastFavorUpdateModel,
PodcastWatchedEpisodeModel, PodcastWatchedPostModel, PodCastAddModel, Setting)
),
tags(
(name = "podcasts", description = "Podcast management endpoints."),
(name = "podcast_episodes", description = "Podcast episode management endpoints."),
(name = "watchtime", description = "Watchtime management endpoints.")
),
modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
        )
    }
}

pub struct ApiKeyMiddleware<S> {
    service: S,
    log_only: bool,
}

const API_KEY_NAME: &str = "todo_apikey";
const API_KEY: &str = "utoipa-rocks";

struct RequireApiKey;

impl<S> Transform<S, ServiceRequest> for RequireApiKey
where
    S: Service<
        ServiceRequest,
        Response = ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = ApiKeyMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ready(Ok(ApiKeyMiddleware {
            service,
            log_only: false,
        }))
    }
}

/// Log api key middleware only logs about missing or invalid api keys
struct LogApiKey;

impl<S> Transform<S, ServiceRequest> for LogApiKey
where
    S: Service<
        ServiceRequest,
        Response = ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = ApiKeyMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ready(Ok(ApiKeyMiddleware {
            service,
            log_only: true,
        }))
    }
}
impl<S> Service<ServiceRequest> for ApiKeyMiddleware<S>
where
    S: Service<
        ServiceRequest,
        Response = ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let response = |req: ServiceRequest, response: HttpResponse| -> Self::Future {
            Box::pin(async { Ok(req.into_response(response)) })
        };

        match req.headers().get(API_KEY_NAME) {
            Some(key) if key != API_KEY => {
                if self.log_only {
                    log::debug!("Incorrect api api provided!!!")
                } else {
                    return response(
                        req,
                        HttpResponse::Unauthorized().json(String::from("incorrect api key")),
                    );
                }
            }
            None => {
                if self.log_only {
                    log::debug!("Missing api key!!!")
                } else {
                    return response(
                        req,
                        HttpResponse::Unauthorized().json(String::from("missing api key")),
                    );
                }
            }
            _ => (), // just passthrough
        }

        if self.log_only {
            log::debug!("Performing operation")
        }

        let future = self.service.call(req);

        Box::pin(async move {
            let response = future.await?;

            Ok(response)
        })
    }
}
