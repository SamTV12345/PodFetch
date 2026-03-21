use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::models::episode::{Episode, EpisodeDto};
use crate::models::podcast_dto::PodcastDto;
use crate::utils::error::CustomError;
use podfetch_domain::user::User;
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
        Episode::log_watchtime(request, username)
    }

    fn get_last_watched(&self, username: &str) -> Result<Vec<Self::LastWatchedItem>, Self::Error> {
        Episode::get_last_watched_episodes(&User::new(
            0,
            username.to_string(),
            "user",
            None::<String>,
            chrono::Utc::now().naive_utc(),
            true,
        ))
    }

    fn get_watchtime(
        &self,
        episode_id: &str,
        username: &str,
    ) -> Result<Option<Self::EpisodeDto>, Self::Error> {
        Episode::get_watchtime(episode_id, username)
            .map(|episode| episode.map(|episode| episode.convert_to_episode_dto()))
    }
}
