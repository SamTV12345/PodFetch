use crate::constants::constants::PODCASTS_ROOT_DIRECTORY;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::environment_service;

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
                + &podcast.image_url.clone()
        }
    }

    pub fn map_podcastepisode_to_dto(&self, podcast_episode: &PodcastEpisode)->PodcastEpisode{
        let podcast_path = environment_service::EnvironmentService::get_server_url(&self.env_service);
        println!("{}",podcast_path.clone()+&podcast_episode.local_url.clone());
        PodcastEpisode{
            id: podcast_episode.id,
            podcast_id: podcast_episode.podcast_id,
            episode_id: podcast_episode.episode_id.clone(),
            name: podcast_episode.name.clone(),
            description: podcast_episode.description.clone(),
            url: podcast_episode.url.clone(),
            date: podcast_episode.date.clone(),
            image_url: podcast_episode.image_url.clone(),
            total_time: podcast_episode.total_time,
            local_url: podcast_path.clone()+&podcast_episode.local_url.clone(),
            local_image_url: podcast_path+&podcast_episode.local_image_url.clone(),
        }
    }
}