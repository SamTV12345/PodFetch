use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable, Selectable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::favorite_podcast_episode::{
    FavoritePodcastEpisode, FavoritePodcastEpisodeRepository,
};

diesel::table! {
    favorite_podcast_episodes (username, episode_id) {
        username -> Text,
        episode_id -> Integer,
        favorite -> Bool,
    }
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = favorite_podcast_episodes)]
struct FavoritePodcastEpisodeEntity {
    username: String,
    episode_id: i32,
    favorite: bool,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = favorite_podcast_episodes)]
struct FavoritePodcastEpisodeInsertEntity {
    username: String,
    episode_id: i32,
    favorite: bool,
}

impl From<FavoritePodcastEpisodeEntity> for FavoritePodcastEpisode {
    fn from(value: FavoritePodcastEpisodeEntity) -> Self {
        Self {
            username: value.username,
            episode_id: value.episode_id,
            favorite: value.favorite,
        }
    }
}

impl From<FavoritePodcastEpisode> for FavoritePodcastEpisodeInsertEntity {
    fn from(value: FavoritePodcastEpisode) -> Self {
        Self {
            username: value.username,
            episode_id: value.episode_id,
            favorite: value.favorite,
        }
    }
}

pub struct DieselFavoritePodcastEpisodeRepository {
    database: Database,
}

impl DieselFavoritePodcastEpisodeRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl FavoritePodcastEpisodeRepository for DieselFavoritePodcastEpisodeRepository {
    type Error = PersistenceError;

    fn get_by_username_and_episode_id(
        &self,
        username_to_search: &str,
        episode_id_to_search: i32,
    ) -> Result<Option<FavoritePodcastEpisode>, Self::Error> {
        use self::favorite_podcast_episodes::dsl as fpe_dsl;
        use self::favorite_podcast_episodes::table as fpe_table;

        fpe_table
            .filter(fpe_dsl::username.eq(username_to_search))
            .filter(fpe_dsl::episode_id.eq(episode_id_to_search))
            .first::<FavoritePodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|favorite| favorite.map(Into::into))
            .map_err(Into::into)
    }

    fn save_or_update(&self, favorite: FavoritePodcastEpisode) -> Result<(), Self::Error> {
        use self::favorite_podcast_episodes::dsl as fpe_dsl;
        use self::favorite_podcast_episodes::table as fpe_table;

        let favorite_to_store = favorite.clone();
        let existing = fpe_table
            .filter(fpe_dsl::username.eq(&favorite.username))
            .filter(fpe_dsl::episode_id.eq(favorite.episode_id))
            .first::<FavoritePodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()?;

        match existing {
            Some(_) => diesel::update(
                fpe_table
                    .filter(fpe_dsl::username.eq(&favorite.username))
                    .filter(fpe_dsl::episode_id.eq(favorite.episode_id)),
            )
            .set(fpe_dsl::favorite.eq(favorite.favorite))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into),
            None => diesel::insert_into(fpe_table)
                .values(FavoritePodcastEpisodeInsertEntity::from(favorite_to_store))
                .execute(&mut self.database.connection()?)
                .map(|_| ())
                .map_err(Into::into),
        }
    }

    fn get_favorites_by_username(
        &self,
        username_to_search: &str,
    ) -> Result<Vec<FavoritePodcastEpisode>, Self::Error> {
        use self::favorite_podcast_episodes::dsl as fpe_dsl;
        use self::favorite_podcast_episodes::table as fpe_table;

        fpe_table
            .filter(fpe_dsl::username.eq(username_to_search))
            .filter(fpe_dsl::favorite.eq(true))
            .load::<FavoritePodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|favs| favs.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn is_liked_by_someone(&self, episode_id_to_find: i32) -> Result<bool, Self::Error> {
        use self::favorite_podcast_episodes::dsl as fpe_dsl;
        use self::favorite_podcast_episodes::table as fpe_table;

        fpe_table
            .filter(fpe_dsl::episode_id.eq(episode_id_to_find))
            .filter(fpe_dsl::favorite.eq(true))
            .first::<FavoritePodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|favorite| favorite.map(|found| found.favorite).unwrap_or(false))
            .map_err(Into::into)
    }
}
