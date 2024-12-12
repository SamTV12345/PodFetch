use std::collections::HashSet;
use std::ops::DerefMut;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use actix_web::{App, Error, HttpServer};
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::{Condition, Logger};
use actix_web::web::{redirect, Data};
use clokwerk::{Scheduler, TimeUnits};
use diesel_migrations::MigrationHarness;
use jsonwebtoken::jwk::{AlgorithmParameters, CommonParameters, Jwk, KeyAlgorithm, RSAKeyParameters, RSAKeyType};
use log::info;
use tokio::spawn;
use tokio::task::JoinHandle;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::DBType;
use crate::{run_poll, POSTGRES_MIGRATIONS, SQLITE_MIGRATIONS};
use crate::adapters::api::api_routes::{check_server_config, get_gpodder_api, global_routes, insert_default_settings_if_not_present};
use crate::adapters::api::ws::server::ChatServer;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::oidc_model::{CustomJwk, CustomJwkSet};
use crate::models::session::Session;
use crate::models::settings::Setting;
use crate::service::environment_service::EnvironmentService;
use crate::service::file_service::FileService;
use crate::service::notification_service::NotificationService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::service::settings_service::SettingsService;
use crate::utils::reqwest_client::get_async_sync_client;

pub async fn run() -> (App<impl ServiceFactory<ServiceRequest,
    Response=ServiceResponse<EitherBody<_>>, Error=Error, Config=(), Service=_, InitError=(), Future=_> + Sized>, JoinHandle<std::io::Result<()>>) {
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

    check_server_config();

    //services
    let podcast_service = PodcastService::new();
    let file_service = FileService::new_db();
    let notification_service = NotificationService::new();
    let settings_service = SettingsService::new();

    let (chat_server, server_tx) = ChatServer::new();

    let chat_server = spawn(chat_server.run());

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
            let settings = Setting::get_settings().unwrap();
            match settings {
                Some(settings) => {
                    if settings.auto_update {
                        let podcast_service = PodcastService::new();
                        info!("Polling for new episodes");
                        match run_poll(podcast_service) {
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
                        PodcastEpisodeService::cleanup_old_episodes(
                            settings.auto_cleanup_days,
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
            let client = get_async_sync_client()
                .build()
                .unwrap();
            let resp = client
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

    if let Some(oidc_config) = ENVIRONMENT_SERVICE.get().unwrap().oidc_config.clone() {
        hash.insert(oidc_config.client_id);
    }

    let env_service = ENVIRONMENT_SERVICE.get().unwrap();
    let sub_dir = env_service.sub_directory.clone().unwrap_or("/".to_string());


        return (App::new()
            .app_data(Data::new(server_tx.clone()))
            .app_data(Data::new(key_param.clone()))
            .app_data(Data::new(jwk.clone()))
            .app_data(Data::new(hash.clone()))
            .service(redirect("/", sub_dir.clone() + "/ui/"))
            .service(get_gpodder_api())
            .service(global_routes())
            .app_data(Data::new(Mutex::new(podcast_service.clone())))
            .app_data(Data::new(Mutex::new(file_service.clone())))
            .app_data(Data::new(Mutex::new(notification_service.clone())))
            .app_data(Data::new(Mutex::new(settings_service.clone())))
            .wrap(Condition::new(cfg!(debug_assertions), Logger::default())), chat_server)
}