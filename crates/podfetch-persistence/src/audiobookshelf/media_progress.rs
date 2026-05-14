use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::audiobookshelf::media_progress::{MediaProgress, MediaProgressRepository};

diesel::table! {
    audiobookshelf_media_progress (id) {
        id -> Text,
        user_id -> Integer,
        library_item_id -> Text,
        episode_id -> Nullable<Text>,
        media_type -> Text,
        duration -> Double,
        position_seconds -> Double,
        progress -> Double,
        is_finished -> Bool,
        hide_from_continue_listening -> Bool,
        last_update -> Timestamp,
        started_at -> Timestamp,
        finished_at -> Nullable<Timestamp>,
    }
}

#[derive(Queryable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = audiobookshelf_media_progress)]
struct MediaProgressEntity {
    id: String,
    user_id: i32,
    library_item_id: String,
    episode_id: Option<String>,
    media_type: String,
    duration: f64,
    position_seconds: f64,
    progress: f64,
    is_finished: bool,
    hide_from_continue_listening: bool,
    last_update: NaiveDateTime,
    started_at: NaiveDateTime,
    finished_at: Option<NaiveDateTime>,
}

impl From<MediaProgressEntity> for MediaProgress {
    fn from(value: MediaProgressEntity) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            library_item_id: value.library_item_id,
            episode_id: value.episode_id,
            media_type: value.media_type,
            duration: value.duration,
            current_time: value.position_seconds,
            progress: value.progress,
            is_finished: value.is_finished,
            hide_from_continue_listening: value.hide_from_continue_listening,
            last_update: value.last_update,
            started_at: value.started_at,
            finished_at: value.finished_at,
        }
    }
}

impl From<MediaProgress> for MediaProgressEntity {
    fn from(value: MediaProgress) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            library_item_id: value.library_item_id,
            episode_id: value.episode_id,
            media_type: value.media_type,
            duration: value.duration,
            position_seconds: value.current_time,
            progress: value.progress,
            is_finished: value.is_finished,
            hide_from_continue_listening: value.hide_from_continue_listening,
            last_update: value.last_update,
            started_at: value.started_at,
            finished_at: value.finished_at,
        }
    }
}

pub struct DieselMediaProgressRepository {
    database: Database,
}

impl DieselMediaProgressRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl MediaProgressRepository for DieselMediaProgressRepository {
    type Error = PersistenceError;

    fn list_for_user(&self, lookup_user_id: i32) -> Result<Vec<MediaProgress>, Self::Error> {
        use self::audiobookshelf_media_progress::dsl::*;

        let mut conn = self.database.connection()?;
        audiobookshelf_media_progress
            .filter(user_id.eq(lookup_user_id))
            .load::<MediaProgressEntity>(&mut conn)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find(
        &self,
        lookup_user_id: i32,
        lookup_item_id: &str,
        lookup_episode_id: Option<&str>,
    ) -> Result<Option<MediaProgress>, Self::Error> {
        use self::audiobookshelf_media_progress::dsl::*;

        let mut conn = self.database.connection()?;
        let mut query = audiobookshelf_media_progress
            .filter(user_id.eq(lookup_user_id))
            .filter(library_item_id.eq(lookup_item_id))
            .into_boxed();
        query = match lookup_episode_id {
            Some(value) => query.filter(episode_id.eq(value)),
            None => query.filter(episode_id.is_null()),
        };
        query
            .first::<MediaProgressEntity>(&mut conn)
            .optional()
            .map(|row| row.map(Into::into))
            .map_err(Into::into)
    }

    fn upsert(&self, progress_value: MediaProgress) -> Result<MediaProgress, Self::Error> {
        use self::audiobookshelf_media_progress::dsl::*;

        let mut conn = self.database.connection()?;
        let entity = MediaProgressEntity::from(progress_value);
        let existing = audiobookshelf_media_progress
            .filter(id.eq(&entity.id))
            .first::<MediaProgressEntity>(&mut conn)
            .optional()
            .map_err(PersistenceError::from)?;

        if existing.is_some() {
            diesel::update(audiobookshelf_media_progress.filter(id.eq(&entity.id)))
                .set(entity.clone())
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        } else {
            diesel::insert_into(audiobookshelf_media_progress)
                .values(entity.clone())
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        }
        Ok(entity.into())
    }

    fn delete(&self, lookup_id: &str) -> Result<usize, Self::Error> {
        use self::audiobookshelf_media_progress::dsl::*;

        let mut conn = self.database.connection()?;
        diesel::delete(audiobookshelf_media_progress.filter(id.eq(lookup_id)))
            .execute(&mut conn)
            .map_err(Into::into)
    }
}
