use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[macro_use]
extern crate serde_derive;
extern crate core;
extern crate serde_json;


use clokwerk::{Scheduler, TimeUnits};
use diesel::r2d2::ConnectionManager;
use jsonwebtoken::jwk::{
    AlgorithmParameters, CommonParameters, Jwk, KeyAlgorithm, RSAKeyParameters, RSAKeyType,
};
use log::info;
use maud::{html, Markup};
use r2d2::Pool;
use regex::Regex;
use std::collections::HashSet;
use std::env::args;
use std::io::Read;
use std::ops::DerefMut;
use std::process::exit;
use std::sync::OnceLock;
use std::time::Duration;
use std::{env, thread};
use std::fmt::format;
use axum::extract::Request;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Router;
use axum::routing::{get, post};
use file_format::FileFormat;
use socketioxide::SocketIoBuilder;
use tokio::{fs, spawn, try_join};
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable as RServable};
mod controllers;
use crate::adapters::api::controllers::routes::{get_gpodder_api, global_routes};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::DBType;
use crate::auth_middleware::AuthFilter;
use crate::command_line_runner::start_command_line;
use crate::constants::inner_constants::{CSS, ENVIRONMENT_SERVICE, JS};
use crate::controllers::notification_controller::{dismiss_notifications, get_notification_router, get_unread_notifications};
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;
use crate::controllers::playlist_controller::{add_playlist, delete_playlist_by_id, delete_playlist_item, get_all_playlists, get_playlist_by_id, get_playlist_router, update_playlist};
use crate::controllers::podcast_controller::{add_podcast, add_podcast_by_feed, delete_podcast, find_all_podcasts, find_podcast, find_podcast_by_id, get_filter, get_podcast_router, get_podcast_settings, refresh_all_podcasts, retrieve_podcast_sample_format, search_podcasts, update_podcast_settings};
use crate::controllers::podcast_controller::{
    add_podcast_from_podindex, download_podcast, favorite_podcast, get_favored_podcasts,
    import_podcasts_from_opml, query_for_podcast, update_active_podcast,
};
use crate::controllers::podcast_episode_controller::{delete_podcast_episode_locally, download_podcast_episodes_of_podcast, find_all_podcast_episodes_of_podcast, get_available_podcasts_not_in_webview, get_podcast_episode_router, get_timeline, like_podcast_episode, retrieve_episode_sample_format};
use crate::controllers::settings_controller::{get_opml, get_settings, get_settings_router, run_cleanup, update_name, update_settings};
use crate::controllers::sys_info_controller::{get_info, get_public_config, get_sys_info, get_sys_info_router, login};
use crate::controllers::tags_controller::{add_podcast_to_tag, delete_podcast_from_tag, delete_tag, get_tags, get_tags_router, insert_tag, update_tag};
use crate::controllers::user_controller::{create_invite, delete_invite, delete_user, get_invite, get_invite_link, get_invites, get_user, get_user_router, get_users, onboard_user, update_role, update_user};
use crate::controllers::watch_time_controller::{get_last_watched, get_watchtime, get_watchtime_router, log_watchtime};
pub use controllers::controller_utils::*;
use crate::controllers::manifest_controller::get_manifest_router;
use crate::controllers::websocket_controller::get_websocket_router;

mod constants;
mod db;
mod models;
mod service;
use crate::models::oidc_model::{CustomJwk, CustomJwkSet};
use crate::models::podcasts::Podcast;
use crate::models::session::Session;
use crate::models::settings::Setting;

use crate::service::environment_service::EnvironmentService;
use crate::service::file_service::FileService;

use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::http_client::get_http_client;

mod config;

mod adapters;
mod application;
mod auth_middleware;
mod command_line_runner;
mod domain;
mod exception;
mod gpodder;
pub mod mutex;
mod test_utils;
pub mod utils;

type DbPool = Pool<ConnectionManager<DBType>>;

import_database_config!();

pub fn run_poll() -> Result<(), CustomError> {
    //check for new episodes
    let podcats_result = Podcast::get_all_podcasts()?;
    for podcast in podcats_result {
        if podcast.active {
            let podcast_clone = podcast.clone();
            PodcastEpisodeService::insert_podcast_episodes(podcast)?;
            PodcastService::schedule_episode_download(podcast_clone)?;
        }
    }
    Ok(())
}

fn fix_links(content: &str) -> String {
    let dir = ENVIRONMENT_SERVICE.sub_directory.clone().unwrap() + "/ui/";
    content.replace("/ui/", &dir)
}

pub static INDEX_HTML: OnceLock<Markup> = OnceLock::new();

