use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[macro_use]
extern crate serde_derive;
extern crate core;
extern crate serde_json;

use actix::Actor;
use actix_files::{Files, NamedFile};
use actix_web::dev::{fn_service, ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::{Condition, Logger};
use actix_web::web::{redirect, Data};
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Scope};
use clokwerk::{Scheduler, TimeUnits};
use std::collections::HashSet;
use std::env::{args, var};
use std::io::Read;
use std::sync::Mutex;
use std::time::Duration;
use std::{env, thread};

use actix_web::body::{BoxBody, EitherBody};
use diesel::r2d2::ConnectionManager;
use jsonwebtoken::jwk::{
    AlgorithmParameters, CommonParameters, Jwk, KeyAlgorithm, RSAKeyParameters, RSAKeyType,
};
use log::info;
use r2d2::Pool;
use regex::Regex;
use std::process::exit;

use tokio::task::spawn_blocking;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod controllers;
use crate::auth_middleware::AuthFilter;
use crate::command_line_runner::start_command_line;
use crate::config::dbconfig::establish_connection;
use crate::config::dbconfig::ConnectionOptions;
use crate::constants::inner_constants::{BASIC_AUTH, CSS, ENVIRONMENT_SERVICE, JS, OIDC_AUTH};
use crate::controllers::api_doc::ApiDoc;
use crate::controllers::notification_controller::{
    dismiss_notifications, get_unread_notifications,
};
use crate::controllers::playlist_controller::{
    add_playlist, delete_playlist_by_id, delete_playlist_item, get_all_playlists,
    get_playlist_by_id, update_playlist,
};
use crate::controllers::podcast_controller::{
    add_podcast, add_podcast_by_feed, delete_podcast, find_all_podcasts, find_podcast,
    find_podcast_by_id, get_filter, proxy_podcast, refresh_all_podcasts,
    retrieve_podcast_sample_format, search_podcasts,
};
use crate::controllers::podcast_controller::{
    add_podcast_from_podindex, download_podcast, favorite_podcast, get_favored_podcasts,
    import_podcasts_from_opml, query_for_podcast, update_active_podcast,
};
use crate::controllers::podcast_episode_controller::{
    delete_podcast_episode_locally, download_podcast_episodes_of_podcast,
    find_all_podcast_episodes_of_podcast, get_timeline, retrieve_episode_sample_format,
};
use crate::controllers::settings_controller::{
    get_opml, get_settings, run_cleanup, update_name, update_settings,
};
use crate::controllers::sys_info_controller::{get_info, get_public_config, get_sys_info, login};
use crate::controllers::user_controller::{
    create_invite, delete_invite, delete_user, get_invite, get_invite_link, get_invites, get_user,
    get_users, onboard_user, update_role, update_user,
};
use crate::controllers::watch_time_controller::{get_last_watched, get_watchtime, log_watchtime};
use crate::controllers::websocket_controller::{
    get_rss_feed, get_rss_feed_for_podcast, start_connection,
};
use crate::dbconfig::DBType;
pub use controllers::controller_utils::*;

mod constants;
mod db;
mod models;
mod service;
use crate::gpodder::parametrization::get_client_parametrization;
use crate::gpodder::routes::get_gpodder_api;
use crate::models::oidc_model::{CustomJwk, CustomJwkSet};
use crate::models::podcasts::Podcast;
use crate::models::session::Session;
use crate::models::settings::Setting;

use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::file_service::FileService;
use crate::service::jwkservice::JWKService;
use crate::service::logging_service::init_logging;

use crate::service::notification_service::NotificationService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::service::settings_service::SettingsService;
use crate::utils::environment_variables::is_env_var_present_and_true;
use crate::utils::error::CustomError;
use crate::utils::podcast_key_checker::check_podcast_request;

mod config;

