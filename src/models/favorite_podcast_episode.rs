use diesel::{Insertable, OptionalExtension, Queryable, QueryableByName, RunQueryDsl};
use utoipa::ToSchema;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::utils::error::{map_db_error, CustomError};
use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes;
use diesel::QueryDsl;
use crate::models::user::User;
use diesel::ExpressionMethods;

#[derive(Serialize, Deserialize, Queryable, Insertable, QueryableByName, Clone, ToSchema)]
#[diesel(table_name=favorite_podcast_episodes)]
pub struct FavoritePodcastEpisode {
    pub(crate) username: String,
    pub(crate) episode_id: i32,
    pub(crate) favorite: bool,
}

impl FavoritePodcastEpisode {
    pub(crate) fn is_podcast_episode_liked_by_someone(episode_id_to_find: i32) -> Result<bool,
        CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes::dsl::*;

       favorite_podcast_episodes.filter(episode_id.eq(episode_id_to_find))
            .filter(favorite.eq(true))
            .first::<FavoritePodcastEpisode>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
           .map(|res| res.map(|res| res.favorite).unwrap_or(false))
    }
}

impl FavoritePodcastEpisode {
    pub(crate) fn like_podcast_episode(episode: i32, user: &User, favored: bool) -> Result<(),
        CustomError> {
        let found_episode_id = Self::get_by_user_id_and_episode_id(&user.username, episode)?;
        match found_episode_id {
            Some(_) => {
                diesel::update(favorite_podcast_episodes::table)
                    .filter(favorite_podcast_episodes::episode_id.eq(episode))
                    .filter(favorite_podcast_episodes::username.eq(&user.username))
                    .set(favorite_podcast_episodes::favorite.eq(favored))
                    .execute(&mut get_connection())
                    .map_err(map_db_error)
                    .map(|_| ())
            }
            None => {
                let new_favorite = FavoritePodcastEpisode::new(&user.username, episode, favored);
                new_favorite.save()
            }
        }
    }
}

impl FavoritePodcastEpisode {
    pub fn new(user_id: &str, episode_id: i32, favorite: bool) -> Self {
        FavoritePodcastEpisode {
            username: user_id.to_string(),
            episode_id,
            favorite,
        }
    }

    pub fn save(&self) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes::dsl::favorite_podcast_episodes;
        diesel::insert_into(favorite_podcast_episodes)
            .values(self)
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_by_user_id_and_episode_id(user_id_to_search: &str, episode_id_to_search: i32) -> Result<Option<Self>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes::dsl::*;
        use diesel::ExpressionMethods;
        favorite_podcast_episodes
            .filter(username.eq(user_id_to_search))
            .filter(episode_id.eq(episode_id_to_search))
            .first(&mut get_connection())
            .optional()
            .map_err(map_db_error)
    }
}