use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::file_service::FileService;
use crate::service::mapping_service::MappingService;
use crate::service::path_service::PathService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use reqwest::blocking::ClientBuilder;
use std::fs::create_dir;
use std::io;

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

        let image_suffix = PodcastEpisodeService::get_url_file_suffix(&podcast_episode.image_url);
        let podcast_save_path = PathService::get_podcast_episode_path(
            &podcast.directory.clone(),
            &podcast_episode.clone().episode_id,
            &suffix,
        );

        let image_podcast_path =
            PathService::get_image_podcast_path(&podcast.directory.clone(), &image_suffix);

        let image_save_path = PathService::get_image_path(
            &podcast.directory.clone(),
            &podcast_episode.clone().episode_id,
            &image_suffix,
        );
        let client = ClientBuilder::new().build().unwrap();
        let mut resp = client.get(podcast_episode.url).send().unwrap();
        let mut image_response = client.get(podcast_episode.image_url).send().unwrap();

        create_dir(format!(
            "podcasts/{}/{}",
            podcast.directory, podcast_episode.episode_id
        ))
        .expect("Error creating directory");

        let mut podcast_out = std::fs::File::create(podcast_save_path.clone()).unwrap();
        let mut image_out = std::fs::File::create(image_save_path.clone()).unwrap();

        if !self
            .file_service
            .check_if_podcast_main_image_downloaded(&podcast.clone().directory)
        {
            let mut image_podcast = std::fs::File::create(image_podcast_path).unwrap();
            io::copy(&mut image_response, &mut image_podcast).expect("failed to copy content");
        }

        io::copy(&mut resp, &mut podcast_out).expect("failed to copy content");

        self.db
            .update_total_podcast_time_and_image(
                &podcast_episode.episode_id,
                &image_save_path,
                &podcast_save_path.clone(),
            )
            .expect("TODO: panic message");
        io::copy(&mut image_response, &mut image_out).expect("failed to copy content");
    }
}
