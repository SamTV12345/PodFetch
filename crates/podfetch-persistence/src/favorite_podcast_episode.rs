use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable, Selectable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::favorite_podcast_episode::{
    FavoritePodcastEpisode, FavoritePodcastEpisodeRepository,
};
use uuid::Uuid;

diesel::table! {
    favorite_podcast_episodes (user_id, episode_id) {
        user_id -> Text,
        episode_id -> Text,
        favorite -> Bool,
    }
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = favorite_podcast_episodes)]
struct FavoritePodcastEpisodeEntity {
    user_id: String,
    episode_id: String,
    favorite: bool,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = favorite_podcast_episodes)]
struct FavoritePodcastEpisodeInsertEntity {
    user_id: String,
    episode_id: String,
    favorite: bool,
}

impl From<FavoritePodcastEpisodeEntity> for FavoritePodcastEpisode {
    fn from(value: FavoritePodcastEpisodeEntity) -> Self {
        Self {
            user_id: Uuid::parse_str(&value.user_id).expect("valid uuid in db"),
            episode_id: Uuid::parse_str(&value.episode_id).expect("valid uuid in db"),
            favorite: value.favorite,
        }
    }
}

impl From<FavoritePodcastEpisode> for FavoritePodcastEpisodeInsertEntity {
    fn from(value: FavoritePodcastEpisode) -> Self {
        Self {
            user_id: value.user_id.to_string(),
            episode_id: value.episode_id.to_string(),
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

    fn get_by_user_id_and_episode_id(
        &self,
        user_id_to_search: Uuid,
        episode_id_to_search: Uuid,
    ) -> Result<Option<FavoritePodcastEpisode>, Self::Error> {
        use self::favorite_podcast_episodes::dsl as fpe_dsl;
        use self::favorite_podcast_episodes::table as fpe_table;

        fpe_table
            .filter(fpe_dsl::user_id.eq(user_id_to_search.to_string()))
            .filter(fpe_dsl::episode_id.eq(episode_id_to_search.to_string()))
            .first::<FavoritePodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|favorite| favorite.map(Into::into))
            .map_err(Into::into)
    }

    fn save_or_update(&self, favorite: FavoritePodcastEpisode) -> Result<(), Self::Error> {
        use self::favorite_podcast_episodes::dsl as fpe_dsl;
        use self::favorite_podcast_episodes::table as fpe_table;

        let favorite_to_store = favorite.clone();
        let user_id = favorite.user_id.to_string();
        let episode_id = favorite.episode_id.to_string();
        let existing = fpe_table
            .filter(fpe_dsl::user_id.eq(user_id.clone()))
            .filter(fpe_dsl::episode_id.eq(episode_id.clone()))
            .first::<FavoritePodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()?;

        match existing {
            Some(_) => diesel::update(
                fpe_table
                    .filter(fpe_dsl::user_id.eq(user_id.clone()))
                    .filter(fpe_dsl::episode_id.eq(episode_id.clone())),
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

    fn get_favorites_by_user_id(
        &self,
        user_id_to_search: Uuid,
    ) -> Result<Vec<FavoritePodcastEpisode>, Self::Error> {
        use self::favorite_podcast_episodes::dsl as fpe_dsl;
        use self::favorite_podcast_episodes::table as fpe_table;

        fpe_table
            .filter(fpe_dsl::user_id.eq(user_id_to_search.to_string()))
            .filter(fpe_dsl::favorite.eq(true))
            .load::<FavoritePodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|favs| favs.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn is_liked_by_someone(&self, episode_id_to_find: Uuid) -> Result<bool, Self::Error> {
        use self::favorite_podcast_episodes::dsl as fpe_dsl;
        use self::favorite_podcast_episodes::table as fpe_table;

        fpe_table
            .filter(fpe_dsl::episode_id.eq(episode_id_to_find.to_string()))
            .filter(fpe_dsl::favorite.eq(true))
            .first::<FavoritePodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|favorite| favorite.map(|found| found.favorite).unwrap_or(false))
            .map_err(Into::into)
    }
}
