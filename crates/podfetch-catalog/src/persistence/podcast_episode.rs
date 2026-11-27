use crate::domain::podcast_episode::{
    EpisodeFilter, NewPodcastEpisode, PodcastEpisode, PodcastEpisodeRepository,
};
use chrono::{Duration, NaiveDateTime, Utc};
use common_infrastructure::db::{Database, PersistenceError};
use diesel::prelude::*;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl,
    TextExpressionMethods,
};
use uuid::Uuid;

diesel::table! {
    podcast_episodes (id) {
        id -> Text,
        legacy_id -> Nullable<BigInt>,
        podcast_id -> Text,
        episode_id -> Text,
        name -> Text,
        url -> Text,
        date_of_recording -> Text,
        image_url -> Text,
        total_time -> Integer,
        description -> Text,
        guid -> Text,
        episode_numbering_processed -> Bool,
        youtube_video_id -> Nullable<Text>,
    }
}

// File state of downloaded episodes (one row per episode that has any).
diesel::table! {
    episode_files (episode_id) {
        episode_id -> Text,
        download_time -> Nullable<Timestamp>,
        file_episode_path -> Nullable<Text>,
        file_image_path -> Nullable<Text>,
        download_location -> Nullable<Text>,
        deleted -> Bool,
    }
}

diesel::joinable!(episode_files -> podcast_episodes (episode_id));

diesel::table! {
    podcasts (id) {
        id -> Text,
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

diesel::allow_tables_to_appear_in_same_query!(podcast_episodes, podcasts, episode_files);

/// `podcast_episodes` with the file state of each episode joined in.
macro_rules! with_files {
    () => {
        podcast_episodes::table.left_join(episode_files::table)
    };
}

/// The columns of [`PodcastEpisodeEntity`], in field order, for queries on
/// [`with_files!`]. Episodes without a file row read as not downloaded and
/// not deleted.
macro_rules! entity_columns {
    () => {
        (
            podcast_episodes::id,
            podcast_episodes::legacy_id,
            podcast_episodes::podcast_id,
            podcast_episodes::episode_id,
            podcast_episodes::name,
            podcast_episodes::url,
            podcast_episodes::date_of_recording,
            podcast_episodes::image_url,
            podcast_episodes::total_time,
            podcast_episodes::description,
            episode_files::download_time.nullable(),
            podcast_episodes::guid,
            diesel::dsl::sql::<diesel::sql_types::Bool>("COALESCE(episode_files.deleted, FALSE)"),
            episode_files::file_episode_path.nullable(),
            episode_files::file_image_path.nullable(),
            podcast_episodes::episode_numbering_processed,
            episode_files::download_location.nullable(),
            podcast_episodes::youtube_video_id,
        )
    };
}

/// Episodes not deleted by the user (no file row counts as not deleted).
macro_rules! not_deleted {
    () => {
        episode_files::deleted
            .nullable()
            .is_null()
            .or(episode_files::deleted.nullable().eq(false))
    };
}

/// An episode with its file state, as the web layer works with it. Read
/// through [`with_files!`] / [`entity_columns!`]; written to
/// `podcast_episodes` and `episode_files` separately.
#[derive(Queryable, Debug, Clone, PartialEq, Eq, Default)]
pub struct PodcastEpisodeEntity {
    pub id: String,
    pub legacy_id: Option<i64>,
    pub podcast_id: String,
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
    pub youtube_video_id: Option<String>,
}

/// The catalog columns of an episode, for updates.
#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = podcast_episodes, treat_none_as_null = true)]
struct CatalogChangeset {
    legacy_id: Option<i64>,
    podcast_id: String,
    episode_id: String,
    name: String,
    url: String,
    date_of_recording: String,
    image_url: String,
    total_time: i32,
    description: String,
    guid: String,
    episode_numbering_processed: bool,
    youtube_video_id: Option<String>,
}

impl From<&PodcastEpisodeEntity> for CatalogChangeset {
    fn from(entity: &PodcastEpisodeEntity) -> Self {
        Self {
            legacy_id: entity.legacy_id,
            podcast_id: entity.podcast_id.clone(),
            episode_id: entity.episode_id.clone(),
            name: entity.name.clone(),
            url: entity.url.clone(),
            date_of_recording: entity.date_of_recording.clone(),
            image_url: entity.image_url.clone(),
            total_time: entity.total_time,
            description: entity.description.clone(),
            guid: entity.guid.clone(),
            episode_numbering_processed: entity.episode_numbering_processed,
            youtube_video_id: entity.youtube_video_id.clone(),
        }
    }
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = podcast_episodes)]
struct NewPodcastEpisodeEntity {
    id: String,
    legacy_id: Option<i64>,
    podcast_id: String,
    episode_id: String,
    name: String,
    url: String,
    date_of_recording: String,
    image_url: String,
    total_time: i32,
    description: String,
    guid: String,
    youtube_video_id: Option<String>,
}

impl From<PodcastEpisodeEntity> for PodcastEpisode {
    fn from(entity: PodcastEpisodeEntity) -> Self {
        Self {
            id: Uuid::parse_str(&entity.id).expect("valid uuid in db"),
            legacy_id: entity.legacy_id,
            podcast_id: Uuid::parse_str(&entity.podcast_id).expect("valid uuid in db"),
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
            youtube_video_id: entity.youtube_video_id,
        }
    }
}

impl From<&PodcastEpisode> for PodcastEpisodeEntity {
    fn from(episode: &PodcastEpisode) -> Self {
        Self {
            id: episode.id.to_string(),
            legacy_id: episode.legacy_id,
            podcast_id: episode.podcast_id.to_string(),
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
            youtube_video_id: episode.youtube_video_id.clone(),
        }
    }
}

impl From<NewPodcastEpisode> for NewPodcastEpisodeEntity {
    fn from(episode: NewPodcastEpisode) -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
            legacy_id: None,
            podcast_id: episode.podcast_id.to_string(),
            episode_id: episode.episode_id,
            name: episode.name,
            url: episode.url,
            date_of_recording: episode.date_of_recording,
            image_url: episode.image_url,
            total_time: episode.total_time,
            description: episode.description,
            guid: episode.guid,
            youtube_video_id: episode.youtube_video_id,
        }
    }
}

