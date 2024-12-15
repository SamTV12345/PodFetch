use crate::adapters::filesystem::file_path::FilenameBuilderReturn;
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::domain::models::podcast::podcast::Podcast;
use crate::service::download_service::DownloadService;

pub struct UpdateEpisodes;


impl UpdateEpisodes {
    pub fn update_available_episodes(available_episodes: Vec<PodcastEpisode>, podcast: Podcast) {
        for e in available_episodes {
            if e.download_time.is_some() {
                let f_e = e.clone();
                let file_name_builder = FilenameBuilderReturn::new(f_e.file_episode_path.unwrap(),
                                                                   f_e.file_image_path.unwrap(), f_e
                                                                       .local_url, f_e
                                                                       .local_image_url);
                let result = DownloadService::handle_metadata_insertion(&file_name_builder, &e
                    .clone(), &podcast);
                if result.is_err() {
                    log::error!("Error while updating metadata for episode: {}", e.id);
                }
            }
        }
    }
}