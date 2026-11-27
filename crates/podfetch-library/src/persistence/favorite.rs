use crate::domain::favorite::{Favorite, FavoriteRepository};
use common_infrastructure::db::{Database, PersistenceError};
use diesel::prelude::*;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use uuid::Uuid;

diesel::table! {
    favorites (user_id, podcast_id) {
        user_id -> Text,
        podcast_id -> Text,
        favored -> Bool,
    }
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, PartialEq, Eq)]
#[diesel(table_name = favorites)]
pub struct FavoriteEntity {
    pub user_id: String,
    pub podcast_id: String,
    pub favored: bool,
}

impl From<FavoriteEntity> for Favorite {
    fn from(entity: FavoriteEntity) -> Self {
        Self {
            user_id: Uuid::parse_str(&entity.user_id).expect("valid uuid in db"),
            podcast_id: Uuid::parse_str(&entity.podcast_id).expect("valid uuid in db"),
            favored: entity.favored,
        }
    }
}

impl From<Favorite> for FavoriteEntity {
    fn from(favorite: Favorite) -> Self {
        Self {
            user_id: favorite.user_id.to_string(),
            podcast_id: favorite.podcast_id.to_string(),
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
                favorites::user_id
                    .eq(entity.user_id.clone())
                    .and(favorites::podcast_id.eq(entity.podcast_id.clone())),
            )
            .first::<FavoriteEntity>(&mut self.database.connection()?)
            .optional()?;

        match existing {
            Some(_) => diesel::update(
                favorites::table.filter(
                    favorites::user_id
                        .eq(entity.user_id.clone())
                        .and(favorites::podcast_id.eq(entity.podcast_id.clone())),
                ),
            )
            .set(favorites::favored.eq(entity.favored))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into),
            None => diesel::insert_into(favorites::table)
                .values(&entity)
                .execute(&mut self.database.connection()?)
                .map(|_| ())
                .map_err(Into::into),
        }
    }

    fn find_by_user_id_and_podcast_id(
        &self,
        user_id: Uuid,
        podcast_id: Uuid,
    ) -> Result<Option<Favorite>, Self::Error> {
        favorites::table
            .filter(
                favorites::user_id
                    .eq(user_id.to_string())
                    .and(favorites::podcast_id.eq(podcast_id.to_string())),
            )
            .first::<FavoriteEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_favored_by_user_id(&self, user_id: Uuid) -> Result<Vec<Favorite>, Self::Error> {
        favorites::table
            .filter(
                favorites::user_id
                    .eq(user_id.to_string())
                    .and(favorites::favored.eq(true)),
            )
            .load::<FavoriteEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id: Uuid) -> Result<(), Self::Error> {
        diesel::delete(favorites::table.filter(favorites::user_id.eq(user_id.to_string())))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete_by_podcast_id(&self, podcast_id: Uuid) -> Result<(), Self::Error> {
        diesel::delete(favorites::table.filter(favorites::podcast_id.eq(podcast_id.to_string())))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_podcast_favor(
        &self,
        podcast_id: Uuid,
        favor: bool,
        user_id: Uuid,
    ) -> Result<(), Self::Error> {
        let podcast_id = podcast_id.to_string();
        let user_id = user_id.to_string();
        let existing = favorites::table
            .filter(
                favorites::podcast_id
                    .eq(podcast_id.clone())
                    .and(favorites::user_id.eq(user_id.clone())),
            )
            .first::<FavoriteEntity>(&mut self.database.connection()?)
            .optional()?;

        match existing {
            Some(_) => diesel::update(
                favorites::table.filter(
                    favorites::podcast_id
                        .eq(podcast_id.clone())
                        .and(favorites::user_id.eq(user_id.clone())),
                ),
            )
            .set(favorites::favored.eq(favor))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into),
            None => diesel::insert_into(favorites::table)
                .values((
                    favorites::podcast_id.eq(podcast_id),
                    favorites::user_id.eq(user_id),
                    favorites::favored.eq(favor),
                ))
                .execute(&mut self.database.connection()?)
                .map(|_| ())
                .map_err(Into::into),
        }
    }

    fn find_by_user_and_podcasts(
        &self,
        user_id: Uuid,
        podcast_ids: &[Uuid],
    ) -> Result<Vec<Favorite>, Self::Error> {
        if podcast_ids.is_empty() {
            return Ok(Vec::new());
        }
        favorites::table
            .filter(favorites::user_id.eq(user_id.to_string()))
            .filter(favorites::podcast_id.eq_any(podcast_ids.iter().map(Uuid::to_string)))
            .load::<FavoriteEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
}
