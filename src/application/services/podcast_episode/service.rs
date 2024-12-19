use chrono::Utc;
use diesel::RunQueryDsl;
use crate::adapters::api::controllers::podcast_episode_controller::TimelineQueryParams;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::model::podcast_episode::podcast_episode::PodcastEpisodeEntity;
use crate::adapters::persistence::repositories::podcast::podcast_episode::PodcastEpisodeRepositoryImpl;
use crate::application::services::playlist::playlist_item_service::PlaylistItemService;
use crate::domain::models::episode::episode::Episode;
use crate::domain::models::favorite::favorite::Favorite;
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::domain::models::podcast::podcast::Podcast;
use crate::domain::models::settings::filter::Filter;
use crate::models::filter::Filter;
use crate::utils::error::{map_db_error, CustomError};

pub struct PodcastEpisodeService;


impl PodcastEpisodeService {
    pub fn get_timeline(
        username_to_search: &str,
        favored_only: bool,
        last_timestamp: Option<String>,
        not_listened: bool
    ) -> Result<(i32, (PodcastEpisode, Podcast, Option<Episode>, Option<Favorite>)), CustomError> {

        Filter::save_decision_for_timeline(
            username_to_search,
            favored_only,
        );

        PodcastEpisodeRepositoryImpl::get_timeline(
            &username_to_search,
            favored_only,
            last_timestamp,
            not_listened
        )
    }

    pub fn get_episodes_by_podcast_id(id_to_search: i32) -> Result<Vec<PodcastEpisode>, CustomError> {
        PodcastEpisodeRepositoryImpl::get_episodes_by_podcast_id(id_to_search)
    }

    pub fn delete_episodes_of_podcast(podcast_id: i32) -> Result<(), CustomError> {
        PodcastEpisodeRepositoryImpl::get_episodes_by_podcast_id(podcast_id)?.iter().for_each
        (|episode| {
            let result = PlaylistItemService::delete_playlist_items_by_episode_id(episode.id);
            if let Err(e) = result {
                log::error!("Error deleting playlist items: {:?}", e);
            }
        });
        PodcastEpisodeRepositoryImpl::delete_episodes_of_podcast(podcast_id)
    }

    pub fn update_podcast_episode(podcast_episode: &PodcastEpisode) -> Result<PodcastEpisode,
        CustomError> {
        PodcastEpisodeRepositoryImpl::update_podcast_episode(podcast_episode);
    }
}