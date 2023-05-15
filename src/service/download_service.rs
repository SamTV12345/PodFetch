use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::file_service::{determine_image_and_local_podcast_audio_url, FileService, prepare_podcast_episode_title_to_directory, prepare_podcast_title_to_directory};
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

    pub fn download_podcast_episode(&mut self, mut podcast_episode: PodcastEpisode, mut podcast: Podcast) {
        let suffix = PodcastEpisodeService::get_url_file_suffix(&podcast_episode.url);

        let image_suffix = PodcastEpisodeService::get_url_file_suffix(&podcast_episode.image_url);


        let (image_save_path, podcast_save_path) = determine_image_and_local_podcast_audio_url
            (podcast.clone(), podcast_episode.clone(), &suffix, &image_suffix);

        let client = ClientBuilder::new().build().unwrap();
        let mut resp = client.get(podcast_episode.url).send().unwrap();
        let mut image_response = client.get(podcast_episode.image_url).send().unwrap();

        let podcast_episode_dir = format!(
            "{}/{}",
            podcast.directory_name, prepare_podcast_episode_title_to_directory(&mut podcast_episode.name)
        );
        let podcast_episode_dir = create_dir(podcast_episode_dir);

        match podcast_episode_dir {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error creating podcast episode directory {}", e);
                match FileService::create_podcast_root_directory_exists(){
                    Ok(_) => {}
                    Err(e) => {
                        if e.kind()==io::ErrorKind::AlreadyExists {
                            log::info!("Podcast root directory already exists")
                        }
                        else {
                            log::error!("Error creating podcast root directory");
                        }
                    }
                }

                match FileService::create_podcast_directory_exists(&podcast.name, &podcast.directory_id) {
                    Ok(_) => {}
                    Err(e) => {
                        if e.kind()==io::ErrorKind::AlreadyExists {
                            log::info!("Podcast directory already exists")
                        }
                        else {
                            log::error!("Error creating podcast directory {}",e);
                        }
                    }
                }
            }
        }

        let image_podcast_path =
            PathService::get_image_podcast_path(&podcast.directory_name.clone(), &image_suffix);
        let mut podcast_out = std::fs::File::create(podcast_save_path.clone()).unwrap();
        let mut image_out = std::fs::File::create(image_save_path.clone()).unwrap();

        if !self
            .file_service
            .check_if_podcast_main_image_downloaded(&podcast.clone().directory_id)
        {
            let mut image_podcast = std::fs::File::create(image_podcast_path).unwrap();
            io::copy(&mut image_response, &mut image_podcast).expect("failed to copy content");
        }

        io::copy(&mut resp, &mut podcast_out).expect("failed to copy content");

        self.db.update_total_podcast_time_and_image(
                &podcast_episode.episode_id,
                &image_save_path,
                &podcast_save_path.clone(),
            )
            .expect("TODO: panic message");
        io::copy(&mut image_response, &mut image_out).expect("failed to copy content");
    }
}