async fn index() -> Markup {
    let html = INDEX_HTML.get_or_init(|| {
        let dir = ENVIRONMENT_SERVICE.sub_directory.clone().unwrap() + "/ui/";
        let manifest_json_location =
            ENVIRONMENT_SERVICE.sub_directory.clone().unwrap() + "/manifest.json";
        let found_files = std::fs::read_dir("./static/assets/")
            .expect("Could not read directory")
            .map(|x| x.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<String>>();
        let js_file = found_files
            .iter()
            .filter(|x| x.starts_with("index") && x.ends_with(".js"))
            .collect::<Vec<&String>>()[0];
        let css_file = found_files
            .iter()
            .filter(|x| x.starts_with("index") && x.ends_with(".css"))
            .collect::<Vec<&String>>()[0];

        let html = html! {
            html {
                head {
                    meta charset="utf-8";
                    meta name="viewport" content="width=device-width, initial-scale=1";
                    title {"Podfetch"};
                    link rel="icon" type="image/png" href="/ui/favicon.ico";
                    link rel="manifest" href=(manifest_json_location);
                    script type="module" crossorigin src=(format!("{}{}{}",dir.clone(),
                        "assets/",js_file))
                    {};
                    link rel="stylesheet" href=(format!("{}{}{}",dir.clone(), "assets/",css_file));
                }
            body {
            div id="root" {};
            div id="modal" {};
            div id="modal1"{};
            div id="modal2"{};
            div id="confirm-modal"{};
                }

        }};
        html
    });

    html.clone()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!(
        "Debug file located at {}",
        concat!(env!("OUT_DIR"), "/built.rs")
    );

    EnvironmentService::print_banner();
    ENVIRONMENT_SERVICE.get_environment();
    if args().len() > 1 {
        if let Err(e) = start_command_line(args()).await {
            log::error!("Error in command line: {}", e);
            exit(1);
        }
        exit(0)
    }

    run_migrations();

    check_server_config();

    //services

    match FileService::create_podcast_root_directory_exists() {
        Ok(_) => {}
        Err(e) => {
            log::error!("Could not create podcast root directory: {}", e);
            panic!("Could not create podcast root directory: {}", e);
        }
    }

    insert_default_settings_if_not_present().expect("Could not insert default settings");

    thread::spawn(|| {
        let mut scheduler = Scheduler::new();

        let polling_interval = ENVIRONMENT_SERVICE.get_polling_interval();
        scheduler.every(polling_interval.minutes()).run(|| {
            let settings = Setting::get_settings().unwrap();
            match settings {
                Some(settings) => {
                    if settings.auto_update {
                        info!("Polling for new episodes");
                        match run_poll() {
                            Ok(_) => {
                                log::info!("Polling for new episodes successful");
                            }
                            Err(e) => {
                                log::error!("Error polling for new episodes: {}", e);
                            }
                        }
                    }
                }
                None => {
                    log::error!("Could not get settings from database");
                }
            }
        });

        scheduler.every(1.day()).run(move || {
            // Clears the session ids once per day
            let conn = &mut get_connection();
            Session::cleanup_sessions(conn).expect(
                "Error clearing old \
            sessions",
            );
            let settings = Setting::get_settings().unwrap();
            match settings {
                Some(settings) => {
                    if settings.auto_cleanup {
                        PodcastEpisodeService::cleanup_old_episodes(settings.auto_cleanup_days)
                    }
                }
                None => {
                    log::error!("Could not get settings from database");
                }
            }
        });

        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(1000));
        }
    });

    let key_param: Option<RSAKeyParameters>;
    let mut hash = HashSet::new();
    let jwk: Option<Jwk>;

    match ENVIRONMENT_SERVICE.oidc_config.clone() {
        Some(jwk_config) => {
            let resp = get_http_client()
                .get(&jwk_config.jwks_uri)
                .send()
                .await
                .unwrap()
                .json::<CustomJwkSet>()
                .await;

            match resp {
                Ok(res) => {
                    let oidc = res
                        .clone()
                        .keys
                        .into_iter()
                        .filter(|x| x.alg.eq(&"RS256"))
                        .collect::<Vec<CustomJwk>>()
                        .first()
                        .cloned();

                    if oidc.is_none() {
                        panic!("No RS256 key found in JWKS")
                    }

                    key_param = Some(RSAKeyParameters {
                        e: oidc.clone().unwrap().e,
                        n: oidc.unwrap().n.clone(),
                        key_type: RSAKeyType::RSA,
                    });

                    jwk = Some(Jwk {
                        common: CommonParameters {
                            public_key_use: None,
                            key_id: None,
                            x509_url: None,
                            x509_chain: None,
                            x509_sha1_fingerprint: None,
                            key_operations: None,
                            key_algorithm: Some(KeyAlgorithm::RS256),
                            x509_sha256_fingerprint: None,
                        },
                        algorithm: AlgorithmParameters::RSA(key_param.clone().unwrap()),
                    });
                }
                Err(_) => {
                    panic!("Error downloading OIDC")
                }
            }
        }
        _ => {
            key_param = None;
            jwk = None;
        }
    }

    if let Some(oidc_config) = ENVIRONMENT_SERVICE.oidc_config.clone() {
        hash.insert(oidc_config.client_id);
    }

    let sub_dir = ENVIRONMENT_SERVICE
        .sub_directory
        .clone()
        .unwrap_or("/".to_string());

    let ui_dir = format!("{}/ui", sub_dir);
    let (layer, io) = SocketIoBuilder::new().build_layer();

    let (router, api) = OpenApiRouter::new()
        .route("/", get(Redirect::to(&ui_dir)))
        .layer(layer)
        .split_for_parts();
    let router = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
        .merge(Redoc::with_url("/redoc", api.clone()))
        // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
        // via SwaggerUi instead we only make rapidoc to point to the existing doc.
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        // Alternative to above
        // .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", api).path("/rapidoc"))
        .merge(Scalar::with_url("/scalar", api));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;


    axum::serve(listener, router).await.unwrap();
    Ok(())
}

