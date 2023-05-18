use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::file_service::{determine_image_and_local_podcast_audio_url, FileService, prepare_podcast_episode_title_to_directory};
use crate::service::mapping_service::MappingService;
use crate::service::path_service::PathService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use reqwest::blocking::ClientBuilder;
use std::fs::create_dir;
use std::io;
use crate::models::file_path::FooBuilder;
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

    pub fn download_podcast_episode(&mut self, podcast_episode: PodcastEpisode, podcast: Podcast) {
        let suffix = PodcastEpisodeService::get_url_file_suffix(&podcast_episode.url);
        let settings_in_db = SettingsService::new().get_settings().unwrap();
        let image_suffix = PodcastEpisodeService::get_url_file_suffix(&podcast_episode.image_url);

        let image_podcast_path;
        let podcast_episode_path ;
        match settings_in_db.use_existing_filename {
            true=>{
                podcast_episode_path = FooBuilder::default()
                    .with_podcast(podcast.clone())
                    .with_suffix(&suffix)
                    .with_episode(podcast_episode.clone())
                    .with_filename("podcast")
                    .with_raw_directory()
                    .build();
                image_podcast_path = FooBuilder::default()
                    .with_podcast(podcast.clone())
                    .with_suffix(&image_suffix)
                    .with_episode(podcast_episode.clone())
                    .with_filename("image")
                    .with_raw_directory()
                    .build();
            },
            false=>{
                podcast_episode_path = FooBuilder::default()
                    .with_suffix(&suffix)
                    .with_episode(podcast_episode.clone())
                    .with_podcast_directory(&podcast.directory_name)
                    .with_podcast(podcast.clone())
                    .with_filename("podcast")
                    .build();

                image_podcast_path = FooBuilder::default()
                    .with_suffix(&image_suffix)
                    .with_episode(podcast_episode.clone())
                    .with_podcast_directory(&podcast.directory_name)
                    .with_podcast(podcast.clone())
                    .with_filename("image")
                    .build();
            }
        }


        let client = ClientBuilder::new().build().unwrap();
        let mut resp = client.get(podcast_episode.clone().url).send().unwrap();
        let mut image_response = client.get(podcast_episode.image_url.clone()).send().unwrap();

        let mut podcast_out = std::fs::File::create(podcast_episode_path.clone()).unwrap();
        let mut image_out = std::fs::File::create(image_podcast_path.clone()).unwrap();

        if !self.file_service.check_if_podcast_main_image_downloaded(&podcast.clone().directory_id)
        {
            let mut image_podcast = std::fs::File::create(image_podcast_path.clone()).unwrap();
            io::copy(&mut image_response, &mut image_podcast).expect("failed to copy content");
        }

        io::copy(&mut resp, &mut podcast_out).expect("failed to copy content");

        self.db.update_total_podcast_time_and_image(
                &podcast_episode.episode_id,
                &image_podcast_path.clone(),
                &podcast_episode_path.clone(),
            )
            .expect("TODO: panic message");
        io::copy(&mut image_response, &mut image_out).expect("failed to copy content");
    }
}
