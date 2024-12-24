use crate::controllers::notification_controller::*;
use crate::controllers::playlist_controller::*;
use crate::controllers::podcast_controller::*;
use crate::controllers::podcast_episode_controller::__path_find_all_podcast_episodes_of_podcast;
use crate::controllers::podcast_episode_controller::*;
use crate::controllers::settings_controller::*;
use crate::controllers::sys_info_controller::SysExtraInfo;
use crate::controllers::sys_info_controller::*;
use crate::controllers::tags_controller::*;
use crate::controllers::user_controller::InvitePostModel;
use crate::controllers::user_controller::UserOnboardingModel;
use crate::controllers::user_controller::*;
use crate::controllers::watch_time_controller::*;
use crate::controllers::websocket_controller::*;
use crate::models;
use crate::models::dto_models::PodcastFavorUpdateModel;
use crate::models::filter::Filter;
use crate::models::invite::Invite;
use crate::models::misc_models::{
    PodcastAddModel, PodcastWatchedEpisodeModel, PodcastWatchedPostModel,
};
use crate::models::notification::Notification;
use crate::models::opml_model::OpmlModel;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::settings::Setting;
use crate::models::tag::Tag;
use crate::models::user::User;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::HttpResponse;
use futures::future::LocalBoxFuture;
use models::itunes_models::ItunesModel;
use std::future;
use std::future::Ready;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

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
    get_settings,get_rss_feed_for_podcast,
    update_settings,
    run_cleanup,get_rss_feed,
    find_all_podcast_episodes_of_podcast,start_connection,
    log_watchtime,delete_user,get_invite_link,delete_invite,
    get_last_watched,update_role,create_invite,get_invites,get_invite,
    get_unread_notifications,login,get_info,get_users,get_user,
    dismiss_notifications,get_public_config,onboard_user,
    get_watchtime,get_timeline,download_podcast_episodes_of_podcast,update_name,get_sys_info,
    get_filter,search_podcasts,add_podcast_by_feed,refresh_all_podcasts,update_active_podcast,
    add_playlist,update_playlist,get_all_playlists,get_playlist_by_id,delete_playlist_by_id,delete_playlist_item,
delete_podcast,proxy_podcast,insert_tag, get_tags, delete_tag, update_tag, add_podcast_to_tag, delete_podcast_from_tag
),
components(
schemas(PodcastDto, PodcastEpisode, ItunesModel,PodcastFavorUpdateModel,
PodcastWatchedEpisodeModel, PodcastWatchedPostModel, PodcastAddModel,Notification, Setting,
Invite,LoginRequest,PlaylistDtoPost,Tag,
Filter,OpmlModel,DeletePodcast, UpdateNameSettings,SysExtraInfo,UserOnboardingModel,User,InvitePostModel, TagCreate,TagWithPodcast)
),
tags(
(name = "podcasts", description = "Podcast management endpoints."),
(name = "podcast_episodes", description = "Podcast episode management endpoints."),
(name = "watchtime", description = "Watchtime management endpoints."),
(name = "notifications", description = "Notification management endpoints."),
(name = "settings", description = "Settings management endpoints. Settings are globally scoped."),
(name = "info", description = "Gets multiple  information about your installation."),
(name = "playlist", description = "Playlist management endpoints."),
(name = "tags", description = "Tag management endpoints."),
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
