use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::episode::{Episode, EpisodeRepository, NewEpisode};

diesel::table! {
    episodes (id) {
        id -> Integer,
        username -> Text,
        device -> Text,
        podcast -> Text,
        episode -> Text,
        timestamp -> Timestamp,
        guid -> Nullable<Text>,
        action -> Text,
        started -> Nullable<Integer>,
        position -> Nullable<Integer>,
        total -> Nullable<Integer>,
    }
}

#[derive(Queryable, Debug, Clone)]
#[diesel(table_name = episodes)]
struct EpisodeEntity {
    id: i32,
    username: String,
    device: String,
    podcast: String,
    episode: String,
    timestamp: NaiveDateTime,
    guid: Option<String>,
    action: String,
    started: Option<i32>,
    position: Option<i32>,
    total: Option<i32>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = episodes)]
struct NewEpisodeEntity {
    username: String,
    device: String,
    podcast: String,
    episode: String,
    timestamp: NaiveDateTime,
    guid: Option<String>,
    action: String,
    started: Option<i32>,
    position: Option<i32>,
    total: Option<i32>,
}

impl From<EpisodeEntity> for Episode {
    fn from(entity: EpisodeEntity) -> Self {
        Self {
            id: entity.id,
            username: entity.username,
            device: entity.device,
            podcast: entity.podcast,
            episode: entity.episode,
            timestamp: entity.timestamp,
            guid: entity.guid,
            action: entity.action,
            started: entity.started,
            position: entity.position,
            total: entity.total,
        }
    }
}

impl From<NewEpisode> for NewEpisodeEntity {
    fn from(episode: NewEpisode) -> Self {
        Self {
            username: episode.username,
            device: episode.device,
            podcast: episode.podcast,
            episode: episode.episode,
            timestamp: episode.timestamp,
            guid: episode.guid,
            action: episode.action,
            started: episode.started,
            position: episode.position,
            total: episode.total,
        }
    }
}

pub struct DieselEpisodeRepository {
    database: Database,
}

impl DieselEpisodeRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl EpisodeRepository for DieselEpisodeRepository {
    type Error = PersistenceError;

    fn create(&self, episode: NewEpisode) -> Result<Episode, Self::Error> {
        diesel::insert_into(episodes::table)
            .values(NewEpisodeEntity::from(episode))
            .get_result::<EpisodeEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn find_by_username_and_episode(
        &self,
        username: &str,
        episode_url: &str,
    ) -> Result<Option<Episode>, Self::Error> {
        episodes::table
            .filter(
                episodes::username
                    .eq(username)
                    .and(episodes::episode.eq(episode_url)),
            )
            .order_by(episodes::timestamp.desc())
            .first::<EpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_username_device_guid(
        &self,
        username: &str,
        device: &str,
        guid: &str,
    ) -> Result<Option<Episode>, Self::Error> {
        episodes::table
            .filter(
                episodes::username
                    .eq(username)
                    .and(episodes::device.eq(device))
                    .and(episodes::guid.eq(guid)),
            )
            .order_by(episodes::timestamp.desc())
            .first::<EpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_actions_by_username(
        &self,
        username: &str,
        since: Option<NaiveDateTime>,
        device: Option<&str>,
        podcast: Option<&str>,
    ) -> Result<Vec<Episode>, Self::Error> {
        let mut query = episodes::table
            .filter(episodes::username.eq(username))
            .into_boxed();

        if let Some(since_date) = since {
            query = query.filter(episodes::timestamp.ge(since_date));
        }

        if let Some(device_id) = device {
            query = query.filter(episodes::device.eq(device_id));
        }

        if let Some(podcast_feed) = podcast {
            query = query.filter(episodes::podcast.eq(podcast_feed));
        }

        query
            .order_by(episodes::timestamp.desc())
            .load::<EpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn update_position(
        &self,
        id: i32,
        position: i32,
        timestamp: NaiveDateTime,
    ) -> Result<(), Self::Error> {
        diesel::update(episodes::table.filter(episodes::id.eq(id)))
            .set((
                episodes::started.eq(position),
                episodes::position.eq(position),
                episodes::timestamp.eq(timestamp),
            ))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error> {
        diesel::delete(episodes::table.filter(episodes::username.eq(username)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete_by_podcast_feed(&self, podcast_feed: &str) -> Result<(), Self::Error> {
        diesel::delete(episodes::table.filter(episodes::podcast.eq(podcast_feed)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }
}
