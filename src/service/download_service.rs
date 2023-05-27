use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::file_service::{FileService};
use crate::service::mapping_service::MappingService;

use crate::service::podcast_episode_service::PodcastEpisodeService;
use reqwest::blocking::ClientBuilder;

use std::io;

use crate::config::dbconfig::establish_connection;
use crate::constants::constants::{PODCAST_FILENAME, PODCAST_IMAGENAME};
use crate::DbConnection;
use crate::models::file_path::FilenameBuilder;
use crate::service::settings_service::SettingsService;

pub struct DownloadService {
    pub db: DB,
    pub mappingservice: MappingService,
    pub client_builder: ClientBuilder,
    pub file_service: FileService,
}

impl DownloadService {
    pub fn new() -> Self {
        DownloadService {
            db: DB::new().unwrap(),
            mappingservice: MappingService::new(),
            client_builder: ClientBuilder::new(),
            file_service: FileService::new(),
        }
    }

    pub fn download_podcast_episode(&mut self, podcast_episode: PodcastEpisode, podcast: Podcast,
                                    db:DB, _conn: &mut DbConnection) {
        let conn = &mut establish_connection();
        let suffix = PodcastEpisodeService::get_url_file_suffix(&podcast_episode.url);
        let settings_in_db = SettingsService::new().get_settings(db.clone(),conn).unwrap();
        let image_suffix = PodcastEpisodeService::get_url_file_suffix(&podcast_episode.image_url);

        let paths;
        match settings_in_db.use_existing_filename {
            true=>{
                paths = FilenameBuilder::default()
                    .with_podcast(podcast.clone())
                    .with_suffix(&suffix)
                    .with_episode(podcast_episode.clone(), conn)
                    .with_filename(PODCAST_FILENAME)
                    .with_image_filename(PODCAST_IMAGENAME)
                    .with_image_suffix(&image_suffix)
                    .with_raw_directory(conn)
                    .build(conn);
            },
            false=>{
                paths = FilenameBuilder::default()
                    .with_suffix(&suffix)
                    .with_image_suffix(&image_suffix)
                    .with_episode(podcast_episode.clone(), conn)
                    .with_podcast_directory(&podcast.directory_name)
                    .with_podcast(podcast.clone())
                    .with_image_filename(PODCAST_IMAGENAME)
                    .with_filename(PODCAST_FILENAME)
                    .build(conn);
            }
        }


        let client = ClientBuilder::new().build().unwrap();
        let mut resp = client.get(podcast_episode.clone().url).send().unwrap();
        let mut image_response = client.get(podcast_episode.image_url.clone()).send().unwrap();

        let mut podcast_out = std::fs::File::create(paths.0.clone()).unwrap();
        let mut image_out = std::fs::File::create(paths.1.clone()).unwrap();

        if !self.file_service.check_if_podcast_main_image_downloaded(&podcast.clone()
            .directory_id, db.clone(), conn)
        {
            let mut image_podcast = std::fs::File::create(paths.1.clone()).unwrap();
            io::copy(&mut image_response, &mut image_podcast).expect("failed to copy content");
        }

        io::copy(&mut resp, &mut podcast_out).expect("failed to copy content");

        self.db.update_total_podcast_time_and_image(
                &podcast_episode.episode_id,
                &paths.1.clone(),
                &paths.0.clone(),
            conn)
            .expect("TODO: panic message");
        io::copy(&mut image_response, &mut image_out).expect("failed to copy content");
    }
}
