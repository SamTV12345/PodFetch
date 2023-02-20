use crate::constants::constants::PODCASTS_ROOT_DIRECTORY;
use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::environment_service;
use crate::service::rust_service::get_url_file_suffix;

pub struct MappingService {
    env_service: environment_service::EnvironmentService,
    db: DB
}

impl MappingService {
    pub fn new() -> MappingService {
        MappingService {
            env_service: environment_service::EnvironmentService::new(),
            db: DB::new().unwrap(),
        }
    }

    pub fn map_podcast_to_podcast_dto(&self, podcast: &Podcast) -> Podcast {
        Podcast {
            id: podcast.id,
            name: podcast.name.clone(),
            directory: podcast.directory.clone(),
            rssfeed: podcast.rssfeed.clone(),
            image_url: environment_service::EnvironmentService::get_server_url(&self.env_service)
                +PODCASTS_ROOT_DIRECTORY+"/"+&podcast.directory+"/image.jpg",
        }
    }

    pub fn map_podcastepisode_to_dto(&self, podcast_episode: &PodcastEpisode)->PodcastEpisode{
        let podcast = self.db.get_podcast(podcast_episode.podcast_id).unwrap();
        let image_suffix = get_url_file_suffix(&podcast_episode.image_url);
        let podcast_suffix = get_url_file_suffix(&podcast_episode.url);
        let podcast_path = environment_service::EnvironmentService::get_server_url(&self.env_service)
            +PODCASTS_ROOT_DIRECTORY+"/"+&podcast.directory+"/"+&podcast_episode.episode_id;
        let podcast_path_clone = podcast_path.clone();
        PodcastEpisode{
            id: podcast_episode.id,
            podcast_id: podcast_episode.podcast_id,
            episode_id: podcast_episode.episode_id.clone(),
            name: podcast_episode.name.clone(),
            url: podcast_path+"/podcast."+&podcast_suffix,
            date: podcast_episode.date.clone(),
            image_url: podcast_path_clone+"/image."+&image_suffix,
        }
    }
}