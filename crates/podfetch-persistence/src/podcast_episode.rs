use crate::db::{Database, PersistenceError};
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::dsl::max;
use diesel::prelude::*;
use diesel::query_source::Alias;
use diesel::{BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, TextExpressionMethods};
use podfetch_domain::episode::Episode;
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_domain::podcast_episode::{
    NewPodcastEpisode, PodcastEpisode, PodcastEpisodeRepository, PodcastEpisodeWithHistory,
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

// Re-declare episode and favorite tables here for joins
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

diesel::table! {
    favorite_podcast_episodes (username, episode_id) {
        username -> Text,
        episode_id -> Integer,
        favorite -> Bool,
    }
}

diesel::table! {
    podcasts (id) {
        id -> Integer,
        name -> Text,
        directory_id -> Text,
        rssfeed -> Text,
        image_url -> Text,
        summary -> Nullable<Text>,
        language -> Nullable<Text>,
        explicit -> Text,
        keywords -> Nullable<Text>,
        last_build_date -> Nullable<Text>,
        author -> Nullable<Text>,
        active -> Bool,
        original_image_url -> Text,
        directory_name -> Text,
        download_location -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    podcast_episodes,
    episodes,
    favorite_podcast_episodes,
    podcasts,
);

#[derive(Queryable, Identifiable, Selectable, AsChangeset, Debug, Clone, PartialEq, Eq, Default)]
#[diesel(table_name = podcast_episodes)]
pub struct PodcastEpisodeEntity {
    pub id: i32,
    pub podcast_id: i32,
    pub episode_id: String,
    pub name: String,
    pub url: String,
    pub date_of_recording: String,
    pub image_url: String,
    pub total_time: i32,
    pub description: String,
    pub download_time: Option<NaiveDateTime>,
    pub guid: String,
    pub deleted: bool,
    pub file_episode_path: Option<String>,
    pub file_image_path: Option<String>,
    pub episode_numbering_processed: bool,
    pub download_location: Option<String>,
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

// Entity for reading from episodes table in joins
#[derive(Queryable, Selectable, Debug, Clone)]
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

// Entity for reading favorites in joins
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = favorite_podcast_episodes)]
struct FavoritePodcastEpisodeEntity {
    username: String,
    episode_id: i32,
    favorite: bool,
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

impl From<FavoritePodcastEpisodeEntity> for FavoritePodcastEpisode {
    fn from(entity: FavoritePodcastEpisodeEntity) -> Self {
        Self {
            username: entity.username,
            episode_id: entity.episode_id,
            favorite: entity.favorite,
        }
    }
}

impl From<PodcastEpisode> for PodcastEpisodeEntity {
    fn from(episode: PodcastEpisode) -> Self {
        Self {
            id: episode.id,
            podcast_id: episode.podcast_id,
            episode_id: episode.episode_id,
            name: episode.name,
            url: episode.url,
            date_of_recording: episode.date_of_recording,
            image_url: episode.image_url,
            total_time: episode.total_time,
            description: episode.description,
            download_time: episode.download_time,
            guid: episode.guid,
            deleted: episode.deleted,
            file_episode_path: episode.file_episode_path,
            file_image_path: episode.file_image_path,
            episode_numbering_processed: episode.episode_numbering_processed,
            download_location: episode.download_location,
        }
    }
}

impl PodcastEpisodeEntity {
    pub fn is_downloaded(&self) -> bool {
        self.download_location.is_some()
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

    fn query_by_url_like(&self, url_pattern: &str) -> Result<Option<PodcastEpisode>, Self::Error> {
        let pattern = format!("%{}%", url_pattern);
        podcast_episodes::table
            .filter(podcast_episodes::url.like(pattern))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn get_nth_page(
        &self,
        last_id: i32,
        limit: i64,
    ) -> Result<Vec<PodcastEpisode>, Self::Error> {
        podcast_episodes::table
            .filter(podcast_episodes::id.gt(last_id))
            .filter(podcast_episodes::file_episode_path.is_not_null())
            .order(podcast_episodes::id.asc())
            .limit(limit)
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_episodes_with_history(
        &self,
        podcast_id: i32,
        username: &str,
        last_date: Option<&str>,
        only_unlistened: bool,
        limit: i64,
    ) -> Result<PodcastEpisodeWithHistory, Self::Error> {
        let (ep1, ep2) = diesel::alias!(episodes as ep1, episodes as ep2);

        // Subquery to get the latest timestamp per episode guid for this user
        let subquery = ep2
            .select(max(ep2.field(episodes::timestamp)))
            .filter(ep2.field(episodes::username).eq(username))
            .filter(ep2.field(episodes::guid).eq(ep1.field(episodes::guid)))
            .group_by(ep2.field(episodes::guid));

        let mut query = podcast_episodes::table
            .filter(podcast_episodes::podcast_id.eq(podcast_id))
            .left_join(ep1.on(ep1.field(episodes::guid).eq(podcast_episodes::guid.nullable())))
            .left_join(
                favorite_podcast_episodes::table.on(
                    favorite_podcast_episodes::episode_id
                        .eq(podcast_episodes::id)
                        .and(favorite_podcast_episodes::username.eq(username)),
                ),
            )
            .filter(
                ep1.field(episodes::timestamp)
                    .nullable()
                    .eq_any(subquery)
                    .or(ep1.field(episodes::timestamp).is_null()),
            )
            .order(podcast_episodes::date_of_recording.desc())
            .limit(limit)
            .into_boxed();

        if let Some(last_date_str) = last_date {
            query = query.filter(podcast_episodes::date_of_recording.lt(last_date_str));
        }

        if only_unlistened {
            query = query.filter(
                ep1.field(episodes::position)
                    .is_null()
                    .or(ep1.field(episodes::total).ne(ep1.field(episodes::position))),
            );
        }

        query
            .load::<(
                PodcastEpisodeEntity,
                Option<EpisodeEntity>,
                Option<FavoritePodcastEpisodeEntity>,
            )>(&mut self.database.connection()?)
            .map(|rows| {
                rows.into_iter()
                    .map(|(pe, ep, fav)| {
                        (
                            pe.into(),
                            ep.map(Into::into),
                            fav.map(Into::into),
                        )
                    })
                    .collect()
            })
            .map_err(Into::into)
    }

    fn get_position_of_episode(
        &self,
        timestamp: &str,
        podcast_id: i32,
    ) -> Result<usize, Self::Error> {
        let result = podcast_episodes::table
            .filter(podcast_episodes::podcast_id.eq(podcast_id))
            .filter(podcast_episodes::date_of_recording.le(timestamp))
            .order(podcast_episodes::date_of_recording.desc())
            .execute(&mut self.database.connection()?)
            .map_err(PersistenceError::from)?;
        Ok(result)
    }

    fn get_last_n_episodes(
        &self,
        podcast_id: i32,
        n: i64,
    ) -> Result<Vec<PodcastEpisode>, Self::Error> {
        podcast_episodes::table
            .filter(podcast_episodes::podcast_id.eq(podcast_id))
            .limit(n)
            .order(podcast_episodes::date_of_recording.desc())
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_all(&self) -> Result<Vec<PodcastEpisode>, Self::Error> {
        podcast_episodes::table
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn check_if_downloaded(&self, url: &str) -> Result<bool, Self::Error> {
        let result = podcast_episodes::table
            .filter(podcast_episodes::download_location.is_not_null())
            .filter(podcast_episodes::url.eq(url))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map_err(PersistenceError::from)?;
        Ok(result.is_some())
    }

    fn get_episodes_older_than_days(
        &self,
        days: i64,
        podcast_id: i32,
    ) -> Result<Vec<PodcastEpisode>, Self::Error> {
        let cutoff = Utc::now().naive_utc() - Duration::days(days);
        podcast_episodes::table
            .filter(podcast_episodes::download_time.lt(cutoff))
            .filter(podcast_episodes::podcast_id.eq(podcast_id))
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_episodes_by_podcast_to_k(&self, top_k: i64) -> Result<Vec<PodcastEpisode>, Self::Error> {
        let (pe1, pe2) = diesel::alias!(podcast_episodes as pe1, podcast_episodes as pe2);

        pe1.select(Alias::fields(&pe1, podcast_episodes::all_columns))
            .inner_join(podcasts::table.on(podcasts::id.eq(pe1.field(podcast_episodes::podcast_id))))
            .filter(
                pe1.field(podcast_episodes::date_of_recording).eq_any(
                    pe2.select(pe2.field(podcast_episodes::date_of_recording))
                        .filter(pe2.field(podcast_episodes::podcast_id).eq(podcasts::id))
                        .order(pe2.field(podcast_episodes::date_of_recording).desc())
                        .limit(top_k),
                ),
            )
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn update_local_paths(
        &self,
        episode_id: &str,
        file_image_path: &str,
        file_episode_path: &str,
    ) -> Result<(), Self::Error> {
        let result = podcast_episodes::table
            .filter(podcast_episodes::episode_id.eq(episode_id))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()?;

        if result.is_some() {
            diesel::update(
                podcast_episodes::table.filter(podcast_episodes::episode_id.eq(episode_id)),
            )
            .set((
                podcast_episodes::file_episode_path.eq(file_episode_path),
                podcast_episodes::file_image_path.eq(file_image_path),
            ))
            .execute(&mut self.database.connection()?)?;
        }
        Ok(())
    }

    fn update_download_status(
        &self,
        url: &str,
        download_location: Option<String>,
        download_time: NaiveDateTime,
    ) -> Result<PodcastEpisode, Self::Error> {
        diesel::update(podcast_episodes::table.filter(podcast_episodes::url.eq(url)))
            .set((
                podcast_episodes::download_location.eq(download_location),
                podcast_episodes::download_time.eq(download_time),
            ))
            .get_result::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn remove_download_status(&self, id: i32) -> Result<(), Self::Error> {
        diesel::update(podcast_episodes::table.filter(podcast_episodes::id.eq(id)))
            .set((
                podcast_episodes::download_location.eq::<Option<String>>(None),
                podcast_episodes::download_time.eq::<Option<NaiveDateTime>>(None),
                podcast_episodes::file_episode_path.eq::<Option<String>>(None),
                podcast_episodes::file_image_path.eq::<Option<String>>(None),
            ))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_guid(&self, episode_id: &str, guid: &str) -> Result<(), Self::Error> {
        diesel::update(
            podcast_episodes::table.filter(podcast_episodes::episode_id.eq(episode_id)),
        )
        .set(podcast_episodes::guid.eq(guid))
        .execute(&mut self.database.connection()?)
        .map(|_| ())
        .map_err(Into::into)
    }

    fn update_deleted(&self, episode_id: &str, deleted: bool) -> Result<usize, Self::Error> {
        diesel::update(
            podcast_episodes::table.filter(podcast_episodes::episode_id.eq(episode_id)),
        )
        .set(podcast_episodes::deleted.eq(deleted))
        .execute(&mut self.database.connection()?)
        .map_err(Into::into)
    }

    fn update_episode_numbering_processed(
        &self,
        episode_id: &str,
        processed: bool,
    ) -> Result<(), Self::Error> {
        diesel::update(
            podcast_episodes::table.filter(podcast_episodes::episode_id.eq(episode_id)),
        )
        .set(podcast_episodes::episode_numbering_processed.eq(processed))
        .execute(&mut self.database.connection()?)
        .map(|_| ())
        .map_err(Into::into)
    }
}