mod auth_middleware;
mod command_line_runner;
mod dbconfig;
mod exception;
mod gpodder;
pub mod mutex;
pub mod utils;
type DbPool = Pool<ConnectionManager<DBType>>;

import_database_config!();

pub fn run_poll(mut podcast_service: PodcastService) -> Result<(), CustomError> {
    //check for new episodes
    let conn = &mut establish_connection();
    let podcats_result = Podcast::get_all_podcasts(conn)?;
    for podcast in podcats_result {
        if podcast.active {
            let podcast_clone = podcast.clone();
            PodcastEpisodeService::insert_podcast_episodes(conn, podcast)?;
            podcast_service.schedule_episode_download(
                podcast_clone,
                None,
                &mut establish_connection(),
            )?;
        }
    }
    Ok(())
}

fn fix_links(content: &str) -> String {
    let dir = var("SUB_DIRECTORY").unwrap() + "/ui/";
    content.replace("/ui/", &dir)
}

async fn index() -> impl Responder {
    let index_html = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/index.html"));

    HttpResponse::Ok()
        .content_type("text/html")
        .body(fix_links(index_html))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!(
        "Debug file located at {}",
        concat!(env!("OUT_DIR"), "/built.rs")
    );
    init_logging();
    ENVIRONMENT_SERVICE.get_or_init(EnvironmentService::new);

    if args().len() > 1 {
        spawn_blocking(move || start_command_line(args()))
            .await
            .expect("TODO: panic message");
        exit(0)
    }

    let conn = establish_connection();

    match conn {
        DBType::Postgresql(mut conn) => {
            let res_migration = conn.run_pending_migrations(POSTGRES_MIGRATIONS);

            if res_migration.is_err() {
                panic!("Could not run migrations: {}", res_migration.err().unwrap());
            }
        }
        DBType::Sqlite(mut conn) => {
            let res_migration = conn.run_pending_migrations(SQLITE_MIGRATIONS);

            if res_migration.is_err() {
                panic!("Could not run migrations: {}", res_migration.err().unwrap());
            }
        }
    }

    check_server_config();
    let pool;
    {
        let conn = establish_connection();
        match conn {
            DBType::Postgresql(_) => {
                pool = init_postgres_db_pool(&ENVIRONMENT_SERVICE.get().unwrap().database_url)
                    .await
                    .expect("Failed to connect to database");
            }
            DBType::Sqlite(_) => {
                pool = init_sqlite_db_pool(&ENVIRONMENT_SERVICE.get().unwrap().database_url)
                    .await
                    .expect("Failed to connect to database");
            }
        }
    }
    let data_pool = Data::new(pool);

    //services
    let podcast_service = PodcastService::new();
    let file_service = FileService::new_db();
    let notification_service = NotificationService::new();
    let settings_service = SettingsService::new();
    let lobby = Lobby::default();

    let chat_server = lobby.start();

    EnvironmentService::print_banner();
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

        ENVIRONMENT_SERVICE.get().unwrap().get_environment();
        let polling_interval = ENVIRONMENT_SERVICE.get().unwrap().get_polling_interval();
        scheduler.every(polling_interval.minutes()).run(|| {
            let conn = &mut establish_connection();
            let settings = Setting::get_settings(conn).unwrap();
            match settings {
                Some(settings) => {
                    if settings.auto_update {
                        let podcast_service = PodcastService::new();
                        info!("Polling for new episodes");
                        run_poll(podcast_service).unwrap();
                    }
                }
                None => {
                    log::error!("Could not get settings from database");
                }
            }
        });

        scheduler.every(1.day()).run(move || {
            // Clears the session ids once per day
            let conn = &mut establish_connection();
            Session::cleanup_sessions(conn).expect(
                "Error clearing old \
            sessions",
            );
            let settings = Setting::get_settings(conn).unwrap();
            match settings {
                Some(settings) => {
                    if settings.auto_cleanup {
                        PodcastEpisodeService::cleanup_old_episodes(
                            settings.auto_cleanup_days,
                            &mut establish_connection(),
                        )
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

    match ENVIRONMENT_SERVICE.get().unwrap().oidc_config.clone() {
        Some(jwk_config) => {
            let resp = reqwest::get(&jwk_config.jwks_uri)
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

    if let Some(oidc_config) = ENVIRONMENT_SERVICE.get().unwrap().oidc_config.clone() {
        hash.insert(oidc_config.client_id);
    }

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(key_param.clone()))
            .app_data(Data::new(jwk.clone()))
            .app_data(Data::new(hash.clone()))
            .service(redirect("/", var("SUB_DIRECTORY").unwrap() + "/ui/"))
            .service(get_gpodder_api())
            .service(get_global_scope())
            .app_data(Data::new(chat_server.clone()))
            .app_data(Data::new(Mutex::new(podcast_service.clone())))
            .app_data(Data::new(Mutex::new(file_service.clone())))
            .app_data(Data::new(Mutex::new(notification_service.clone())))
            .app_data(Data::new(Mutex::new(settings_service.clone())))
            .app_data(data_pool.clone())
            .app_data(Data::new(Mutex::new(JWKService::new())))
            .wrap(Condition::new(cfg!(debug_assertions), Logger::default()))
    })
    .workers(4)
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

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

pub fn get_global_scope() -> Scope {
    let base_path = var("SUB_DIRECTORY").unwrap_or("/".to_string());
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
        .service(get_rss_feed_for_podcast)
}

fn get_podcast_serving() -> actix_web::Scope<
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
        .service(retrieve_episode_sample_format)
        .service(retrieve_podcast_sample_format)
}

pub fn config_secure_user_management(cfg: &mut web::ServiceConfig) {
    if is_env_var_present_and_true(BASIC_AUTH) || is_env_var_present_and_true(OIDC_AUTH) {
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
    let conn = &mut establish_connection();
    let settings = Setting::get_settings(conn)?;
    match settings {
        Some(_) => {
            info!("Settings already present");
            Ok(())
        }
        None => {
            info!("No settings found, inserting default settings");
            Setting::insert_default_settings(conn)?;
            Ok(())
        }
    }
}

pub fn check_server_config() {
    let env_service = ENVIRONMENT_SERVICE.get().unwrap();
    if env_service.http_basic && (env_service.password.is_none() || env_service.username.is_none())
    {
        eprintln!("BASIC_AUTH activated but no username or password set. Please set username and password in the .env file.");
        exit(1);
    }

    if env_service.gpodder_integration_enabled
        && !(env_service.http_basic || env_service.oidc_configured)
    {
        eprintln!("GPODDER_INTEGRATION_ENABLED activated but no BASIC_AUTH or OIDC_AUTH set. Please set BASIC_AUTH or OIDC_AUTH in the .env file.");
        exit(1);
    }

    if env_service.http_basic && env_service.oidc_configured {
        eprintln!("You cannot have oidc and basic auth enabled at the same time. Please disable one of them.");
        exit(1);
    }
}

async fn init_postgres_db_pool(
    database_url: &str,
) -> Result<Pool<ConnectionManager<DBType>>, String> {
    let db_connections = var("DB_CONNECTIONS")
        .unwrap_or("10".to_string())
        .parse()
        .unwrap_or(10);
    let manager = ConnectionManager::<DBType>::new(database_url);
    let pool = Pool::builder()
        .max_size(db_connections)
        .build(manager)
        .expect("Failed to create pool.");
    Ok(pool)
}

async fn init_sqlite_db_pool(
    database_url: &str,
) -> Result<Pool<ConnectionManager<DBType>>, String> {
    let manager = ConnectionManager::<DBType>::new(database_url);
    let pool = Pool::builder()
        .max_size(16)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(120)),
        }))
        .build(manager)
        .unwrap();
    Ok(pool)
}