impl From<PodcastEpisode> for PodcastEpisodeEntity {
    fn from(episode: PodcastEpisode) -> Self {
        Self {
            id: episode.id.to_string(),
            legacy_id: episode.legacy_id,
            podcast_id: episode.podcast_id.to_string(),
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
            youtube_video_id: episode.youtube_video_id,
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

    fn load_by_id(&self, id: &str) -> Result<PodcastEpisode, PersistenceError> {
        with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::id.eq(id))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }
}

impl PodcastEpisodeRepository for DieselPodcastEpisodeRepository {
    type Error = PersistenceError;

    fn create(&self, episode: NewPodcastEpisode) -> Result<PodcastEpisode, Self::Error> {
        let row = NewPodcastEpisodeEntity::from(episode);
        let id = row.id.clone();
        diesel::insert_into(podcast_episodes::table)
            .values(row)
            .execute(&mut self.database.connection()?)?;
        self.load_by_id(&id)
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<PodcastEpisode>, Self::Error> {
        with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::id.eq(id.to_string()))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_legacy_id(&self, legacy_id: i64) -> Result<Option<PodcastEpisode>, Self::Error> {
        with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::legacy_id.eq(legacy_id))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_episode_id(&self, episode_id: &str) -> Result<Option<PodcastEpisode>, Self::Error> {
        with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::episode_id.eq(episode_id))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_url(
        &self,
        url: &str,
        podcast_id: Option<Uuid>,
    ) -> Result<Option<PodcastEpisode>, Self::Error> {
        let mut query = with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::url.eq(url))
            .into_boxed();

        if let Some(pid) = podcast_id {
            query = query.filter(podcast_episodes::podcast_id.eq(pid.to_string()));
        }

        query
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_guid(&self, guid: &str) -> Result<Option<PodcastEpisode>, Self::Error> {
        with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::guid.eq(guid))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_newest(
        &self,
        filter: &EpisodeFilter,
        before: Option<&str>,
        limit: i64,
    ) -> Result<Vec<PodcastEpisode>, Self::Error> {
        if filter.podcast_ids.as_ref().is_some_and(Vec::is_empty)
            || filter.episode_ids.as_ref().is_some_and(Vec::is_empty)
        {
            return Ok(Vec::new());
        }
        let mut query = with_files!()
            .select(entity_columns!())
            .order(podcast_episodes::date_of_recording.desc())
            .limit(limit)
            .into_boxed();
        if let Some(ids) = &filter.podcast_ids {
            query =
                query.filter(podcast_episodes::podcast_id.eq_any(ids.iter().map(Uuid::to_string)));
        }
        if let Some(ids) = &filter.episode_ids {
            query = query.filter(podcast_episodes::id.eq_any(ids.iter().map(Uuid::to_string)));
        }
        if let Some(before) = before {
            query = query.filter(podcast_episodes::date_of_recording.lt(before.to_owned()));
        }
        query
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn count_matching(
        &self,
        filter: &EpisodeFilter,
        guids: Option<&[String]>,
    ) -> Result<i64, Self::Error> {
        if filter.podcast_ids.as_ref().is_some_and(Vec::is_empty)
            || filter.episode_ids.as_ref().is_some_and(Vec::is_empty)
            || guids.is_some_and(<[String]>::is_empty)
        {
            return Ok(0);
        }
        let mut query = podcast_episodes::table.count().into_boxed();
        if let Some(ids) = &filter.podcast_ids {
            query =
                query.filter(podcast_episodes::podcast_id.eq_any(ids.iter().map(Uuid::to_string)));
        }
        if let Some(ids) = &filter.episode_ids {
            query = query.filter(podcast_episodes::id.eq_any(ids.iter().map(Uuid::to_string)));
        }
        if let Some(guids) = guids {
            query = query.filter(podcast_episodes::guid.eq_any(guids));
        }
        query
            .get_result::<i64>(&mut self.database.connection()?)
            .map_err(Into::into)
    }

    fn search_by_text(&self, text: &str) -> Result<Vec<PodcastEpisode>, Self::Error> {
        let pattern = format!("%{text}%");
        with_files!()
            .select(entity_columns!())
            .filter(
                podcast_episodes::name
                    .like(pattern.clone())
                    .or(podcast_episodes::description.like(pattern)),
            )
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_by_guids(&self, guids: &[String]) -> Result<Vec<PodcastEpisode>, Self::Error> {
        let conn = &mut self.database.connection()?;
        let mut found = Vec::new();
        // Chunked to stay below the bind parameter limit.
        for chunk in guids.chunks(500) {
            let rows = with_files!()
                .select(entity_columns!())
                .filter(podcast_episodes::guid.eq_any(chunk))
                .load::<PodcastEpisodeEntity>(conn)?;
            found.extend(rows.into_iter().map(Into::into));
        }
        Ok(found)
    }

    fn find_by_podcast_id(&self, podcast_id: Uuid) -> Result<Vec<PodcastEpisode>, Self::Error> {
        with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::podcast_id.eq(podcast_id.to_string()))
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_by_file_path(&self, path: &str) -> Result<Option<PodcastEpisode>, Self::Error> {
        with_files!()
            .select(entity_columns!())
            .filter(
                episode_files::file_episode_path
                    .eq(path)
                    .or(episode_files::file_image_path.eq(path)),
            )
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn update(&self, episode: &PodcastEpisode) -> Result<(), Self::Error> {
        let entity = PodcastEpisodeEntity::from(episode);
        diesel::update(podcast_episodes::table.filter(podcast_episodes::id.eq(&entity.id)))
            .set(CatalogChangeset::from(&entity))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        let id = id.to_string();
        // The other contexts removed what they keep about the episode
        // before (see `podcast_removal` in the web crate).
        diesel::delete(podcast_episodes::table.filter(podcast_episodes::id.eq(&id)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete_by_podcast_id(&self, podcast_id: Uuid) -> Result<(), Self::Error> {
        let podcast_id = podcast_id.to_string();
        // The other contexts removed what they keep about the episodes
        // before (see `podcast_removal` in the web crate).
        diesel::delete(podcast_episodes::table.filter(podcast_episodes::podcast_id.eq(&podcast_id)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn query_by_url_like(&self, url_pattern: &str) -> Result<Option<PodcastEpisode>, Self::Error> {
        let pattern = format!("%{url_pattern}%");
        with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::url.like(pattern))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn get_nth_page(&self, last_id: Uuid, limit: i64) -> Result<Vec<PodcastEpisode>, Self::Error> {
        with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::id.gt(last_id.to_string()))
            .filter(episode_files::file_episode_path.is_not_null())
            .order(podcast_episodes::id.asc())
            .limit(limit)
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_episodes_page(
        &self,
        podcast_id: Uuid,
        last_date: Option<&str>,
        limit: i64,
    ) -> Result<Vec<PodcastEpisode>, Self::Error> {
        let mut query = with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::podcast_id.eq(podcast_id.to_string()))
            .order(podcast_episodes::date_of_recording.desc())
            .limit(limit)
            .into_boxed();
        if let Some(last_date) = last_date {
            query = query.filter(podcast_episodes::date_of_recording.lt(last_date.to_owned()));
        }
        query
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_position_of_episode(
        &self,
        timestamp: &str,
        podcast_id: Uuid,
    ) -> Result<usize, Self::Error> {
        let result = podcast_episodes::table
            .filter(podcast_episodes::podcast_id.eq(podcast_id.to_string()))
            .filter(podcast_episodes::date_of_recording.le(timestamp))
            .count()
            .get_result::<i64>(&mut self.database.connection()?)
            .map_err(PersistenceError::from)?;
        Ok(result.saturating_sub(1) as usize)
    }

    fn get_total_episode_count(&self, podcast_id: Uuid) -> Result<usize, Self::Error> {
        let count = podcast_episodes::table
            .filter(podcast_episodes::podcast_id.eq(podcast_id.to_string()))
            .count()
            .get_result::<i64>(&mut self.database.connection()?)
            .map_err(PersistenceError::from)?;
        Ok(count as usize)
    }

    fn get_last_n_episodes(
        &self,
        podcast_id: Uuid,
        n: i64,
    ) -> Result<Vec<PodcastEpisode>, Self::Error> {
        with_files!()
            .select(entity_columns!())
            .filter(podcast_episodes::podcast_id.eq(podcast_id.to_string()))
            .limit(n)
            .order(podcast_episodes::date_of_recording.desc())
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_all(&self) -> Result<Vec<PodcastEpisode>, Self::Error> {
        with_files!()
            .select(entity_columns!())
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_inbox_episodes(
        &self,
        exclude_episode_ids: &[Uuid],
        last_date: Option<&str>,
        limit: i64,
    ) -> Result<Vec<PodcastEpisode>, Self::Error> {
        let exclude: Vec<String> = exclude_episode_ids
            .iter()
            .map(ToString::to_string)
            .collect();

        // `ne_all` on an empty list renders as an always-true predicate in
        // Diesel, so an untouched inbox correctly returns every candidate.
        // The inbox captures every new episode regardless of download state;
        // downloaded-but-not-yet-triaged episodes show up here too.
        let mut query = with_files!()
            .select(entity_columns!())
            .filter(not_deleted!())
            .filter(podcast_episodes::id.ne_all(exclude))
            .order(podcast_episodes::date_of_recording.desc())
            .limit(limit)
            .into_boxed();

        if let Some(last_date_str) = last_date {
            query = query.filter(podcast_episodes::date_of_recording.lt(last_date_str));
        }

        query
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_downloaded_episodes_paginated(
        &self,
        last_date: Option<&str>,
        limit: i64,
    ) -> Result<Vec<PodcastEpisode>, Self::Error> {
        let mut query = with_files!()
            .select(entity_columns!())
            .filter(episode_files::download_location.is_not_null())
            .filter(not_deleted!())
            .order(podcast_episodes::date_of_recording.desc())
            .limit(limit)
            .into_boxed();

        if let Some(last_date_str) = last_date {
            query = query.filter(podcast_episodes::date_of_recording.lt(last_date_str));
        }

        query
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn check_if_downloaded(&self, url: &str) -> Result<bool, Self::Error> {
        let result = with_files!()
            .select(entity_columns!())
            .filter(episode_files::download_location.is_not_null())
            .filter(podcast_episodes::url.eq(url))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map_err(PersistenceError::from)?;
        Ok(result.is_some())
    }

    fn get_episodes_older_than_days(
        &self,
        days: i64,
        podcast_id: Uuid,
    ) -> Result<Vec<PodcastEpisode>, Self::Error> {
        let cutoff = Utc::now().naive_utc() - Duration::days(days);
        with_files!()
            .select(entity_columns!())
            .filter(episode_files::download_time.lt(cutoff))
            .filter(podcast_episodes::podcast_id.eq(podcast_id.to_string()))
            .load::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_episodes_by_podcast_to_k(&self, top_k: i64) -> Result<Vec<PodcastEpisode>, Self::Error> {
        let (pe1, pe2) = diesel::alias!(podcast_episodes as pe1, podcast_episodes as pe2);
        let mut conn = self.database.connection()?;

        // The newest `top_k` episodes of every podcast ...
        let ids: Vec<String> = pe1
            .select(pe1.field(podcast_episodes::id))
            .inner_join(
                podcasts::table.on(podcasts::id.eq(pe1.field(podcast_episodes::podcast_id))),
            )
            .filter(
                pe1.field(podcast_episodes::date_of_recording).eq_any(
                    pe2.select(pe2.field(podcast_episodes::date_of_recording))
                        .filter(pe2.field(podcast_episodes::podcast_id).eq(podcasts::id))
                        .order(pe2.field(podcast_episodes::date_of_recording).desc())
                        .limit(top_k),
                ),
            )
            .load::<String>(&mut conn)?;
        // ... with their file state.
        let mut found = Vec::new();
        for chunk in ids.chunks(500) {
            let rows = with_files!()
                .select(entity_columns!())
                .filter(podcast_episodes::id.eq_any(chunk))
                .load::<PodcastEpisodeEntity>(&mut conn)?;
            found.extend(rows.into_iter().map(Into::into));
        }
        Ok(found)
    }

    fn update_guid(&self, episode_id: &str, guid: &str) -> Result<(), Self::Error> {
        diesel::update(podcast_episodes::table.filter(podcast_episodes::episode_id.eq(episode_id)))
            .set(podcast_episodes::guid.eq(guid))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_total_time(&self, episode_id: &str, total_time: i32) -> Result<(), Self::Error> {
        diesel::update(podcast_episodes::table.filter(podcast_episodes::episode_id.eq(episode_id)))
            .set(podcast_episodes::total_time.eq(total_time))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_episode_numbering_processed(
        &self,
        episode_id: &str,
        processed: bool,
    ) -> Result<(), Self::Error> {
        diesel::update(podcast_episodes::table.filter(podcast_episodes::episode_id.eq(episode_id)))
            .set(podcast_episodes::episode_numbering_processed.eq(processed))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }
}
