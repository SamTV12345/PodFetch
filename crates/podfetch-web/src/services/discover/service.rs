use crate::podcast::{Feed, PodindexResponse};
use crate::services::discover::categories::{
    CategoryMapping, all_category_names, name_to_itunes_genre, names_to_podindex_ids,
};
use crate::services::discover::itunes_charts::{ItunesChartEntry, ItunesChartsService};
use crate::services::podcast::service::PodcastService;
use common_infrastructure::error::CustomError;
use podfetch_domain::favorite::FavoriteRepository;
use podfetch_domain::podcast::PodcastRepository;
use podfetch_domain::user::User;
use podfetch_persistence::db::database;
use podfetch_persistence::favorite::DieselFavoriteRepository;
use podfetch_persistence::podcast::DieselPodcastRepository;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryDto {
    pub name: String,
    pub itunes_genre_id: u32,
    pub podindex_id: u32,
}

impl From<&CategoryMapping> for CategoryDto {
    fn from(value: &CategoryMapping) -> Self {
        Self {
            name: value.name.to_string(),
            itunes_genre_id: value.itunes_genre_id,
            podindex_id: value.podindex_id,
        }
    }
}

pub struct DiscoverService;

impl DiscoverService {
    pub async fn trending(
        categories: &[u32],
        language: Option<&str>,
        limit: u32,
    ) -> Result<PodindexResponse, CustomError> {
        let max = limit.clamp(1, 100);
        PodcastService::trending_on_podindex(categories, language, max).await
    }

    pub async fn charts(
        country: &str,
        genre: Option<u32>,
        limit: u32,
    ) -> Result<Vec<ItunesChartEntry>, CustomError> {
        ItunesChartsService::top_podcasts(country, genre, limit).await
    }

    pub fn categories() -> Vec<CategoryDto> {
        crate::services::discover::categories::CATEGORY_MAP
            .iter()
            .map(CategoryDto::from)
            .collect()
    }

    pub fn category_name_to_itunes_genre(name: &str) -> Option<u32> {
        name_to_itunes_genre(name)
    }

    pub fn all_category_names() -> Vec<&'static str> {
        all_category_names()
    }

    /// Produce a "for you" trending feed based on the categories of the user's
    /// subscribed podcasts.
    ///
    /// Logic: collect `keywords` for all favored podcasts of the user, split
    /// by comma, map each to Podcastindex category IDs, keep the top-3 most
    /// common, hit Podcastindex trending with those categories and the user's
    /// preferred language, then filter out feeds whose URL matches one the
    /// user is already subscribed to.
    pub async fn for_you(user: &User, limit: u32) -> Result<Vec<Feed>, CustomError> {
        let favorite_repo = DieselFavoriteRepository::new(database());
        let podcast_repo = DieselPodcastRepository::new(database());

        let favored = favorite_repo
            .find_favored_by_user_id(user.id)
            .map_err(CustomError::from)?;

        let mut subscribed_urls: HashSet<String> = HashSet::new();
        let mut category_counts: HashMap<u32, usize> = HashMap::new();

        for favorite in favored {
            let Ok(Some(podcast)) = podcast_repo.find_by_id(favorite.podcast_id) else {
                continue;
            };
            subscribed_urls.insert(podcast.rssfeed.clone());
            if let Some(keywords) = podcast.keywords.as_deref() {
                let names: Vec<&str> = keywords.split(',').map(str::trim).collect();
                for id in names_to_podindex_ids(&names) {
                    *category_counts.entry(id).or_insert(0) += 1;
                }
            }
        }

        let mut sorted: Vec<(u32, usize)> = category_counts.into_iter().collect();
        sorted.sort_by_key(|b| std::cmp::Reverse(b.1));
        let top_cats: Vec<u32> = sorted.into_iter().take(3).map(|(id, _)| id).collect();

        let language = user.language.as_deref();
        let max = limit.clamp(1, 100);
        let over_fetch = (max * 2).min(200);
        let trending =
            PodcastService::trending_on_podindex(&top_cats, language, over_fetch).await?;

        let filtered: Vec<Feed> = trending
            .feeds
            .into_iter()
            .filter(|f| {
                f.url
                    .as_deref()
                    .map(|u| !subscribed_urls.contains(u))
                    .unwrap_or(true)
            })
            .take(max as usize)
            .collect();

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_dto_is_populated() {
        let cats = DiscoverService::categories();
        assert!(cats.len() > 20);
        assert!(cats.iter().any(|c| c.name == "News"));
    }
}
