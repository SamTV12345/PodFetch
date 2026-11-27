//! The timeline: the newest episodes across all podcasts.
//!
//! API composition over three contexts: the catalog lists the episodes,
//! the library restricts them to favorites and adds favorite marks, and
//! listening adds the user's latest action and filters out heard episodes.

use crate::history::EpisodeDto;
use crate::history::map_episode_to_dto;
use crate::podcast::PodcastDto;
use crate::podcast::map_podcast_to_dto;
use crate::podcast_episode::{TimelineFavorite, TimelineQueryParams};
use crate::podcast_episode_dto::PodcastEpisodeDto;
use crate::services::podcast::service::PodcastService;
use common_infrastructure::error::CustomError;
use podfetch_catalog::domain::podcast_episode::{
    EpisodeFilter, PodcastEpisode, PodcastEpisodeRepository,
};
use podfetch_catalog::persistence::podcast_episode::DieselPodcastEpisodeRepository;
use podfetch_identity::api::Principal;
use podfetch_listening::domain::episode::{Episode, EpisodeRepository, Listener};
use podfetch_persistence::db::database;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const PAGE: usize = 20;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    pub data: Vec<(
        PodcastEpisodeDto,
        PodcastDto,
        Option<EpisodeDto>,
        Option<TimelineFavorite>,
    )>,
    pub total_elements: i64,
}

impl TimelineItem {
    pub fn get_timeline(
        user: Principal,
        api_key: Option<&str>,
        params: TimelineQueryParams,
        server_url: &str,
    ) -> Result<TimelineItem, CustomError> {
        let user_id: Uuid = user.id.into();
        let favored_only = params.favored_only.unwrap_or(false);
        let _ = crate::library::filters().save_timeline_decision(user_id, favored_only);

        let catalog = DieselPodcastEpisodeRepository::new(database());
        let actions = crate::listening::episode_actions();
        let listener = Listener::new(user.id.as_uuid(), user.username.clone());

        // Library: which podcasts / episodes the listing is restricted to.
        let favorite_episodes = crate::library::favorite_episodes();
        let filter = EpisodeFilter {
            podcast_ids: favored_only
                .then(|| PodcastService::favored_podcast_ids(user_id))
                .transpose()?,
            episode_ids: params
                .favored_episodes
                .then(|| -> Result<Vec<Uuid>, CustomError> {
                    Ok(favorite_episodes
                        .get_favorites_by_user_id(user_id)?
                        .into_iter()
                        .filter(|favorite| favorite.favorite)
                        .map(|favorite| favorite.episode_id)
                        .collect())
                })
                .transpose()?,
        };

        // Listening: episodes the user already has any action for.
        let heard: Vec<String> = if params.not_listened {
            actions
                .guids_with_actions(&listener)
                .map_err(CustomError::from)?
        } else {
            Vec::new()
        };
        let heard_set: HashSet<&str> = heard.iter().map(String::as_str).collect();

        let mut total_elements = catalog
            .count_matching(&filter, None)
            .map_err(CustomError::from)?;
        for chunk in heard.chunks(500) {
            total_elements -= catalog
                .count_matching(&filter, Some(chunk))
                .map_err(CustomError::from)?;
        }

        // Catalog: read pages until PAGE episodes passed the listening filter.
        let mut cursor = params.last_timestamp;
        let mut page: Vec<PodcastEpisode> = Vec::new();
        loop {
            let batch = catalog
                .find_newest(&filter, cursor.as_deref(), PAGE as i64)
                .map_err(CustomError::from)?;
            let exhausted = batch.len() < PAGE;
            cursor = batch.last().map(|e| e.date_of_recording.clone());
            for episode in batch {
                if heard_set.contains(episode.guid.as_str()) {
                    continue;
                }
                page.push(episode);
                if page.len() == PAGE {
                    break;
                }
            }
            if exhausted || page.len() == PAGE {
                break;
            }
        }

        // Details for the page, each from its own context, in bulk.
        let guids: Vec<String> = page.iter().map(|e| e.guid.clone()).collect();
        let mut latest_actions: HashMap<String, Episode> = actions
            .find_latest_by_guids(&listener, &guids)
            .map_err(CustomError::from)?
            .into_iter()
            .filter_map(|action| Some((action.guid.clone()?, action)))
            .collect();
        let episode_ids: Vec<Uuid> = page.iter().map(|e| e.id).collect();
        let mut episode_favorites = favorite_episodes.favorites_of(user_id, &episode_ids)?;
        let podcast_ids: Vec<Uuid> = page
            .iter()
            .map(|e| e.podcast_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let podcasts: HashMap<String, _> = PodcastService::get_podcasts_by_ids(&podcast_ids)?
            .into_iter()
            .map(|podcast| (podcast.id.clone(), podcast))
            .collect();
        let podcast_favorites = PodcastService::favorites_of_podcasts(user_id, &podcast_ids)?;

        let data = page
            .into_iter()
            .filter_map(|episode| {
                let podcast = podcasts.get(&episode.podcast_id.to_string())?.clone();
                let history = latest_actions.remove(&episode.guid);
                let favorite_episode = episode_favorites.remove(&episode.id);
                let podcast_favorite = podcast_favorites
                    .get(&episode.podcast_id)
                    .map(|&favored| TimelineFavorite { favored });
                Some((
                    PodcastEpisodeDto::from_episode_with_user(
                        episode.into(),
                        api_key,
                        favorite_episode,
                        server_url,
                    ),
                    map_podcast_to_dto(podcast.into(), server_url),
                    history.as_ref().map(map_episode_to_dto),
                    podcast_favorite,
                ))
            })
            .collect();

        Ok(TimelineItem {
            data,
            total_elements,
        })
    }
}
