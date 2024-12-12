use std::io::Read;
use std::process::exit;
use actix_files::{Files, NamedFile};
use actix_web::{web, Error, HttpResponse, Scope};
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::{fn_service, ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::web::redirect;
use log::info;
use regex::Regex;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::adapters::api::controllers::podcast_controller::{add_podcast, add_podcast_by_feed, add_podcast_from_podindex, delete_podcast, download_podcast, favorite_podcast, find_all_podcasts, find_podcast, find_podcast_by_id, get_favored_podcasts, get_filter, get_podcast_settings, import_podcasts_from_opml, proxy_podcast, query_for_podcast, refresh_all_podcasts, retrieve_podcast_sample_format, search_podcasts, update_active_podcast, update_podcast_settings};
use crate::auth_middleware::AuthFilter;
use crate::constants::inner_constants::{CSS, ENVIRONMENT_SERVICE, JS};
use crate::{fix_links, index};
use crate::adapters::api::api_doc::ApiDoc;
use crate::adapters::api::controllers::device_controller::{get_devices_of_user, post_device};
use crate::adapters::api::controllers::manifest_controller::get_manifest;
use crate::adapters::api::controllers::notification_controller::{dismiss_notifications, get_unread_notifications};
use crate::adapters::api::controllers::playlist_controller::{add_playlist, delete_playlist_by_id, delete_playlist_item, get_all_playlists, get_playlist_by_id, update_playlist};
use crate::adapters::api::controllers::podcast_episode_controller::{delete_podcast_episode_locally, download_podcast_episodes_of_podcast, find_all_podcast_episodes_of_podcast, get_available_podcasts_not_in_webview, get_timeline, retrieve_episode_sample_format};
use crate::adapters::api::controllers::settings_controller::{get_opml, get_settings, run_cleanup, update_name, update_settings};
use crate::adapters::api::controllers::sys_info_controller::{get_info, get_public_config, get_sys_info};
use crate::adapters::api::controllers::tags_controller::{add_podcast_to_tag, delete_podcast_from_tag, delete_tag, get_tags, insert_tag, update_tag};
use crate::adapters::api::controllers::user_controller::{create_invite, delete_invite, delete_user, get_invite, get_invite_link, get_invites, get_user, get_users, onboard_user, update_role, update_user};
use crate::adapters::api::controllers::watch_time_controller::{get_last_watched, get_watchtime, log_watchtime};
use crate::adapters::api::controllers::websocket_controller::{get_rss_feed, get_rss_feed_for_podcast, start_connection};
use crate::gpodder::auth::authentication::login;
use crate::gpodder::parametrization::get_client_parametrization;
use crate::gpodder::subscription::subscriptions::{get_subscriptions, upload_subscription_changes};
use crate::models::settings::Setting;
use crate::service::environment_service::EnvironmentService;
use crate::utils::error::CustomError;
use crate::utils::podcast_key_checker::check_podcast_request;

pub fn get_api_config() -> Scope {
    web::scope("/api/v1").configure(config)
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_invite)
        .service(onboard_user)
        .service(login)
        .service(get_public_config)
        .service(get_private_api());
}

fn get_podcast_serving() -> Scope<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope("/podcasts")
        .wrap_fn(check_podcast_request)
        .service(Files::new("/", "podcasts").disable_content_disposition())
}

fn get_private_api() -> Scope<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<EitherBody<BoxBody>>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let middleware = AuthFilter::new();

    web::scope("")
        .wrap(middleware)
        .service(delete_playlist_item)
        .service(update_name)
        .service(get_filter)
        .service(search_podcasts)
        .service(add_podcast_by_feed)
        .service(refresh_all_podcasts)
        .service(get_info)
        .service(get_timeline)
        .service(get_available_podcasts_not_in_webview)
        .configure(config_secure_user_management)
        .service(find_podcast)
        .service(add_podcast)
        .service(find_all_podcasts)
        .service(find_all_podcast_episodes_of_podcast)
        .service(find_podcast_by_id)
        .service(log_watchtime)
        .service(get_last_watched)
        .service(get_watchtime)
        .service(get_unread_notifications)
        .service(dismiss_notifications)
        .service(download_podcast)
        .service(query_for_podcast)
        .service(download_podcast_episodes_of_podcast)
        .service(get_sys_info)
        .service(get_favored_podcasts)
        .service(favorite_podcast)
        .service(get_settings)
        .service(update_settings)
        .service(update_active_podcast)
        .service(import_podcasts_from_opml)
        .service(run_cleanup)
        .service(add_podcast_from_podindex)
        .service(delete_podcast)
        .service(get_opml)
        .service(add_playlist)
        .service(update_playlist)
        .service(get_all_playlists)
        .service(get_playlist_by_id)
        .service(delete_playlist_by_id)
        .service(delete_podcast_episode_locally)
        .service(insert_tag)
        .service(delete_tag)
        .service(update_tag)
        .service(get_tags)
        .service(add_podcast_to_tag)
        .service(delete_podcast_from_tag)
        .service(retrieve_episode_sample_format)
        .service(retrieve_podcast_sample_format)
        .service(update_podcast_settings)
        .service(get_podcast_settings)
}

