use chrono::Utc;
use diesel::r2d2::ConnectionManager;
use diesel::RunQueryDsl;
use r2d2::PooledConnection;
use crate::adapters::api::controllers::podcast_episode_controller::TimelineQueryParams;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::DBType;
use crate::adapters::persistence::model::podcast_episode::podcast_episode::PodcastEpisodeEntity;
use crate::adapters::persistence::repositories::podcast::podcast_episode::PodcastEpisodeRepositoryImpl;
use crate::application::services::filter::service::FilterService;
use crate::application::services::playlist::playlist_item_service::PlaylistItemService;
use crate::domain::models::episode::episode::Episode;
use crate::domain::models::favorite::favorite::Favorite;
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::domain::models::podcast::podcast::Podcast;
use crate::domain::models::settings::filter::Filter;
use crate::domain::models::user::user::User;
use crate::utils::error::{map_db_error, CustomError};

pub struct PodcastEpisodeService;

impl PodcastEpisodeService {
    pub(crate) fn get_position_of_episode(timestamp: &String, pid: i32) -> Result<i64,
        CustomError> {
        PodcastEpisodeRepositoryImpl::get_position_of_episode(timestamp, pid)
    }
}

impl PodcastEpisodeService {
    pub fn get_timeline(
        username_to_search: &str,
        favored_only: bool,
        last_timestamp: Option<String>,
        not_listened: bool
    ) -> Result<(i32, (PodcastEpisode, Podcast, Option<Episode>, Option<Favorite>)), CustomError> {

        FilterService::save_decision_for_timeline(
            username_to_search,
            favored_only,
        )?;

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
        PodcastEpisodeRepositoryImpl::update_podcast_episode(podcast_episode)
    }

    pub fn get_podcast_episodes_of_podcast(
        podcast_id: i32,
        last_podcast_episode: Option<String>,
        user: User,
    ) -> Result<Vec<(PodcastEpisode, Option<Episode>)>, CustomError> {
        let last_podcast_episode = last_podcast_episode.unwrap_or_else(|| Utc::now().to_rfc3339());
        PodcastEpisodeRepositoryImpl::get_podcast_episodes_of_podcast(podcast_id,
                                                                      Option::from(last_podcast_episode), user)
    }
}