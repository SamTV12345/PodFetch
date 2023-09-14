use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::service::file_service::{FileService};
use crate::service::mapping_service::MappingService;

use reqwest::blocking::ClientBuilder;

use std::io;
use reqwest::header::HeaderMap;

use crate::config::dbconfig::establish_connection;
use crate::constants::inner_constants::{PODCAST_FILENAME, PODCAST_IMAGENAME};
use crate::models::file_path::FilenameBuilder;
use crate::models::settings::Setting;
use crate::utils::append_to_header::add_basic_auth_headers_conditionally;
use crate::utils::error::CustomError;
use crate::utils::file_extension_determination::{determine_file_extension, FileType};

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

        let client = ClientBuilder::new()
            .build()
            .unwrap();

        let conn = &mut establish_connection();
        let suffix = determine_file_extension(&podcast_episode.url, &client, FileType::Audio);
        let settings_in_db = Setting::get_settings(conn)?.unwrap();
        let image_suffix = determine_file_extension(&podcast_episode.image_url, &client,
                                                    FileType::Image);


        let mut header_map = HeaderMap::new();
        add_basic_auth_headers_conditionally(podcast_episode.clone().url, &mut header_map);
        let mut resp = client.get(podcast_episode.clone().url)
            .headers(header_map.clone())
            .send()
            .unwrap();

        let mut image_response = client.get(podcast_episode.image_url.clone()).headers(header_map).send()
            .unwrap();

        //TODO Continue here with the direct path and also add test cases
        let paths = match settings_in_db.use_existing_filename {
            true=>{
                FilenameBuilder::default()
                    .with_podcast(podcast.clone())
                    .with_suffix(&suffix)
                    .with_settings(settings_in_db)
                    .with_episode(podcast_episode.clone(), conn)?
                    .with_filename(PODCAST_FILENAME)
                    .with_image_filename(PODCAST_IMAGENAME)
                    .with_image_suffix(&image_suffix)
                    .with_raw_directory(conn)?
                    .build(conn)?
            },
            false=>{
                 FilenameBuilder::default()
                    .with_suffix(&suffix)
                     .with_settings(settings_in_db)
                    .with_image_suffix(&image_suffix)
                    .with_episode(podcast_episode.clone(), conn)?
                    .with_podcast_directory(&podcast.directory_name)
                    .with_podcast(podcast.clone())
                    .with_image_filename(PODCAST_IMAGENAME)
                    .with_filename(PODCAST_FILENAME)
                    .build(conn)?
            }
        };


        let mut podcast_out = std::fs::File::create(&paths.filename).unwrap();
        let mut image_out = std::fs::File::create(&paths.image_filename).unwrap();

        if !self.file_service.check_if_podcast_main_image_downloaded(&podcast.clone()
            .directory_id,  conn)
        {
            let mut image_podcast = std::fs::File::create(&paths.image_filename).unwrap();
            io::copy(&mut image_response, &mut image_podcast).expect("failed to copy content");
        }

        io::copy(&mut resp, &mut podcast_out).expect("failed to copy content");

        PodcastEpisode::update_local_paths(
                &podcast_episode.episode_id,
                &paths.local_image_url,
                &paths.local_file_url,
                &paths.image_filename,
                &paths.filename,
            conn)?;
        io::copy(&mut image_response, &mut image_out).expect("failed to copy content");
        Ok(())
    }
}
