use crate::domain::models::favorite::favorite::Favorite;
use crate::domain::models::order_criteria::{OrderCriteria, OrderOption};
use crate::domain::models::podcast::podcast::Podcast;
use crate::domain::models::tag::tag::Tag;
use crate::utils::error::CustomError;

pub trait QueryUseCase {
    fn search_podcasts(order: OrderCriteria, title: Option<String>, latest_pub: OrderOption,
                       designated_username: &str) -> Result<Vec<(Podcast, Option<Favorite>)>,
        CustomError>;
    fn search_podcasts_favored(order: OrderCriteria,
                               title: Option<String>,
                               latest_pub: OrderOption,
                               designated_username: &str) -> Result<Vec<(Podcast,
                                                                                 Favorite)>,
        CustomError>;
    fn get_favored_podcasts(found_username: &str) -> Result<Vec<(Podcast, Vec<Tag>)>,
        CustomError>;
}