pub fn config_secure_user_management(cfg: &mut web::ServiceConfig) {
    if ENVIRONMENT_SERVICE.get().unwrap().any_auth_enabled {
        cfg.service(get_secure_user_management());
    }
}

pub fn get_ui_config() -> Scope {
    web::scope("/ui")
        .service(redirect("", "./"))
        .route("/index.html", web::get().to(index))
        .route("/{path:[^.]*}", web::get().to(index))
        .default_service(fn_service(|req: ServiceRequest| async {
            let (req, _) = req.into_parts();
            let path = req.path();

            let test = Regex::new(r"/ui/(.*)").unwrap();
            let rs = test.captures(path).unwrap().get(1).unwrap().as_str();
            let file = NamedFile::open_async(format!("{}/{}", "./static", rs)).await?;
            let mut content = String::new();

            let type_of = file.content_type().to_string();
            let res = file.file().read_to_string(&mut content);

            match res {
                Ok(_) => {}
                Err(_) => return Ok(ServiceResponse::new(req.clone(), file.into_response(&req))),
            }
            if type_of.contains(CSS) || type_of.contains(JS) {
                content = fix_links(&content)
            }
            let res = HttpResponse::Ok().content_type(type_of).body(content);
            Ok(ServiceResponse::new(req, res))
        }))
}

pub fn get_secure_user_management() -> Scope {
    web::scope("/users")
        .service(create_invite)
        .service(get_invites)
        .service(get_user)
        .service(get_users)
        .service(update_role)
        .service(delete_user)
        .service(delete_invite)
        .service(get_invite_link)
        .service(update_user)
}

pub fn insert_default_settings_if_not_present() -> Result<(), CustomError> {
    let settings = Setting::get_settings()?;
    match settings {
        Some(_) => {
            info!("Settings already present");
            Ok(())
        }
        None => {
            info!("No settings found, inserting default settings");
            Setting::insert_default_settings()?;
            Ok(())
        }
    }
}

pub fn global_routes() -> Scope {
    let env_service = ENVIRONMENT_SERVICE.get().unwrap();
    let base_path = env_service.sub_directory.clone().unwrap_or("/".to_string());
    let openapi = ApiDoc::openapi();
    let service = get_api_config();

    web::scope(&base_path)
        .service(get_client_parametrization)
        .service(proxy_podcast)
        .service(get_ui_config())
        .service(get_podcast_serving())
        .service(redirect("/swagger-ui", "/swagger-ui/"))
        .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi))
        .service(redirect("/", "./ui/"))
        .service(service)
        .service(start_connection)
        .service(get_rss_feed)
        .service(get_manifest)
        .service(get_rss_feed_for_podcast)
}


pub fn get_gpodder_api() -> Scope {
    if ENVIRONMENT_SERVICE
        .get()
        .unwrap()
        .gpodder_integration_enabled
    {
        web::scope("/api/2")
            .service(login)
            .service(get_authenticated_gpodder())
    } else {
        web::scope("/api/2")
    }
}


fn get_authenticated_gpodder() -> Scope<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<EitherBody<BoxBody>>,
        Error = Error,
        InitError = (),
    >,
> {
    web::scope("")
        .wrap(crate::gpodder::session_middleware::CookieFilter::new())
        .service(post_device)
        .service(get_devices_of_user)
        .service(get_subscriptions)
        .service(upload_subscription_changes)
        .service(crate::gpodder::episodes::gpodder_episodes::get_episode_actions)
        .service(crate::gpodder::episodes::gpodder_episodes::upload_episode_actions)
}

pub fn check_server_config() {
    let env_service = ENVIRONMENT_SERVICE.get().unwrap();
    if env_service.http_basic && (env_service.password.is_none() || env_service.username.is_none())
    {
        eprintln!("BASIC_AUTH activated but no username or password set. Please set username and password in the .env file.");
        exit(1);
    }

    if env_service.gpodder_integration_enabled
        && !(env_service.http_basic || env_service.oidc_configured || env_service.reverse_proxy)
    {
        eprintln!("GPODDER_INTEGRATION_ENABLED activated but no BASIC_AUTH or OIDC_AUTH set. Please set BASIC_AUTH or OIDC_AUTH in the .env file.");
        exit(1);
    }

    if check_if_multiple_auth_is_configured(env_service) {
        eprintln!("You cannot have oidc and basic auth enabled at the same time. Please disable one of them.");
        exit(1);
    }
}

fn check_if_multiple_auth_is_configured(env: &EnvironmentService) -> bool {
    let mut num_of_auth_count = 0;
    if env.http_basic {
        num_of_auth_count += 1;
    }
    if env.oidc_configured {
        num_of_auth_count += 1;
    }
    if env.reverse_proxy {
        num_of_auth_count += 1;
    }
    num_of_auth_count > 1
}