pub fn run_migrations() {
    let mut conn = get_connection();
    let conn = conn.deref_mut();

    match conn {
        DBType::Postgresql(ref mut conn) => {
            let res_migration = conn.run_pending_migrations(POSTGRES_MIGRATIONS);

            if res_migration.is_err() {
                panic!("Could not run migrations: {}", res_migration.err().unwrap());
            }
        }
        DBType::Sqlite(ref mut conn) => {
            let res_migration = conn.run_pending_migrations(SQLITE_MIGRATIONS);

            if res_migration.is_err() {
                panic!("Could not run migrations: {}", res_migration.err().unwrap());
            }
        }
    }
}

pub fn get_api_config() -> Router {
    Router::new()
        .nest("/api/v1", Router::new().merge(config()))

}

fn config() -> Router {
    Router::new()
        .route("/users/invites/{invite_id}", get(get_invite))
        .route("/users", post(onboard_user))
        .route("/login", post(login))
        .route("/sys/config", get(get_public_config))
        .merge(get_private_api())
}

fn get_private_api() -> Router {
    Router::new()
        .merge(get_playlist_router())
        .merge(get_podcast_router())
        .merge(get_sys_info_router())
        .merge(get_watchtime_router())
        .merge(get_manifest_router())
        .merge(get_notification_router())
        .merge(get_podcast_episode_router())
        .merge(get_settings_router())
        .merge(get_sys_info_router())
        .merge(get_tags_router())
        .merge(get_user_router())
        .merge(get_websocket_router())
}


async fn handle_ui_access(req: Request) -> Result<impl IntoResponse, CustomError> {
    let (req_parts, _) = req.into_parts();
    let path = req_parts.uri.path();

    let test = Regex::new(r"/ui/(.*)").unwrap();
    let rs = test.captures(path).unwrap().get(1).unwrap().as_str();
    let file_path = format!("{}/{}", "./static", rs);

    let type_of = match FileFormat::from_file(&file_path) {
        Ok(e)=>Ok(e.media_type()),
        Err(_)=>Err(CustomErrorInner::NotFound.into())
    }?;

    let mut content = match fs::read_to_string(file_path).await {
        Ok(e)=>Ok(e),
        Err(_)=>return Err(CustomErrorInner::NotFound.into())
    }?;

    if type_of.contains(CSS) || type_of.contains(JS) {
        content = fix_links(&content)
    }
    let res = Response::builder().header("Content-Type", type_of).body(content).unwrap();
    Ok(res)
}

pub fn get_ui_config() -> Router {
    Router::new().nest("/ui", Router::new()
        .route("/index.html", get(index))
        .route("/{path:[^.]*}", get(index))
        .fallback(handle_ui_access))
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

pub fn check_server_config() {
    if ENVIRONMENT_SERVICE.http_basic
        && (ENVIRONMENT_SERVICE.password.is_none() || ENVIRONMENT_SERVICE.username.is_none())
    {
        eprintln!("BASIC_AUTH activated but no username or password set. Please set username and password in the .env file.");
        exit(1);
    }

    if ENVIRONMENT_SERVICE.gpodder_integration_enabled
        && !(ENVIRONMENT_SERVICE.http_basic
            || ENVIRONMENT_SERVICE.oidc_configured
            || ENVIRONMENT_SERVICE.reverse_proxy)
    {
        eprintln!("GPODDER_INTEGRATION_ENABLED activated but no BASIC_AUTH or OIDC_AUTH set. Please set BASIC_AUTH or OIDC_AUTH in the .env file.");
        exit(1);
    }

    if check_if_multiple_auth_is_configured() {
        eprintln!("You cannot have oidc and basic auth enabled at the same time. Please disable one of them.");
        exit(1);
    }
}

fn check_if_multiple_auth_is_configured() -> bool {
    let mut num_of_auth_count = 0;
    if ENVIRONMENT_SERVICE.http_basic {
        num_of_auth_count += 1;
    }
    if ENVIRONMENT_SERVICE.oidc_configured {
        num_of_auth_count += 1;
    }
    if ENVIRONMENT_SERVICE.reverse_proxy {
        num_of_auth_count += 1;
    }
    num_of_auth_count > 1
}
