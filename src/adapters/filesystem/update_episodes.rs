use crate::adapters::filesystem::download_service::DownloadService;
use crate::adapters::filesystem::file_path::FilenameBuilderReturn;
use crate::application::services::notification::notification_service::NotificationService;
use crate::application::services::podcast_episode::service::PodcastEpisodeService;
use crate::domain::models::notification::notification::Notification;
use crate::domain::models::podcast::episode::{PodcastEpisode, PodcastEpisodeStatus};
use crate::domain::models::podcast::podcast::Podcast;
use crate::utils::error::CustomError;

pub struct UpdateEpisodes;


impl UpdateEpisodes {
    pub fn update_available_episodes(available_episodes: Vec<PodcastEpisode>, podcast: Podcast) {
        for mut e in available_episodes {
            if e.download_time.is_some() {
                let f_e = e.clone();
                let file_name_builder = FilenameBuilderReturn::new(f_e.file_episode_path.unwrap(),
                                                                   f_e.file_image_path.unwrap(), f_e
                                                                       .local_url, f_e
                                                                       .local_image_url);
                let result = DownloadService::handle_metadata_insertion(&file_name_builder, &mut e, &podcast);
                if result.is_err() {
                    log::error!("Error while updating metadata for episode: {}", e.id);
                }
            }
        }
    }

    pub fn perform_download(
        podcast_episode: &mut PodcastEpisode,
        podcast: &Podcast,
    ) -> Result<PodcastEpisode, CustomError> {
        log::info!("Downloading podcast episode: {}", podcast_episode.name);
        let mut download_service = DownloadService::new();
        download_service.download_podcast_episode(podcast_episode.clone(), podcast)?;
        podcast_episode.status = PodcastEpisodeStatus::Downloaded;
        PodcastEpisodeService::update_podcast_episode(podcast_episode)?;
        let notification = Notification {
            id: 0,
            message: podcast_episode.name.to_string(),
            created_at: chrono::Utc::now().naive_utc().to_string(),
            type_of_message: "Download".to_string(),
            status: "unread".to_string(),
        };
        NotificationService::insert_notification(notification)?;
        Ok(podcast_episode.clone())
    }
}