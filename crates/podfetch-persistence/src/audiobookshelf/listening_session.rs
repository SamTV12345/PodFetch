use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_domain::audiobookshelf::listening_session::{
    ListeningSession, ListeningSessionRepository,
};

diesel::table! {
    audiobookshelf_listening_sessions (id) {
        id -> Text,
        user_id -> Integer,
        library_id -> Nullable<Text>,
        library_item_id -> Text,
        episode_id -> Nullable<Text>,
        media_type -> Text,
        duration -> Double,
        current_time -> Double,
        time_listening -> Double,
        play_method -> Integer,
        started_at -> Timestamp,
        updated_at -> Timestamp,
        display_title -> Nullable<Text>,
        display_author -> Nullable<Text>,
        cover_path -> Nullable<Text>,
    }
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = audiobookshelf_listening_sessions)]
struct ListeningSessionEntity {
    id: String,
    user_id: i32,
    library_id: Option<String>,
    library_item_id: String,
    episode_id: Option<String>,
    media_type: String,
    duration: f64,
    current_time: f64,
    time_listening: f64,
    play_method: i32,
    started_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    display_title: Option<String>,
    display_author: Option<String>,
    cover_path: Option<String>,
}

impl From<ListeningSessionEntity> for ListeningSession {
    fn from(value: ListeningSessionEntity) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            library_id: value.library_id,
            library_item_id: value.library_item_id,
            episode_id: value.episode_id,
            media_type: value.media_type,
            duration: value.duration,
            current_time: value.current_time,
            time_listening: value.time_listening,
            play_method: value.play_method,
            started_at: value.started_at,
            updated_at: value.updated_at,
            display_title: value.display_title,
            display_author: value.display_author,
            cover_path: value.cover_path,
        }
    }
}

impl From<ListeningSession> for ListeningSessionEntity {
    fn from(value: ListeningSession) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            library_id: value.library_id,
            library_item_id: value.library_item_id,
            episode_id: value.episode_id,
            media_type: value.media_type,
            duration: value.duration,
            current_time: value.current_time,
            time_listening: value.time_listening,
            play_method: value.play_method,
            started_at: value.started_at,
            updated_at: value.updated_at,
            display_title: value.display_title,
            display_author: value.display_author,
            cover_path: value.cover_path,
        }
    }
}

pub struct DieselListeningSessionRepository {
    database: Database,
}

impl DieselListeningSessionRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl ListeningSessionRepository for DieselListeningSessionRepository {
    type Error = PersistenceError;

    fn create(&self, session: ListeningSession) -> Result<ListeningSession, Self::Error> {
        use self::audiobookshelf_listening_sessions::dsl::*;

        let mut conn = self.database.connection()?;
        let entity = ListeningSessionEntity::from(session);
        diesel::insert_into(audiobookshelf_listening_sessions)
            .values(entity.clone())
            .execute(&mut conn)
            .map_err(PersistenceError::from)?;
        Ok(entity.into())
    }

    fn list_for_user(
        &self,
        lookup_user_id: i32,
        limit: i64,
    ) -> Result<Vec<ListeningSession>, Self::Error> {
        use self::audiobookshelf_listening_sessions::dsl::*;

        let mut conn = self.database.connection()?;
        audiobookshelf_listening_sessions
            .filter(user_id.eq(lookup_user_id))
            .order(updated_at.desc())
            .limit(limit)
            .load::<ListeningSessionEntity>(&mut conn)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
}
