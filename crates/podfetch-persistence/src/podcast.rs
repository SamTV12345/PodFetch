use crate::db::{Database, PersistenceError};
use diesel::prelude::*;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::podcast::{
    NewPodcast, Podcast, PodcastMetadataUpdate, PodcastRepository,
};

diesel::table! {
    podcasts (id) {
        id -> Integer,
        name -> Text,
        directory_id -> Text,
        rssfeed -> Text,
        image_url -> Text,
        summary -> Nullable<Text>,
        language -> Nullable<Text>,
        explicit -> Nullable<Text>,
        keywords -> Nullable<Text>,
        last_build_date -> Nullable<Text>,
        author -> Nullable<Text>,
        active -> Bool,
        original_image_url -> Text,
        directory_name -> Text,
        download_location -> Nullable<Text>,
        guid -> Nullable<Text>,
    }
}

#[derive(Queryable, Identifiable, Selectable, Debug, Clone)]
#[diesel(table_name = podcasts)]
struct PodcastEntity {
    id: i32,
    name: String,
    directory_id: String,
    rssfeed: String,
    image_url: String,
    summary: Option<String>,
    language: Option<String>,
    explicit: Option<String>,
    keywords: Option<String>,
    last_build_date: Option<String>,
    author: Option<String>,
    active: bool,
    original_image_url: String,
    directory_name: String,
    download_location: Option<String>,
    guid: Option<String>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = podcasts)]
struct NewPodcastEntity {
    name: String,
    directory_id: String,
    rssfeed: String,
    image_url: String,
    original_image_url: String,
    directory_name: String,
}

impl From<PodcastEntity> for Podcast {
    fn from(entity: PodcastEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            directory_id: entity.directory_id,
            rssfeed: entity.rssfeed,
            image_url: entity.image_url,
            summary: entity.summary,
            language: entity.language,
            explicit: entity.explicit,
            keywords: entity.keywords,
            last_build_date: entity.last_build_date,
            author: entity.author,
            active: entity.active,
            original_image_url: entity.original_image_url,
            directory_name: entity.directory_name,
            download_location: entity.download_location,
            guid: entity.guid,
        }
    }
}

impl From<NewPodcast> for NewPodcastEntity {
    fn from(podcast: NewPodcast) -> Self {
        Self {
            name: podcast.name,
            directory_id: podcast.directory_id,
            rssfeed: podcast.rssfeed,
            image_url: podcast.image_url.clone(),
            original_image_url: podcast.image_url,
            directory_name: podcast.directory_name,
        }
    }
}

pub struct DieselPodcastRepository {
    database: Database,
}

impl DieselPodcastRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl PodcastRepository for DieselPodcastRepository {
    type Error = PersistenceError;

    fn create(&self, podcast: NewPodcast) -> Result<Podcast, Self::Error> {
        diesel::insert_into(podcasts::table)
            .values(NewPodcastEntity::from(podcast))
            .get_result::<PodcastEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn find_by_id(&self, id: i32) -> Result<Option<Podcast>, Self::Error> {
        podcasts::table
            .filter(podcasts::id.eq(id))
            .first::<PodcastEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_rss_feed(&self, rss_feed: &str) -> Result<Option<Podcast>, Self::Error> {
        podcasts::table
            .filter(podcasts::rssfeed.eq(rss_feed))
            .first::<PodcastEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_directory_id(&self, directory_id: &str) -> Result<Option<Podcast>, Self::Error> {
        podcasts::table
            .filter(podcasts::directory_id.eq(directory_id))
            .first::<PodcastEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_image_path(&self, path: &str) -> Result<Option<Podcast>, Self::Error> {
        podcasts::table
            .filter(podcasts::image_url.eq(path))
            .first::<PodcastEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_all(&self) -> Result<Vec<Podcast>, Self::Error> {
        podcasts::table
            .load::<PodcastEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn update_metadata(&self, update: PodcastMetadataUpdate) -> Result<(), Self::Error> {
        diesel::update(podcasts::table.filter(podcasts::id.eq(update.id)))
            .set((
                podcasts::author.eq(update.author),
                podcasts::keywords.eq(update.keywords),
                podcasts::explicit.eq(update.explicit),
                podcasts::language.eq(update.language),
                podcasts::summary.eq(update.description),
                podcasts::last_build_date.eq(update.last_build_date),
                podcasts::guid.eq(update.guid),
            ))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_active(&self, id: i32, active: bool) -> Result<(), Self::Error> {
        diesel::update(podcasts::table.filter(podcasts::id.eq(id)))
            .set(podcasts::active.eq(active))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_name(&self, id: i32, name: &str) -> Result<(), Self::Error> {
        diesel::update(podcasts::table.filter(podcasts::id.eq(id)))
            .set(podcasts::name.eq(name))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_rss_feed(&self, id: i32, rss_feed: &str) -> Result<(), Self::Error> {
        diesel::update(podcasts::table.filter(podcasts::id.eq(id)))
            .set(podcasts::rssfeed.eq(rss_feed))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_original_image_url(&self, id: i32, image_url: &str) -> Result<(), Self::Error> {
        diesel::update(podcasts::table.filter(podcasts::id.eq(id)))
            .set(podcasts::original_image_url.eq(image_url))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete(&self, id: i32) -> Result<(), Self::Error> {
        diesel::delete(podcasts::table.filter(podcasts::id.eq(id)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }
}
