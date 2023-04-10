use base64::{engine::general_purpose, Engine};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[macro_use]
extern crate serde_derive;
extern crate core;
extern crate serde_json;

use actix_web_httpauth::middleware::HttpAuthentication;
use actix::Actor;
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::{fn_service, ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::error::ErrorUnauthorized;
use actix_web::middleware::{Condition, Logger};
use actix_web::web::{redirect, Data};
use actix_web::{http, web, App, Error, HttpResponse, HttpServer, Responder, Scope};
use actix_web_httpauth::extractors::basic::BasicAuth;
use clokwerk::{Scheduler, TimeUnits};
use std::sync::{Mutex};
use std::time::Duration;
use std::{env, thread};
use std::env::var;
use std::io::Read;
use std::str::FromStr;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{Algorithm, decode, DecodingKey, Validation};
use jsonwebtoken::jwk::{Jwk};
use log::{info};
use serde_json::{from_str, Value};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use std::time::{SystemTime, UNIX_EPOCH};
use actix_web::http::header::HeaderValue;
use diesel::r2d2::ConnectionManager;
use diesel::SqliteConnection;
use r2d2::Pool;
use regex::Regex;
use reqwest::header::HeaderName;

pub mod schema;
mod controllers;
use crate::config::dbconfig::{ConnectionOptions, establish_connection, get_database_url};
use crate::constants::constants::{BASIC_AUTH, ERROR_LOGIN_MESSAGE, OIDC_AUTH, TELEGRAM_API_ENABLED, TELEGRAM_BOT_CHAT_ID, TELEGRAM_BOT_TOKEN, USERNAME};
use crate::controllers::api_doc::ApiDoc;
use crate::controllers::notification_controller::{
    dismiss_notifications, get_unread_notifications,
};
use crate::controllers::podcast_controller::{
    add_podcast, find_all_podcasts, find_podcast, find_podcast_by_id,
};
use crate::controllers::podcast_controller::{
    add_podcast_from_podindex, download_podcast, favorite_podcast, get_favored_podcasts,
    import_podcasts_from_opml, query_for_podcast, update_active_podcast,
};
use crate::controllers::podcast_episode_controller::{
    download_podcast_episodes_of_podcast, find_all_podcast_episodes_of_podcast,
};
use crate::controllers::settings_controller::{get_settings, run_cleanup, update_settings};
use crate::controllers::sys_info_controller::{get_public_config, get_sys_info, login};
use crate::controllers::watch_time_controller::{get_last_watched, get_watchtime, log_watchtime};
use crate::controllers::websocket_controller::{
    get_rss_feed, get_rss_feed_for_podcast, start_connection,
};
pub use controllers::controller_utils::*;
use crate::controllers::user_controller::{create_invite, get_invite, get_invites, get_users, onboard_user, update_role};

mod constants;
mod db;
mod models;
mod service;
use crate::db::DB;
use crate::models::oidc_model::{CustomJwk, CustomJwkSet};
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

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

async fn validator(
    mut req: ServiceRequest,
    _credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let authorization = req.headers().get("Authorization").unwrap().to_str();

    match authorization {
        Ok(auth) => {
            let auth = auth.to_string();
            let auth = auth.split(" ").collect::<Vec<&str>>();
            let auth = auth[1];
            let auth = general_purpose::STANDARD.decode(auth).unwrap();
            let auth = String::from_utf8(auth).unwrap();
            let auth = auth.split(":").collect::<Vec<&str>>();
            let username = auth[0];
            let password = auth[1];
            let env = EnvironmentService::new();
            if username == env.username && password == env.password {
                req.headers_mut().append(HeaderName::from_str(USERNAME).unwrap(),
                                     HeaderValue::from_str(username).unwrap());
                return Ok(req);
            }
        }
        Err(_) => {
            return Err((ErrorUnauthorized(ERROR_LOGIN_MESSAGE), req));
        }
    }
    return Err((ErrorUnauthorized(ERROR_LOGIN_MESSAGE), req));
}

async fn validate_oidc_token(rq: ServiceRequest, bearer: BearerAuth, mut jwk_service: JWKService)
                             ->Result<ServiceRequest,
    (Error, ServiceRequest)> {
    // Check if the Authorization header exists and has a Bearer token
    let token = bearer.token();


    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_secs();

    let response;
    match jwk_service.jwk {
        Some(jwk)=>{
            if since_the_epoch-jwk_service.timestamp>3600{
                //refetch and update timestamp
                info!("Renewing jwk set");
                response = get_jwk().await;
                jwk_service.timestamp = since_the_epoch
            }
            else{
                info!("Using cached jwk set");
                response = jwk;
            }
        }
        None=>{
            // Fetch on cold start
            response = get_jwk().await;
            jwk_service.jwk = Some(response.clone());
        }
    }

    // Filter out all unknown algorithms
    let response = response.clone().keys.into_iter().filter(|x| {
        x.alg.eq(&"RS256")
    }).collect::<Vec<CustomJwk>>();

    let jwk = response.clone();
    let custom_jwk = jwk.get(0).expect("Your jwk set needs to have RS256");

    let jwk_string = serde_json::to_string(&custom_jwk).unwrap();

    let jwk = from_str::<Jwk>(&*jwk_string).unwrap();
    let key = DecodingKey::from_jwk(&jwk).unwrap();
    let validation = Validation::new(Algorithm::RS256);
    match decode::<Value>(&token, &key, &validation) {
        Ok(_) => Ok(rq),
        Err(e) =>{
            info!("Error: {:?}",e);
            Err((ErrorUnauthorized("Invalid oidc token."), rq))
        }
    }
}

async fn get_jwk() -> CustomJwkSet {
    let jwk_uri = var("OIDC_JWKS").expect("OIDC_JWKS must be set");
    let response = reqwest::get(jwk_uri).await.unwrap()
        .json::<CustomJwkSet>()
        .await.unwrap();
    response
}

pub fn run_poll(
    mut podcast_service: PodcastService,
    mut podcast_episode_service: PodcastEpisodeService,
) {
    //check for new episodes
    let podcats_result = DB::get_podcasts(&mut establish_connection()).unwrap();
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

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //services
    let podcast_episode_service = PodcastEpisodeService::new();
    let podcast_service = PodcastService::new();
    let db = DB::new().unwrap();
    let mapping_service = MappingService::new();
    let file_service = FileService::new();
    let environment_service = EnvironmentService::new();
    let notification_service = NotificationService::new();
    let settings_service = SettingsService::new();


    let lobby = Lobby::default();
    let pool = init_db_pool(&get_database_url()).await.expect("Failed to connect to database");

    let chat_server = lobby.start();

    let mut connection = establish_connection();
    connection.run_pending_migrations(MIGRATIONS).unwrap();

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
    check_server_config(environment_service.clone());

    thread::spawn(|| {
        let mut scheduler = Scheduler::new();
        let env = EnvironmentService::new();
        env.get_environment();
        let polling_interval = env.get_polling_interval();
        scheduler.every(polling_interval.minutes()).run(|| {
            let settings = DB::new().unwrap().get_settings();
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

        scheduler.every(1.day()).run(|| {
            let mut db = DB::new().unwrap();
            let mut podcast_episode_service = PodcastEpisodeService::new();
            let settings = db.get_settings();
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
            .app_data(Data::new(pool.clone()))
            .wrap(Condition::new(true,Logger::default()))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

pub fn get_api_config() -> Scope {
    web::scope("/api/v1")
        .configure(config)
}


fn config(cfg: &mut web::ServiceConfig){
    cfg.service(get_invite)
        .service(onboard_user)
        .service(login)
        .service(get_public_config)
        .service(get_private_api());
}

pub fn get_global_scope()->Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse<EitherBody<EitherBody<BoxBody>>>, Error = Error, InitError = ()>>{
    let base_path = var("SUB_DIRECTORY").unwrap_or("/".to_string());
    let openapi = ApiDoc::openapi();
    let service = get_api_config();

    let dev_enabled = var("DEV").is_ok();

    web::scope(&base_path)
        .service(Files::new("/podcasts", "podcasts"))
        .service(redirect("/swagger-ui", "/swagger-ui/"))
        .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi))
        .wrap(Condition::new(dev_enabled, get_cors_config()))
        .service(redirect("/", "./ui/"))
        .service(service)
        .service(get_ui_config())
        .service(start_connection)
        .service(get_rss_feed)
        .service(get_rss_feed_for_podcast)
}

fn get_private_api() -> Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse<EitherBody<EitherBody<EitherBody<EitherBody<BoxBody>>>, EitherBody<EitherBody<BoxBody>>>>, Error = Error, InitError = ()>> {
    let enable_basic_auth = var(BASIC_AUTH).is_ok();
    let auth = HttpAuthentication::basic(validator);
    let enable_oidc_auth = var(OIDC_AUTH).is_ok();
    let jwk_service = JWKService{
        timestamp:0,
        jwk:None
    };

    let oidc_auth = HttpAuthentication::bearer(move |srv,req|{
        validate_oidc_token(srv,req, jwk_service.clone())
    });

    web::scope("")
        .wrap(Condition::new(enable_basic_auth, auth))
        .wrap(Condition::new(enable_oidc_auth, oidc_auth))
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
        .service(get_watchtime)
        .service(get_settings)
        .service(update_settings)
        .service(update_active_podcast)
        .service(import_podcasts_from_opml)
        .service(run_cleanup)
        .service(add_podcast_from_podindex)
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

pub fn get_cors_config() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .allowed_header(http::header::CONNECTION)
        .allowed_header(http::header::UPGRADE)
        .max_age(3600)
}

pub fn get_public_user_management() ->Scope{
    web::scope("")
        .service(onboard_user)
        .service(get_invite)
}

pub fn get_secure_user_management() ->Scope{
    web::scope("/users")
        .service(create_invite)
        .service(get_invites)
        .service(onboard_user)
        .service(get_users)
        .service(update_role)
}

pub fn insert_default_settings_if_not_present() {
    let mut db = DB::new().unwrap();
    let settings = db.get_settings();
    match settings {
        Some(_) => {
            info!("Settings already present");
        }
        None => {
            info!("No settings found, inserting default settings");
            db.insert_default_settings();
        }
    }
}

pub fn check_server_config(service1: EnvironmentService) {
    if service1.http_basic {
        if service1.password.is_empty() || service1.username.is_empty() {
            log::error!("BASIC_AUTH activated but no username or password set. Please set username and password in the .env file.");
            std::process::exit(1);
        }
    }

    if service1.http_basic && service1.oidc_configured{
        log::error!("You cannot have oidc and basic auth enabled at the same time. Please disable one of them.");
    }

    if var(TELEGRAM_API_ENABLED).is_ok(){
        if !var(TELEGRAM_BOT_TOKEN).is_ok() || !var(TELEGRAM_BOT_CHAT_ID).is_ok() {
            log::error!("TELEGRAM_API_ENABLED activated but no TELEGRAM_API_TOKEN or TELEGRAM_API_CHAT_ID set. Please set TELEGRAM_API_TOKEN and TELEGRAM_API_CHAT_ID in the .env file.");
            std::process::exit(1);
        }
    }
}

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