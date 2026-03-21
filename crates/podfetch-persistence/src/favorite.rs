use crate::db::{Database, PersistenceError};
use diesel::prelude::*;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::favorite::{Favorite, FavoriteRepository};

diesel::table! {
    favorites (username, podcast_id) {
        username -> Text,
        podcast_id -> Integer,
        favored -> Bool,
    }
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = favorites)]
struct FavoriteEntity {
    username: String,
    podcast_id: i32,
    favored: bool,
}

impl From<FavoriteEntity> for Favorite {
    fn from(entity: FavoriteEntity) -> Self {
        Self {
            username: entity.username,
            podcast_id: entity.podcast_id,
            favored: entity.favored,
        }
    }
}

impl From<Favorite> for FavoriteEntity {
    fn from(favorite: Favorite) -> Self {
        Self {
            username: favorite.username,
            podcast_id: favorite.podcast_id,
            favored: favorite.favored,
        }
    }
}

pub struct DieselFavoriteRepository {
    database: Database,
}

impl DieselFavoriteRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl FavoriteRepository for DieselFavoriteRepository {
    type Error = PersistenceError;

    fn upsert(&self, favorite: Favorite) -> Result<(), Self::Error> {
        let entity = FavoriteEntity::from(favorite);

        let existing = favorites::table
            .filter(
                favorites::username
                    .eq(&entity.username)
                    .and(favorites::podcast_id.eq(entity.podcast_id)),
            )
            .first::<FavoriteEntity>(&mut self.database.connection()?)
            .optional()?;

        match existing {
            Some(_) => {
                diesel::update(
                    favorites::table.filter(
                        favorites::username
                            .eq(&entity.username)
                            .and(favorites::podcast_id.eq(entity.podcast_id)),
                    ),
                )
                .set(favorites::favored.eq(entity.favored))
                .execute(&mut self.database.connection()?)
                .map(|_| ())
                .map_err(Into::into)
            }
            None => diesel::insert_into(favorites::table)
                .values(&entity)
                .execute(&mut self.database.connection()?)
                .map(|_| ())
                .map_err(Into::into),
        }
    }

    fn find_by_username_and_podcast_id(
        &self,
        username: &str,
        podcast_id: i32,
    ) -> Result<Option<Favorite>, Self::Error> {
        favorites::table
            .filter(
                favorites::username
                    .eq(username)
                    .and(favorites::podcast_id.eq(podcast_id)),
            )
            .first::<FavoriteEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_favored_by_username(&self, username: &str) -> Result<Vec<Favorite>, Self::Error> {
        favorites::table
            .filter(favorites::username.eq(username).and(favorites::favored.eq(true)))
            .load::<FavoriteEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error> {
        diesel::delete(favorites::table.filter(favorites::username.eq(username)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }
}
