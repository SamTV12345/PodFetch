use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::podcast_episode::{
    NewPodcastEpisode, PodcastEpisode, PodcastEpisodeRepository,
};

diesel::table! {
    podcast_episodes (id) {
        id -> Integer,
        podcast_id -> Integer,
        episode_id -> Text,
        name -> Text,
        url -> Text,
        date_of_recording -> Text,
        image_url -> Text,
        total_time -> Integer,
        description -> Text,
        download_time -> Nullable<Timestamp>,
        guid -> Text,
        deleted -> Bool,
        file_episode_path -> Nullable<Text>,
        file_image_path -> Nullable<Text>,
        episode_numbering_processed -> Bool,
        download_location -> Nullable<Text>,
    }
}

#[derive(Queryable, Identifiable, Selectable, AsChangeset, Debug, Clone)]
#[diesel(table_name = podcast_episodes)]
struct PodcastEpisodeEntity {
    id: i32,
    podcast_id: i32,
    episode_id: String,
    name: String,
    url: String,
    date_of_recording: String,
    image_url: String,
    total_time: i32,
    description: String,
    download_time: Option<NaiveDateTime>,
    guid: String,
    deleted: bool,
    file_episode_path: Option<String>,
    file_image_path: Option<String>,
    episode_numbering_processed: bool,
    download_location: Option<String>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = podcast_episodes)]
struct NewPodcastEpisodeEntity {
    podcast_id: i32,
    episode_id: String,
    name: String,
    url: String,
    date_of_recording: String,
    image_url: String,
    total_time: i32,
    description: String,
    guid: String,
}

impl From<PodcastEpisodeEntity> for PodcastEpisode {
    fn from(entity: PodcastEpisodeEntity) -> Self {
        Self {
            id: entity.id,
            podcast_id: entity.podcast_id,
            episode_id: entity.episode_id,
            name: entity.name,
            url: entity.url,
            date_of_recording: entity.date_of_recording,
            image_url: entity.image_url,
            total_time: entity.total_time,
            description: entity.description,
            download_time: entity.download_time,
            guid: entity.guid,
            deleted: entity.deleted,
            file_episode_path: entity.file_episode_path,
            file_image_path: entity.file_image_path,
            episode_numbering_processed: entity.episode_numbering_processed,
            download_location: entity.download_location,
        }
    }
}

impl From<&PodcastEpisode> for PodcastEpisodeEntity {
    fn from(episode: &PodcastEpisode) -> Self {
        Self {
            id: episode.id,
            podcast_id: episode.podcast_id,
            episode_id: episode.episode_id.clone(),
            name: episode.name.clone(),
            url: episode.url.clone(),
            date_of_recording: episode.date_of_recording.clone(),
            image_url: episode.image_url.clone(),
            total_time: episode.total_time,
            description: episode.description.clone(),
            download_time: episode.download_time,
            guid: episode.guid.clone(),
            deleted: episode.deleted,
            file_episode_path: episode.file_episode_path.clone(),
            file_image_path: episode.file_image_path.clone(),
            episode_numbering_processed: episode.episode_numbering_processed,
            download_location: episode.download_location.clone(),
        }
    }
}

impl From<NewPodcastEpisode> for NewPodcastEpisodeEntity {
    fn from(episode: NewPodcastEpisode) -> Self {
        Self {
            podcast_id: episode.podcast_id,
            episode_id: episode.episode_id,
            name: episode.name,
            url: episode.url,
            date_of_recording: episode.date_of_recording,
            image_url: episode.image_url,
            total_time: episode.total_time,
            description: episode.description,
            guid: episode.guid,
        }
    }
}

pub struct DieselPodcastEpisodeRepository {
    database: Database,
}

impl DieselPodcastEpisodeRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl PodcastEpisodeRepository for DieselPodcastEpisodeRepository {
    type Error = PersistenceError;

    fn create(&self, episode: NewPodcastEpisode) -> Result<PodcastEpisode, Self::Error> {
        diesel::insert_into(podcast_episodes::table)
            .values(NewPodcastEpisodeEntity::from(episode))
            .get_result::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn find_by_id(&self, id: i32) -> Result<Option<PodcastEpisode>, Self::Error> {
        podcast_episodes::table
            .filter(podcast_episodes::id.eq(id))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_episode_id(&self, episode_id: &str) -> Result<Option<PodcastEpisode>, Self::Error> {
        podcast_episodes::table
            .filter(podcast_episodes::episode_id.eq(episode_id))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_url(
        &self,
        url: &str,
        podcast_id: Option<i32>,
    ) -> Result<Option<PodcastEpisode>, Self::Error> {
        let mut query = podcast_episodes::table
            .filter(podcast_episodes::url.eq(url))
            .into_boxed();

        if let Some(pid) = podcast_id {
            query = query.filter(podcast_episodes::podcast_id.eq(pid));
        }

        query
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_guid(&self, guid: &str) -> Result<Option<PodcastEpisode>, Self::Error> {
        podcast_episodes::table
            .filter(podcast_episodes::guid.eq(guid))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_podcast_id(&self, podcast_id: i32) -> Result<Vec<PodcastEpisode>, Self::Error> {
        podcast_episodes::table
            .filter(podcast_episodes::podcast_id.eq(podcast_id))
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_by_file_path(&self, path: &str) -> Result<Option<PodcastEpisode>, Self::Error> {
        podcast_episodes::table
            .filter(
                podcast_episodes::file_episode_path
                    .eq(path)
                    .or(podcast_episodes::file_image_path.eq(path)),
            )
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn update(&self, episode: &PodcastEpisode) -> Result<(), Self::Error> {
        let entity = PodcastEpisodeEntity::from(episode);
        diesel::update(podcast_episodes::table.filter(podcast_episodes::id.eq(episode.id)))
            .set(&entity)
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete(&self, id: i32) -> Result<(), Self::Error> {
        diesel::delete(podcast_episodes::table.filter(podcast_episodes::id.eq(id)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete_by_podcast_id(&self, podcast_id: i32) -> Result<(), Self::Error> {
        diesel::delete(podcast_episodes::table.filter(podcast_episodes::podcast_id.eq(podcast_id)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }
}
