use crate::adapters::persistence::model::favorite::favorites::FavoriteEntity;
use crate::domain::models::favorite::favorite::Favorite;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::podcasts::Podcast;
use crate::utils::error::CustomError;

pub trait FavoriteRepository {
    fn search_podcasts(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        designated_username: &str,
    ) -> Result<Vec<(Podcast, Option<Favorite>)>, CustomError>;

    fn get_favored_podcasts(
        found_username: &str,
    ) -> Result<Vec<Podcast>, CustomError>;
    fn search_podcasts_favored(order: OrderCriteria,
                               title: Option<String>,
                               latest_pub: OrderOption,
                               designated_username: &str) -> Result<Vec<(Podcast, FavoriteEntity)>, CustomError>;
}