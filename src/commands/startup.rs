use std::ops::DerefMut;
use std::process::exit;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use axum::body::Body;
use axum::extract::Request;
use axum::middleware::from_fn;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Router;
use axum::routing::get;
use clokwerk::{Scheduler, TimeUnits};
use diesel::r2d2::ConnectionManager;
use diesel_migrations::MigrationHarness;
use log::info;
use maud::{html, Markup};
use r2d2::Pool;
use socketioxide::extract::SocketRef;
use socketioxide::SocketIoBuilder;
use tokio::fs;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar};
use utoipa_swagger_ui::SwaggerUi;
use crate::constants::inner_constants::{CSS, ENVIRONMENT_SERVICE, JS, MAIN_ROOM};
use crate::adapters::api::controllers::routes::global_routes;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::DBType;
use crate::auth_middleware::{handle_basic_auth, handle_no_auth, handle_oidc_auth, handle_proxy_auth};
use crate::controllers::file_hosting::podcast_serving;
use crate::controllers::manifest_controller::get_manifest_router;
use crate::controllers::notification_controller::get_notification_router;
use crate::controllers::playlist_controller::get_playlist_router;
use crate::controllers::podcast_controller::get_podcast_router;
use crate::controllers::podcast_episode_controller::get_podcast_episode_router;
use crate::controllers::server::SOCKET_IO_LAYER;
use crate::controllers::settings_controller::get_settings_router;
use crate::controllers::sys_info_controller::{get_public_config, get_sys_info_router, login};
use crate::controllers::tags_controller::get_tags_router;
use crate::controllers::user_controller::{get_invite, get_user_router, onboard_user};
use crate::controllers::watch_time_controller::get_watchtime_router;
use crate::controllers::websocket_controller::get_websocket_router;
use crate::import_database_config;
use crate::models::podcasts::Podcast;
use crate::models::session::Session;
use crate::models::settings::Setting;
use crate::service::file_service::FileService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::utils::error::{CustomError, CustomErrorInner};
use utoipa_scalar::Servable as UtoipaServable;

pub type DbPool = Pool<ConnectionManager<DBType>>;
use crate::embed_migrations;
use crate::EmbeddedMigrations;

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

#[
utoipa::path(
get,
path="/index.html",
)
]
async fn index() -> Result<Response<String>, CustomError> {
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

    let response = Response::builder()
        .header("Content-Type", "text/html")
        .status(200)
        .body(html.0.to_string())
        .unwrap();
    Ok(response)
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


pub fn get_ui_config() -> OpenApiRouter {
    OpenApiRouter::new()
        .nest("/ui/", OpenApiRouter::new()
        .routes(routes!(index))
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



async fn handle_ui_access(req: Request) -> Result<impl IntoResponse, CustomError> {
    let (req_parts, _) = req.into_parts();
    let path = req_parts.uri.path();

    if path.contains("..") {
        return Err(CustomErrorInner::NotFound.into());
    }
    let mut file_path = format!("{}{}", "./static", path);

    let mut content = match fs::read(&file_path).await {
        Ok(e)=>Ok::<Vec<u8>, CustomError>(e),
        Err(_)=>{
            file_path = "./static/index.html".to_string();
            Ok(fs::read("./static/index.html").await.unwrap())
        }
    }?;
    let content_type = mime_guess::from_path(&file_path).first_or_octet_stream();

    if content_type.to_string().contains(CSS) || content_type.to_string().contains(JS) {
        let str_content = String::from_utf8(content).unwrap();
        content = fix_links(&str_content).as_bytes().to_vec();
    }

    //let res = Response::builder().header("Content-Type", content_type.to_string()).body(content)
    //    .unwrap();
    let res = Response::builder()
        .header("Content-Type", content_type.to_string())
        .body(Body::from(content))
        .unwrap();
    Ok(res)
}


pub fn get_api_config() -> OpenApiRouter {
    OpenApiRouter::new()
        .merge(get_ui_config())
        .merge(podcast_serving())
        .merge(get_manifest_router())
        .merge(get_websocket_router())
        .nest("/api/v1", OpenApiRouter::new().merge(config()))

}

fn config() -> OpenApiRouter {
    use crate::controllers::user_controller::__path_get_invite;
    use crate::controllers::user_controller::__path_onboard_user;
    use crate::controllers::sys_info_controller::__path_get_public_config;
    use crate::controllers::sys_info_controller::__path_login;
    OpenApiRouter::new()
        .routes(routes!(get_invite))
        .routes(routes!(onboard_user))
        .routes(routes!(get_public_config))
        .routes(routes!(login))
        .merge(get_private_api())
}

fn get_private_api() -> OpenApiRouter {
    let router = OpenApiRouter::new()
        .merge(get_playlist_router())
        .merge(get_podcast_router())
        .merge(get_sys_info_router())
        .merge(get_watchtime_router())
        .merge(get_notification_router())
        .merge(get_podcast_episode_router())
        .merge(get_settings_router())
        .merge(get_tags_router())
        .merge(get_user_router());

    if ENVIRONMENT_SERVICE.http_basic {
        router.layer(from_fn(handle_basic_auth))
    } else if ENVIRONMENT_SERVICE.oidc_configured {
        router.layer(from_fn(handle_oidc_auth))
    } else if ENVIRONMENT_SERVICE.reverse_proxy {
        router.layer(from_fn(handle_proxy_auth))
    } else {
        router.layer(from_fn(handle_no_auth))
    }
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


pub fn handle_config_for_server_startup() -> Router {
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

    let sub_dir = ENVIRONMENT_SERVICE
        .sub_directory
        .clone()
        .unwrap_or("/".to_string());

    let ui_dir = format!("{}/ui/", sub_dir);
    let (layer, io) = SocketIoBuilder::new().build_layer();
    io.ns("/", |socket: SocketRef|{
        log::info!("Socket connected {}", socket.id);
    });
    io.ns("/".to_owned() +MAIN_ROOM, ||{
        info!("Socket connected to main room");
    });
    SOCKET_IO_LAYER.get_or_init(|| io);

    let (router, api) = OpenApiRouter::new()
        .merge(global_routes())
        .route("/", get(Redirect::to(&ui_dir)))
        .split_for_parts();
    router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
        .merge(Redoc::with_url("/redoc", api.clone()))
        // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
        // via SwaggerUi instead we only make rapidoc to point to the existing doc.
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        // Alternative to above
        // .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", api).path("/rapidoc"))
        .merge(Scalar::with_url("/scalar", api))
        .layer(layer)
}


#[cfg(test)]
pub mod tests {
    use axum_test::TestServer;
    use crate::commands::startup::handle_config_for_server_startup;

    pub fn handle_test_startup() -> TestServer {

        let mut test_server = TestServer::new(handle_config_for_server_startup()).unwrap();
        test_server.add_header("Authorization", "Basic cG9zdGdyZXM6cG9zdGdyZXM=");
        test_server
    }
}