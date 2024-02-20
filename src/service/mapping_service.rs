use crate::constants::inner_constants::{DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE};
use crate::models::favorites::Favorite;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::service::environment_service;

#[derive(Clone)]
pub struct MappingService {}

impl MappingService {
    pub fn map_podcast_to_podcast_dto(podcast: &Podcast) -> Podcast {
        Podcast {
            id: podcast.id,
            name: podcast.name.clone(),
            directory_id: podcast.directory_id.clone(),
            rssfeed: podcast.rssfeed.clone(),
            image_url: environment_service::EnvironmentService::get_server_url(
                ENVIRONMENT_SERVICE.get().unwrap(),
            ) + &podcast.image_url.clone(),
            language: podcast.language.clone(),
            keywords: podcast.keywords.clone(),
            summary: podcast.summary.clone(),
            explicit: podcast.clone().explicit,
            last_build_date: podcast.clone().last_build_date,
            author: podcast.author.clone(),
            active: podcast.active,
            original_image_url: podcast.original_image_url.clone(),
            directory_name: podcast.directory_name.clone(),
        }
    }

    pub fn map_podcast_to_podcast_dto_with_favorites(
        podcast_favorite_grouped: &(Podcast, Option<Favorite>),
    ) -> PodcastDto {
        let favorite = podcast_favorite_grouped.1.is_some()
            && podcast_favorite_grouped.1.clone().unwrap().favored;
        PodcastDto {
            id: podcast_favorite_grouped.0.id,
            name: podcast_favorite_grouped.0.name.clone(),
            directory_id: podcast_favorite_grouped.0.directory_id.clone(),
            rssfeed: podcast_favorite_grouped.0.rssfeed.clone(),
            image_url: environment_service::EnvironmentService::get_server_url(
                ENVIRONMENT_SERVICE.get().unwrap(),
            ) + &podcast_favorite_grouped.0.image_url.clone(),
            language: podcast_favorite_grouped.0.language.clone(),
            keywords: podcast_favorite_grouped.0.keywords.clone(),
            summary: podcast_favorite_grouped.0.summary.clone(),
            explicit: podcast_favorite_grouped.0.clone().explicit,
            last_build_date: podcast_favorite_grouped.0.clone().last_build_date,
            author: podcast_favorite_grouped.0.author.clone(),
            active: podcast_favorite_grouped.0.active,
            original_image_url: podcast_favorite_grouped.0.original_image_url.clone(),
            favorites: favorite,
        }
    }

    pub fn map_podcast_to_podcast_dto_with_favorites_option(
        podcast_favorite_grouped: &(Podcast, Favorite),
    ) -> PodcastDto {
        Self::map_podcast_to_podcast_dto_with_favorites(&(
            podcast_favorite_grouped.0.clone(),
            Some(podcast_favorite_grouped.1.clone()),
        ))
    }

    pub fn map_podcastepisode_to_dto(podcast_episode: &PodcastEpisode) -> PodcastEpisode {
        PodcastEpisode {
            id: podcast_episode.id,
            podcast_id: podcast_episode.podcast_id,
            episode_id: podcast_episode.episode_id.clone(),
            name: podcast_episode.name.clone(),
            description: podcast_episode.description.clone(),
            url: podcast_episode.url.clone(),
            date_of_recording: podcast_episode.date_of_recording.clone(),
            image_url: Self::map_image_url(&podcast_episode.image_url.clone()),
            total_time: podcast_episode.total_time,
            local_url: podcast_episode.local_url.clone(),
            local_image_url: Self::map_image_url(&podcast_episode.local_image_url),
            status: podcast_episode.status.clone(),
            download_time: podcast_episode.download_time,
            guid: podcast_episode.guid.clone(),
            deleted: podcast_episode.deleted,
            file_episode_path: None,
            file_image_path: None,
        }
    }

    fn map_image_url(image_url: &str) -> String {
        match image_url == DEFAULT_IMAGE_URL {
            true => {
                let env = ENVIRONMENT_SERVICE.get().unwrap();

                env.server_url.clone().to_owned() + DEFAULT_IMAGE_URL
            }
            false => image_url.to_string(),
        }
    }
}
