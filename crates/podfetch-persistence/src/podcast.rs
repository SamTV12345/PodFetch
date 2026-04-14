use crate::db::{Database, PersistenceError};
use diesel::prelude::*;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl,
};
use podfetch_domain::favorite::Favorite;
use podfetch_domain::podcast::{
    NewPodcast, Podcast, PodcastMetadataUpdate, PodcastRepository, PodcastWithFavorite,
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
        added_by -> Nullable<Integer>,
    }
}

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

diesel::table! {
    favorites (user_id, podcast_id) {
        user_id -> Integer,
        podcast_id -> Integer,
        favored -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(podcasts, podcast_episodes, favorites);

#[derive(Queryable, Identifiable, Selectable, Debug, Clone, Default)]
#[diesel(table_name = podcasts)]
pub struct PodcastEntity {
    pub id: i32,
    pub name: String,
    pub directory_id: String,
    pub rssfeed: String,
    pub image_url: String,
    pub summary: Option<String>,
    pub language: Option<String>,
    pub explicit: Option<String>,
    pub keywords: Option<String>,
    pub last_build_date: Option<String>,
    pub author: Option<String>,
    pub active: bool,
    pub original_image_url: String,
    pub directory_name: String,
    pub download_location: Option<String>,
    pub guid: Option<String>,
    pub added_by: Option<i32>,
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
    added_by: Option<i32>,
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
            added_by: entity.added_by,
        }
    }
}

impl From<Podcast> for PodcastEntity {
    fn from(value: Podcast) -> Self {
        Self {
            id: value.id,
            name: value.name,
            directory_id: value.directory_id,
            rssfeed: value.rssfeed,
            image_url: value.image_url,
            summary: value.summary,
            language: value.language,
            explicit: value.explicit,
            keywords: value.keywords,
            last_build_date: value.last_build_date,
            author: value.author,
            active: value.active,
            original_image_url: value.original_image_url,
            directory_name: value.directory_name,
            download_location: value.download_location,
            guid: value.guid,
            added_by: value.added_by,
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
            added_by: podcast.added_by,
        }
    }
}

#[derive(Queryable, Debug, Clone)]
struct FavoriteEntity {
    user_id: i32,
    podcast_id: i32,
    favored: bool,
}

impl From<FavoriteEntity> for Favorite {
    fn from(entity: FavoriteEntity) -> Self {
        Self {
            user_id: entity.user_id,
            podcast_id: entity.podcast_id,
            favored: entity.favored,
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

    fn find_by_track_id(&self, track_id: i32) -> Result<Option<Podcast>, Self::Error> {
        // track_id is stored as string in directory_id
        podcasts::table
            .filter(podcasts::directory_id.eq(track_id.to_string()))
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

    fn find_by_episode_id(&self, episode_id: i32) -> Result<Option<Podcast>, Self::Error> {
        podcasts::table
            .inner_join(podcast_episodes::table.on(podcast_episodes::podcast_id.eq(podcasts::id)))
            .filter(podcast_episodes::id.eq(episode_id))
            .select(podcasts::all_columns)
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

    fn find_all_with_favorites(
        &self,
        user_id: i32,
    ) -> Result<Vec<PodcastWithFavorite>, Self::Error> {
        podcasts::table
            .left_join(
                favorites::table.on(favorites::user_id
                    .eq(user_id)
                    .and(favorites::podcast_id.eq(podcasts::id))),
            )
            .load::<(PodcastEntity, Option<FavoriteEntity>)>(&mut self.database.connection()?)
            .map(|results| {
                results
                    .into_iter()
                    .map(|(podcast, favorite)| PodcastWithFavorite {
                        podcast: podcast.into(),
                        favorite: favorite.map(Into::into),
                    })
                    .collect()
            })
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

    fn update_image_url_and_download_location(
        &self,
        directory_id: &str,
        image_url: &str,
        download_location: &str,
    ) -> Result<(), Self::Error> {
        diesel::update(podcasts::table.filter(podcasts::directory_id.eq(directory_id)))
            .set((
                podcasts::image_url.eq(image_url),
                podcasts::download_location.eq(Some(download_location.to_string())),
            ))
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

    fn count_by_added_by(&self, user_id: i32) -> Result<i64, Self::Error> {
        use diesel::dsl::count_star;
        podcasts::table
            .filter(podcasts::added_by.eq(user_id))
            .select(count_star())
            .first::<i64>(&mut self.database.connection()?)
            .map_err(Into::into)
    }
}
