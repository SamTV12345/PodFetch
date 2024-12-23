use std::path::PathBuf;
use crate::adapters::filesystem::update_podcast::UpdatePodcast;
use crate::adapters::persistence::repositories::podcast::podcast::PodcastRepositoryImpl;
use crate::application::services::episode::episode_service::EpisodeService;
use crate::application::services::favorite::service::FavoriteService;
use crate::application::services::podcast_episode::service::PodcastEpisodeService;
use crate::application::services::tag::tag::TagService;
use crate::application::usecases::favorite::query_use_case::QueryUseCase;
use crate::domain::models::favorite::favorite::Favorite;
use crate::domain::models::order_criteria::{OrderCriteria, OrderOption};
use crate::domain::models::podcast::podcast::Podcast;
use crate::domain::models::tag::tag::Tag;
use crate::utils::error::CustomError;

pub struct PodcastService;

impl PodcastService {
    pub(crate) fn search_podcasts_favored(order: OrderCriteria, title: Option<String>, latest_pub:
    OrderOption, designated_username: String, tag: Option<String>) ->  Result<Vec<(Podcast, Favorite, Vec<Tag>)>, CustomError> {
        let podcasts =
            FavoriteService::search_podcasts_favored(order, title, latest_pub,
                                              &designated_username)?;
        let mut podcast_dto_vec: Vec<(Podcast, Favorite, Vec<Tag>)> = Vec::new();
        for podcast in podcasts {
            let tags_of_podcast = TagService::get_tags_of_podcast(podcast.0.id, &designated_username)?;
            podcast_dto_vec.push((podcast.0, podcast.1, tags_of_podcast));
        }

        if let Some(tag) = tag {
            podcast_dto_vec = podcast_dto_vec.into_iter().filter(|podcast| {
                podcast.2.iter().any(|tag_of_podcast| tag_of_podcast.name == tag)
            }).collect();
        }

        Ok(podcast_dto_vec)
    }
}

impl PodcastService {
    pub fn delete_podcast(podcast_id: i32, delete_files: bool) -> Result<(), CustomError> {
        let found_podcast = PodcastRepositoryImpl::get_podcast(podcast_id)?;
        if found_podcast.is_none() {
            return Ok(());
        }

        let found_podcast = found_podcast.unwrap();
        if delete_files {
            UpdatePodcast::delete_podcast_files(&PathBuf::from(found_podcast.directory_name))?;
        }
        EpisodeService::delete_watchtime(podcast_id)?;
        PodcastEpisodeService::delete_episodes_of_podcast(podcast_id)?;

        Ok(())
    }

    pub fn get_all_podcasts() -> Result<Vec<Podcast>, CustomError> {
        PodcastRepositoryImpl::get_all_podcasts()
    }

    pub fn get_podcast(id: i32) -> Result<Option<Podcast>, CustomError> {
        PodcastRepositoryImpl::get_podcast(id)
    }

    pub fn get_podcast_by_directory_id(podcast_id: &str) -> Result<Option<Podcast>, CustomError> {
        PodcastRepositoryImpl::get_podcast_by_directory_id(podcast_id)
    }

    pub fn update_podcast(podcast: Podcast) -> Result<Podcast, CustomError> {
        PodcastRepositoryImpl::update_podcast(podcast)
    }

    pub fn get_podcast_by_rss_feed(rss_feed: &str) -> Result<Option<Podcast>, CustomError> {
        PodcastRepositoryImpl::get_podcast_by_rss_feed(rss_feed)
    }

    pub fn search_podcasts(order: OrderCriteria,
                                   title: Option<String>,
                                   latest_pub: OrderOption,
                                   designated_username: String,
                                   tag: Option<String>) -> Result<Vec<(Podcast, Option<Favorite>,
                                                                       Vec<Tag>)>, CustomError> {
        let podcasts =
            FavoriteService::search_podcasts(order, title, latest_pub, &designated_username)?;
        let mut podcast_dto_vec: Vec<(Podcast, Option<Favorite>, Vec<Tag>)> = Vec::new();
        for podcast in podcasts {
            let tags_of_podcast = TagService::get_tags_of_podcast(podcast.0.id, &designated_username)?;
            podcast_dto_vec.push((podcast.0, podcast.1, tags_of_podcast));
        }

        if let Some(tag) = tag {
            podcast_dto_vec = podcast_dto_vec.into_iter().filter(|podcast| {
                podcast.2.iter().any(|tag_of_podcast| tag_of_podcast.name == tag)
            }).collect();
        }

        Ok(podcast_dto_vec)
    }

    pub fn get_podcasts(username: &str) -> Result<Vec<(Podcast, Tag)>, CustomError> {
        PodcastRepositoryImpl::get_podcasts(username)
    }
}