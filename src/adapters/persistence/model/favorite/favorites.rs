use crate::adapters::persistence::dbconfig::schema::favorites;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::service::mapping_service::MappingService;
use crate::utils::error::{map_db_error, CustomError};
use crate::DBType as DbConnection;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Text};
use serde::{Deserialize, Serialize};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::domain::models::favorite::favorite::Favorite;
use crate::models::tag::Tag;

#[derive(
    Queryable,
    Associations,
    Debug,
    PartialEq,
    QueryableByName,
    Serialize,
    Deserialize,
    Insertable,
    Clone,
    AsChangeset,
)]
#[diesel(belongs_to(Podcast, foreign_key = podcast_id))]
#[diesel(belongs_to(User, foreign_key = username))]
pub struct FavoriteEntity {
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Integer)]
    pub podcast_id: i32,
    #[diesel(sql_type = Bool)]
    pub favored: bool,
}


impl Into<Favorite> for FavoriteEntity {
    fn into(self) -> Favorite {
        Favorite {
            username: self.username,
            podcast_id: self.podcast_id,
            favored: self.favored,
        }
    }
}

impl FavoriteEntity {
    pub fn delete_by_username(
        username1: String,
        conn: &mut DbConnection,
    ) -> Result<(), diesel::result::Error> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::*;
        diesel::delete(favorites.filter(username.eq(username1))).execute(conn)?;
        Ok(())
    }

    pub fn update_podcast_favor(
        podcast_id_1: &i32,
        favor: bool,
        username_1: String,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favored as favor_column;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::podcast_id;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::username;

        let res = f_db
            .filter(
                podcast_id
                    .eq(podcast_id_1)
                    .and(username.eq(username_1.clone())),
            )
            .first::<FavoriteEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;

        match res {
            Some(..) => {
                diesel::update(
                    f_db.filter(podcast_id.eq(podcast_id_1).and(username.eq(username_1))),
                )
                .set(favor_column.eq(favor))
                .execute(&mut get_connection())
                .map_err(map_db_error)?;
                Ok(())
            }
            None => {
                insert_into(f_db)
                    .values((
                        podcast_id.eq(podcast_id_1),
                        username.eq(username_1),
                        favor_column.eq(favor),
                    ))
                    .execute(&mut get_connection())
                    .map_err(map_db_error)?;
                Ok(())
            }
        }
    }
}
