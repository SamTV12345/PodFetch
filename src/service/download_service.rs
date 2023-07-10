use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::service::file_service::{FileService};
use crate::service::mapping_service::MappingService;

use crate::service::podcast_episode_service::PodcastEpisodeService;
use reqwest::blocking::ClientBuilder;

use std::io;
use reqwest::header::HeaderMap;

use crate::config::dbconfig::establish_connection;
use crate::constants::constants::{PODCAST_FILENAME, PODCAST_IMAGENAME};
use crate::models::file_path::FilenameBuilder;
use crate::service::settings_service::SettingsService;
use crate::utils::append_to_header::add_basic_auth_headers_conditionally;
use crate::utils::error::CustomError;

pub struct DownloadService {
    pub mappingservice: MappingService,
    pub client_builder: ClientBuilder,
    pub file_service: FileService,
}

impl DownloadService {
    pub fn new() -> Self {
        DownloadService {
            mappingservice: MappingService::new(),
            client_builder: ClientBuilder::new(),
            file_service: FileService::new(),
        }
    }

    pub fn download_podcast_episode(&mut self, podcast_episode: PodcastEpisode, podcast: Podcast)
        -> Result<(),CustomError> {
        let conn = &mut establish_connection();
        let suffix = PodcastEpisodeService::get_url_file_suffix(&podcast_episode.url);
        let settings_in_db = SettingsService::new().get_settings(conn)?.unwrap();
        let image_suffix = PodcastEpisodeService::get_url_file_suffix(&podcast_episode.image_url);

        let paths;
        match settings_in_db.use_existing_filename {
            true=>{
                paths = FilenameBuilder::default()
                    .with_podcast(podcast.clone())
                    .with_suffix(&suffix)
                    .with_episode(podcast_episode.clone(), conn)?
                    .with_filename(PODCAST_FILENAME)
                    .with_image_filename(PODCAST_IMAGENAME)
                    .with_image_suffix(&image_suffix)
                    .with_raw_directory(conn)?
                    .build(conn)?;
            },
            false=>{
                paths = FilenameBuilder::default()
                    .with_suffix(&suffix)
                    .with_image_suffix(&image_suffix)
                    .with_episode(podcast_episode.clone(), conn)?
                    .with_podcast_directory(&podcast.directory_name)
                    .with_podcast(podcast.clone())
                    .with_image_filename(PODCAST_IMAGENAME)
                    .with_filename(PODCAST_FILENAME)
                    .build(conn)?;
            }
        }


        let client = ClientBuilder::new()
            .build()
            .unwrap();

        let mut header_map = HeaderMap::new();
        add_basic_auth_headers_conditionally(podcast_episode.clone().url, &mut header_map);
        let mut resp = client.get(podcast_episode.clone().url)
            .headers(header_map.clone())
            .send()
            .unwrap();
        let mut image_response = client.get(podcast_episode.image_url.clone()).headers(header_map).send()
            .unwrap();

        let mut podcast_out = std::fs::File::create(paths.0.clone()).unwrap();
        let mut image_out = std::fs::File::create(paths.1.clone()).unwrap();

        if !self.file_service.check_if_podcast_main_image_downloaded(&podcast.clone()
            .directory_id,  conn)
        {
            let mut image_podcast = std::fs::File::create(paths.1.clone()).unwrap();
            io::copy(&mut image_response, &mut image_podcast).expect("failed to copy content");
        }

        io::copy(&mut resp, &mut podcast_out).expect("failed to copy content");

        PodcastEpisode::update_total_podcast_time_and_image(
                &podcast_episode.episode_id,
                &paths.1.clone(),
                &paths.0.clone(),
            conn)
            .expect("TODO: panic message");
        io::copy(&mut image_response, &mut image_out).expect("failed to copy content");
        Ok(())
    }
}
