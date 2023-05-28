use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[macro_use]
extern crate serde_derive;
extern crate core;
extern crate serde_json;

use actix::{Actor};
use actix_files::{Files, NamedFile};
use actix_web::dev::{fn_service, ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::{Condition, Logger};
use actix_web::web::{redirect, Data};
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Scope};
use clokwerk::{Scheduler, TimeUnits};
use std::sync::{Mutex};
use std::time::Duration;
use std::{env, thread};
use std::env::{args, var};
use std::io::Read;
use std::process::exit;
use actix_web::body::{BoxBody, EitherBody};
use log::{info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use diesel::r2d2::{ConnectionManager};
use r2d2::{Pool};
use regex::Regex;
mod controllers;
use crate::config::dbconfig::{ConnectionOptions, establish_connection, get_database_url};
use crate::constants::constants::{BASIC_AUTH, OIDC_AUTH, TELEGRAM_API_ENABLED, TELEGRAM_BOT_CHAT_ID, TELEGRAM_BOT_TOKEN};
use crate::controllers::api_doc::ApiDoc;
use crate::controllers::notification_controller::{
    dismiss_notifications, get_unread_notifications,
};
use crate::controllers::podcast_controller::{add_podcast, add_podcast_by_feed, delete_podcast, find_all_podcasts, find_podcast, find_podcast_by_id, get_filter, proxy_podcast, refresh_all_podcasts, search_podcasts};
use crate::controllers::podcast_controller::{
    add_podcast_from_podindex, download_podcast, favorite_podcast, get_favored_podcasts,
    import_podcasts_from_opml, query_for_podcast, update_active_podcast,
};
use crate::controllers::podcast_episode_controller::{download_podcast_episodes_of_podcast, find_all_podcast_episodes_of_podcast, get_timeline};
use crate::controllers::settings_controller::{get_opml, get_settings, run_cleanup, update_name, update_settings};
use crate::controllers::sys_info_controller::{get_info, get_public_config, get_sys_info, login};
use crate::controllers::watch_time_controller::{get_last_watched, get_watchtime, log_watchtime};
use crate::controllers::websocket_controller::{
    get_rss_feed, get_rss_feed_for_podcast, start_connection,
};
pub use controllers::controller_utils::*;
use crate::auth_middleware::{AuthFilter};
use crate::command_line_runner::start_command_line;
use crate::controllers::user_controller::{create_invite, delete_invite, delete_user, get_invite, get_invite_link, get_invites, get_users, onboard_user, update_role};

mod constants;
mod db;
mod models;
mod service;
use crate::db::DB;
use crate::gpodder::parametrization::get_client_parametrization;
use crate::gpodder::routes::get_gpodder_api;
use crate::models::session::Session;
use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::file_service::FileService;
use crate::service::jwkservice::JWKService;
use crate::service::logging_service::init_logging;
use crate::service::mapping_service::MappingService;
use crate::service::notification_service::NotificationService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::service::settings_service::SettingsService;


mod config;

pub mod utils;
pub mod mutex;
mod exception;
mod gpodder;
mod command_line_runner;
mod auth_middleware;
mod dbconfig;

import_database_connections!();

type DbPool = Pool<ConnectionManager<DbConnection>>;

import_database_config!();

pub fn run_poll(
    mut podcast_service: PodcastService,
    mut podcast_episode_service: PodcastEpisodeService) {
    //check for new episodes
    let podcats_result = DB::get_all_podcasts(&mut establish_connection()).unwrap();
    for podcast in podcats_result {
        if podcast.active {
            let podcast_clone = podcast.clone();
            podcast_episode_service.insert_podcast_episodes(&mut establish_connection(), podcast);
            podcast_service.schedule_episode_download(podcast_clone, None, &mut establish_connection());
        }
    }
}

fn fix_links(content: &str)->String{
    let dir = var("SUB_DIRECTORY").unwrap()+"/ui/";
    content.replace("/ui/",&dir)
}

async fn index() -> impl Responder {
    let index_html = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/static/index.html"
    ));


    HttpResponse::Ok()
        .content_type("text/html")
        .body(fix_links(index_html))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Debug file located at {}", concat!(env!("OUT_DIR"), "/built.rs"));


    if args().len()>1 {
        start_command_line(args());
        exit(0)
    }

    let environment_service = EnvironmentService::new();

    check_server_config(environment_service.clone());
    let pool = init_db_pool(&get_database_url()).await.expect("Failed to connect to database");
    let data_pool = Data::new(pool);

    //services
    let podcast_episode_service = PodcastEpisodeService::new();
    let podcast_service = PodcastService::new();
    let db = DB::new().unwrap();
    let mapping_service = MappingService::new();
    let file_service = FileService::new_db();
    let notification_service = NotificationService::new();
    let settings_service = SettingsService::new();
    let lobby = Lobby::default();

    let chat_server = lobby.start();
    let mut connection = establish_connection();
    let res_migration = connection.run_pending_migrations(MIGRATIONS);

    if res_migration.is_err(){
        panic!("Could not run migrations: {}",res_migration.err().unwrap());
    }

    EnvironmentService::print_banner();
    init_logging();
    match FileService::create_podcast_root_directory_exists(){
        Ok(_)=>{},
        Err(e)=>{
            log::error!("Could not create podcast root directory: {}",e);
            panic!("Could not create podcast root directory: {}",e);
        }
    }

    insert_default_settings_if_not_present();

    thread::spawn(|| {
        let mut scheduler = Scheduler::new();
        let env = EnvironmentService::new();
        env.get_environment();
        let polling_interval = env.get_polling_interval();
        scheduler.every(polling_interval.minutes()).run(|| {
            let conn = &mut establish_connection();
            let settings = DB::new().unwrap().get_settings(conn);
            match settings {
                Some(settings) => {
                    if settings.auto_update {
                        let podcast_service = PodcastService::new();
                        let podcast_episode_service = PodcastEpisodeService::new();
                        info!("Polling for new episodes");
                        run_poll(podcast_service, podcast_episode_service);
                    }
                }
                None => {
                    log::error!("Could not get settings from database");
                }
            }
        });

        scheduler.every(1.day()).run(move || {
            // Clears the session ids once per day
            let conn= &mut establish_connection();
            Session::cleanup_sessions(conn).expect("Error clearing old \
            sessions");
            let mut db = DB::new().unwrap();
            let mut podcast_episode_service = PodcastEpisodeService::new();
            let settings = db.get_settings(conn);
            match settings {
                Some(settings) => {
                    if settings.auto_cleanup {
                        podcast_episode_service.cleanup_old_episodes(settings.auto_cleanup_days,
                                                                     &mut establish_connection());
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
    HttpServer::new(move || {
        App::new()
            .service(redirect("/", var("SUB_DIRECTORY").unwrap()+"/ui/"))
            .service(get_gpodder_api(environment_service.clone()))
            .service(get_global_scope())
            .app_data(Data::new(chat_server.clone()))
            .app_data(Data::new(Mutex::new(podcast_episode_service.clone())))
            .app_data(Data::new(Mutex::new(podcast_service.clone())))
            .app_data(Data::new(Mutex::new(db.clone())))
            .app_data(Data::new(Mutex::new(mapping_service.clone())))
            .app_data(Data::new(Mutex::new(file_service.clone())))
            .app_data(Data::new(Mutex::new(environment_service.clone())))
            .app_data(Data::new(Mutex::new(notification_service.clone())))
            .app_data(Data::new(Mutex::new(settings_service.clone())))
            .app_data(data_pool.clone())
            .app_data(Data::new(Mutex::new(JWKService::new())))
            .wrap(Condition::new(cfg!(debug_assertions),Logger::default()))
    })
        .workers(4)
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

pub fn get_api_config() -> Scope {
    web::scope("/api/v1")
        .configure(|cfg|{
            config(cfg)
        })
}


fn config(cfg: &mut web::ServiceConfig){
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
        .service(Files::new("/podcasts", "podcasts")
            .disable_content_disposition())
        .service(redirect("/swagger-ui", "/swagger-ui/"))
        .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi))
        .service(redirect("/", "./ui/"))
        .service(service)
        .service(start_connection)
        .service(get_rss_feed)
        .service(get_rss_feed_for_podcast)
}

fn get_private_api() -> Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse<EitherBody<BoxBody>>, Error = actix_web::Error, InitError = ()>> {
    let middleware = AuthFilter::new();
    web::scope("")
        .wrap(middleware)
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
}

pub fn config_secure_user_management(cfg: &mut web::ServiceConfig){
    if var(BASIC_AUTH).is_ok()||var(OIDC_AUTH).is_ok() {
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
            let rs =  test.captures(path).unwrap().get(1).unwrap().as_str();
            let file = NamedFile::open_async(format!("{}/{}",
                                                     "./static", rs)).await?;
            let mut content = String::new();

            let type_of = file.content_type().to_string();
            let res = file.file().read_to_string(&mut content);

            match res {
                Ok(_) => {},
                Err(_) => {
                    return Ok(ServiceResponse::new(req.clone(), file.into_response(&req)))
                }
            }
            if type_of.contains("css"){
                content  = fix_links(&content)
            }
            else if type_of.contains("javascript"){
                content = fix_links(&content)
            }
            let res = HttpResponse::Ok()
                .content_type(type_of)
                .body(content);
            Ok(ServiceResponse::new(req, res))}))

}

pub fn get_secure_user_management() ->Scope{
    web::scope("/users")
        .service(create_invite)
        .service(get_invites)
        .service(get_users)
        .service(update_role)
        .service(delete_user)
        .service(delete_invite)
        .service(get_invite_link)
}

pub fn insert_default_settings_if_not_present() {
    let mut db = DB::new().unwrap();
    let conn = &mut establish_connection();
    let settings = db.get_settings(conn);
    match settings {
        Some(_) => {
            info!("Settings already present");
        }
        None => {
            info!("No settings found, inserting default settings");
            db.insert_default_settings(conn);
        }
    }
}

pub fn check_server_config(service1: EnvironmentService) {
    let database_url = get_database_url();
    #[cfg(sqlite)]
    if !database_url.starts_with("sqlite"){
        eprintln!("You are using sqlite as database but the database url does not start with sqlite. \
        Please check your .env file.");
        exit(1);
    }

    #[cfg(mysql)]
    if !database_url.starts_with("mysql"){
        eprintln!("You are using mySQL as database but the database url does not start with  \
        sqlite. Please check your .env file.");
        exit(1);
    }

    #[cfg(postgresql)]
    if !database_url.starts_with("postgres"){
        eprintln!("You are using postgres as database but the database url does not start with  \
        sqlite. Please check your .env file.");
        exit(1);
    }

    if service1.http_basic {
        if service1.password.is_empty() || service1.username.is_empty() {
            log::error!("BASIC_AUTH activated but no username or password set. Please set username and password in the .env file.");
            exit(1);
        }
    }

    if service1.gpodder_integration_enabled{
        if !(service1.http_basic || service1.oidc_configured){
            eprintln!("GPODDER_INTEGRATION_ENABLED activated but no BASIC_AUTH or OIDC_AUTH set. Please set BASIC_AUTH or OIDC_AUTH in the .env file.");
            exit(1);
        }
    }

    if service1.http_basic && service1.oidc_configured{
        log::error!("You cannot have oidc and basic auth enabled at the same time. Please disable one of them.");
    }

    if var(TELEGRAM_API_ENABLED).is_ok(){
        if !var(TELEGRAM_BOT_TOKEN).is_ok() || !var(TELEGRAM_BOT_CHAT_ID).is_ok() {
            eprintln!("TELEGRAM_API_ENABLED activated but no TELEGRAM_API_TOKEN or TELEGRAM_API_CHAT_ID set. Please set TELEGRAM_API_TOKEN and TELEGRAM_API_CHAT_ID in the .env file.");
            exit(1);
        }
    }
}

#[cfg(sqlite)]
async fn init_db_pool(database_url: &str)-> Result<Pool<ConnectionManager<SqliteConnection>>,
    String> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder().max_size(16)
        .connection_customizer(Box::new(ConnectionOptions {
        enable_wal: true,
        enable_foreign_keys: true,
        busy_timeout: Some(Duration::from_secs(120)),
    })).build(manager).unwrap();
    Ok(pool)
}


#[cfg(postgresql)]
async fn init_db_pool(database_url: &str)-> Result<Pool<ConnectionManager<DbConnection>>,
    String> {
    let db_connections = var("DB_CONNECTIONS").unwrap_or("10".to_string()).parse()
        .unwrap_or(10);
    let manager = ConnectionManager::<DbConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(db_connections)
        .build(manager)
        .expect("Failed to create pool.");
    Ok(pool)
}

#[cfg(mysql)]
async fn init_db_pool(database_url: &str)-> Result<Pool<ConnectionManager<DbConnection>>,
    String> {
    let manager = ConnectionManager::<DbConnection>::new(database_url);
    let pool = Pool::builder().max_size(16)
        .build(manager).unwrap();
    Ok(pool)
}