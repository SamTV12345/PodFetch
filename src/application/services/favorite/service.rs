use crate::adapters::persistence::repositories::favorite::favorite_repository_impl::FavoriteRepositoryImpl;
use crate::application::repositories::favorite_repository::FavoriteRepository;
use crate::application::usecases::favorite::query_use_case::QueryUseCase;
use crate::domain::models::favorite::favorite::Favorite;
use crate::domain::models::podcast::podcast::Podcast;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::tag::Tag;
use crate::utils::error::{CustomError};

pub struct FavoriteService;

impl QueryUseCase for FavoriteService {
    fn search_podcasts(order: OrderCriteria, title: Option<String>, latest_pub: OrderOption,
                       designated_username: &str) -> Result<Vec<(Podcast, Option<Favorite>)>,
        CustomError> {
        FavoriteRepositoryImpl::search_podcasts(order, title, latest_pub, designated_username)
    }

    fn search_podcasts_favored(order: OrderCriteria,
                                       title: Option<String>,
                                       latest_pub: OrderOption,
                                       designated_username: &str) -> Result<Vec<(Podcast,
                                                                                 Favorite)>,
        CustomError> {
        FavoriteRepositoryImpl::search_podcasts_favored(order, title, latest_pub,
                                                       designated_username)
    }

    fn get_favored_podcasts(found_username: &str) -> Result<Vec<(Podcast, Vec<Tag>)>,
        CustomError>{
        FavoriteRepositoryImpl::get_favored_podcasts(found_username)
    }
}

impl FavoriteService {
    fn delete_by_username(username: &str) -> Result<(), CustomError> {
        FavoriteRepositoryImpl::delete_by_username(username)
    }
}