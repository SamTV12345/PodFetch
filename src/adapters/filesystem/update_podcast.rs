use std::fs;
use std::path::PathBuf;
use crate::adapters::filesystem::download_service::DownloadService;
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::domain::models::podcast::podcast::Podcast;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::{map_io_error, CustomError};

pub struct UpdatePodcast;


impl UpdatePodcast {
    pub fn delete_podcast_files(podcast_dir: &PathBuf) -> Result<(), CustomError> {
        fs::remove_dir_all(podcast_dir).map_err(|e|map_io_error(e, Some(podcast_dir.into())))?;
        Ok(())
    }

    pub async fn download_podcast_episode(podcast_episode: &PodcastEpisode, podcast: &Podcast) ->
                                                                                               Result<(), CustomError> {
        let mut download_service = DownloadService::new();
        download_service.download_podcast_episode(podcast_episode.clone(), podcast).await;
        match podcast_episode.status.is_downloaded() {
            true=>{

            }
            false=> {

            }
        }
        Ok(())
    }
}
