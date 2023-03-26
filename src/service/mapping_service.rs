use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::models::models::{PodcastHistoryItem, PodcastWatchedEpisodeModelWithPodcastEpisode};
use crate::service::environment_service;

#[derive(Clone)]
pub struct MappingService {
    env_service: environment_service::EnvironmentService,
}

impl MappingService {
    pub fn new() -> MappingService {
        MappingService {
            env_service: environment_service::EnvironmentService::new(),
        }
    }

    pub fn map_podcast_to_podcast_dto(&self, podcast: &Podcast) -> Podcast {
        Podcast {
            id: podcast.id,
            name: podcast.name.clone(),
            directory: podcast.directory.clone(),
            rssfeed: podcast.rssfeed.clone(),
            image_url: environment_service::EnvironmentService::get_server_url(&self.env_service)
                + &podcast.image_url.clone(),
            language: podcast.language.clone(),
            keywords: podcast.keywords.clone(),
            summary: podcast.summary.clone(),
            explicit: podcast.clone().explicit,
            favored: podcast.favored,
            last_build_date: podcast.clone().last_build_date,
            author: podcast.author.clone(),
            active: podcast.active,
        }
    }

    pub fn map_podcastepisode_to_dto(&self, podcast_episode: &PodcastEpisode)->PodcastEpisode{
        let podcast_path = environment_service::EnvironmentService::get_server_url(&self.env_service);
        PodcastEpisode{
            id: podcast_episode.id,
            podcast_id: podcast_episode.podcast_id,
            episode_id: podcast_episode.episode_id.clone(),
            name: podcast_episode.name.clone(),
            description: podcast_episode.description.clone(),
            url: podcast_episode.url.clone(),
            date_of_recording: podcast_episode.date_of_recording.clone(),
            image_url: podcast_episode.image_url.clone(),
            total_time: podcast_episode.total_time,
            local_url: podcast_path.clone()+&podcast_episode.local_url.clone(),
            local_image_url: podcast_path+&podcast_episode.local_image_url.clone(),
            status: podcast_episode.status.clone(),
            download_time: podcast_episode.download_time.clone(),
        }
    }

    pub fn map_podcast_history_item_to_with_podcast_episode (&self, podcast_watched_model: &PodcastHistoryItem,
                                                             podcast_episode: PodcastEpisode,
                                                             podcast: Podcast)
                                                             ->PodcastWatchedEpisodeModelWithPodcastEpisode
    {

        let cloned_podcast_watched_model = podcast_watched_model.clone();
        PodcastWatchedEpisodeModelWithPodcastEpisode{
            id: podcast_watched_model.clone().id,
            watched_time: podcast_watched_model.clone().watched_time,
            podcast_id: podcast_watched_model.clone().podcast_id,
            episode_id: cloned_podcast_watched_model.episode_id,
            date: cloned_podcast_watched_model.date,
            url: podcast_episode.clone().url,
            name: podcast_episode.clone().name,
            image_url: podcast_episode.clone().image_url,
            total_time: podcast_episode.clone().total_time,
            podcast_episode,
            podcast
        }
    }
}
