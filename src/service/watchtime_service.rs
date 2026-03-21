use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::mappers::episode_mapper::map_episode_to_dto;
use crate::models::episode::Episode;
use crate::mappers::podcast_dto_mapper::map_podcast_to_dto;
use crate::utils::error::CustomError;
use podfetch_domain::user::User;
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_web::history::EpisodeDto;
use podfetch_web::podcast::PodcastDto;
use podfetch_web::watchtime::{
    PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel,
    WatchtimeApplicationService,
};

#[derive(Clone, Default)]
pub struct WatchtimeService;

impl WatchtimeService {
    pub fn new() -> Self {
        Self
    }
}

impl WatchtimeApplicationService for WatchtimeService {
    type Error = CustomError;
    type EpisodeDto = EpisodeDto;
    type LastWatchedItem =
        PodcastWatchedEpisodeModelWithPodcastEpisode<PodcastEpisodeDto, PodcastDto, EpisodeDto>;

    fn log_watchtime(
        &self,
        username: String,
        request: PodcastWatchedPostModel,
    ) -> Result<(), Self::Error> {
        Episode::log_watchtime(&request.podcast_episode_id, request.time, username)
    }

    fn get_last_watched(&self, username: &str) -> Result<Vec<Self::LastWatchedItem>, Self::Error> {
        let user = User::new(
            0,
            username.to_string(),
            "user",
            None::<String>,
            chrono::Utc::now().naive_utc(),
            true,
        );
        Episode::get_last_watched_episodes(&user)
        .map(|items| {
            items.into_iter()
                .map(|(podcast_episode, episode, podcast)| {
                    PodcastWatchedEpisodeModelWithPodcastEpisode {
                        podcast_episode: (
                            podcast_episode,
                            Some(user.clone()),
                            None::<FavoritePodcastEpisode>,
                        )
                            .into(),
                        podcast: map_podcast_to_dto(podcast),
                        episode: map_episode_to_dto(&episode),
                    }
                })
                .collect()
        })
    }

    fn get_watchtime(
        &self,
        episode_id: &str,
        username: &str,
    ) -> Result<Option<Self::EpisodeDto>, Self::Error> {
        Episode::get_watchtime(episode_id, username)
            .map(|episode| episode.as_ref().map(map_episode_to_dto))
    }
}
