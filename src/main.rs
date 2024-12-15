use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[macro_use]
extern crate serde_derive;
extern crate core;
extern crate serde_json;

use actix_files::{Files, NamedFile};
use actix_web::dev::{fn_service, ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::{Condition, Logger};
use actix_web::web::{redirect, Data};
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Scope};
use clokwerk::{Scheduler, TimeUnits};
use std::collections::HashSet;
use std::env::args;
use std::io::Read;
use std::sync::Mutex;
use std::time::Duration;
use std::{env, thread};
use std::ops::DerefMut;
use actix_web::body::{BoxBody, EitherBody};
use diesel::r2d2::ConnectionManager;
use jsonwebtoken::jwk::{
    AlgorithmParameters, CommonParameters, Jwk, KeyAlgorithm, RSAKeyParameters, RSAKeyType,
};
use log::info;
use r2d2::Pool;
use regex::Regex;
use std::process::exit;
use tokio::{spawn, try_join};

use crate::auth_middleware::AuthFilter;
use crate::command_line_runner::start_command_line;
use crate::constants::inner_constants::{CSS, ENVIRONMENT_SERVICE, JS};
use crate::adapters::api::controllers::podcast_controller::{add_podcast, add_podcast_by_feed,
delete_podcast,
find_all_podcasts, find_podcast, find_podcast_by_id, get_filter, get_podcast_settings, refresh_all_podcasts, retrieve_podcast_sample_format, search_podcasts, update_podcast_settings};
use crate::adapters::api::controllers::podcast_controller::{
    add_podcast_from_podindex, download_podcast, favorite_podcast, get_favored_podcasts,
    import_podcasts_from_opml, query_for_podcast, update_active_podcast,
};
use crate::adapters::api::run::run;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::DBType;
use crate::domain::models::podcast::podcast::Podcast;

mod constants;
mod models;
mod service;


use crate::service::environment_service::EnvironmentService;
use crate::service::file_service::FileService;
use crate::service::logging_service::init_logging;

use crate::service::notification_service::NotificationService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;
use crate::service::settings_service::SettingsService;
use crate::utils::error::CustomError;
use crate::utils::podcast_key_checker::check_podcast_request;
use crate::utils::reqwest_client::get_async_sync_client;

mod config;

mod auth_middleware;
mod command_line_runner;
mod exception;
mod gpodder;
pub mod mutex;
pub mod utils;
mod adapters;
mod domain;
mod application;

type DbPool = Pool<ConnectionManager<DBType>>;

import_database_config!();

pub fn run_poll(mut podcast_service: PodcastService) -> Result<(), CustomError> {
    //check for new episodes
    let podcats_result = PodcastService::get_all_podcasts()?;
    for podcast in podcats_result {
        if podcast.active {
            let podcast_clone = podcast.clone();
            PodcastEpisodeService::insert_podcast_episodes(podcast)?;
            podcast_service.schedule_episode_download(
                podcast_clone,
                None,
            )?;
        }
    }
    Ok(())
}

fn fix_links(content: &str) -> String {
    let env_service = ENVIRONMENT_SERVICE.get().unwrap();
    let dir = env_service.sub_directory.clone().unwrap() + "/ui/";
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
    let env_service = EnvironmentService::new();

    println!(
        "Debug file located at {}",
        concat!(env!("OUT_DIR"), "/built.rs")
    );
    init_logging();
    ENVIRONMENT_SERVICE.get_or_init(|| env_service);

    if args().len() > 1 {
        start_command_line(args()).await;
        exit(0)
    }

    let (app, chat_server) = run().await;
    let http_server = HttpServer::new(||{
       return  app
    }).workers(4)
        .bind(("0.0.0.0", 8000))?
        .run();
    try_join!(http_server, async move { chat_server.await.unwrap() })?;
    Ok(())
}





