use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::audiobookshelf::playback_session::{
    PlayMethod, PlaybackSession, PlaybackSessionRepository,
};
use uuid::Uuid;

diesel::table! {
    audiobookshelf_playback_sessions (id) {
        id -> Text,
        user_id -> Text,
        library_id -> Nullable<Text>,
        library_item_id -> Text,
        episode_id -> Nullable<Text>,
        media_type -> Text,
        play_method -> Integer,
        position_seconds -> Double,
        duration -> Double,
        started_at -> Timestamp,
        updated_at -> Timestamp,
        finished_at -> Nullable<Timestamp>,
        time_listening_total -> Double,
        display_title -> Nullable<Text>,
        display_author -> Nullable<Text>,
        cover_path -> Nullable<Text>,
        media_metadata_json -> Nullable<Text>,
        device_info_json -> Nullable<Text>,
    }
}

#[derive(Queryable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = audiobookshelf_playback_sessions)]
struct PlaybackSessionEntity {
    id: String,
    user_id: String,
    library_id: Option<String>,
    library_item_id: String,
    episode_id: Option<String>,
    media_type: String,
    play_method: i32,
    position_seconds: f64,
    duration: f64,
    started_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    finished_at: Option<NaiveDateTime>,
    time_listening_total: f64,
    display_title: Option<String>,
    display_author: Option<String>,
    cover_path: Option<String>,
    media_metadata_json: Option<String>,
    device_info_json: Option<String>,
}

impl From<PlaybackSessionEntity> for PlaybackSession {
    fn from(value: PlaybackSessionEntity) -> Self {
        Self {
            id: value.id,
            user_id: Uuid::parse_str(&value.user_id).expect("valid uuid in db"),
            library_id: value.library_id,
            library_item_id: value.library_item_id,
            episode_id: value.episode_id,
            media_type: value.media_type,
            play_method: PlayMethod::from_i32(value.play_method),
            current_time: value.position_seconds,
            duration: value.duration,
            started_at: value.started_at,
            updated_at: value.updated_at,
            finished_at: value.finished_at,
            time_listening_total: value.time_listening_total,
            display_title: value.display_title,
            display_author: value.display_author,
            cover_path: value.cover_path,
            media_metadata_json: value.media_metadata_json,
            device_info_json: value.device_info_json,
        }
    }
}

impl From<PlaybackSession> for PlaybackSessionEntity {
    fn from(value: PlaybackSession) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id.to_string(),
            library_id: value.library_id,
            library_item_id: value.library_item_id,
            episode_id: value.episode_id,
            media_type: value.media_type,
            play_method: value.play_method.as_i32(),
            position_seconds: value.current_time,
            duration: value.duration,
            started_at: value.started_at,
            updated_at: value.updated_at,
            finished_at: value.finished_at,
            time_listening_total: value.time_listening_total,
            display_title: value.display_title,
            display_author: value.display_author,
            cover_path: value.cover_path,
            media_metadata_json: value.media_metadata_json,
            device_info_json: value.device_info_json,
        }
    }
}

pub struct DieselPlaybackSessionRepository {
    database: Database,
}

impl DieselPlaybackSessionRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl PlaybackSessionRepository for DieselPlaybackSessionRepository {
    type Error = PersistenceError;

    fn create(&self, session: PlaybackSession) -> Result<PlaybackSession, Self::Error> {
        use self::audiobookshelf_playback_sessions::dsl::*;

        let mut conn = self.database.connection()?;
        let entity = PlaybackSessionEntity::from(session);
        diesel::insert_into(audiobookshelf_playback_sessions)
            .values(entity.clone())
            .execute(&mut conn)
            .map_err(PersistenceError::from)?;
        Ok(entity.into())
    }

    fn find_by_id(&self, lookup_id: &str) -> Result<Option<PlaybackSession>, Self::Error> {
        use self::audiobookshelf_playback_sessions::dsl::*;

        let mut conn = self.database.connection()?;
        audiobookshelf_playback_sessions
            .filter(id.eq(lookup_id))
            .first::<PlaybackSessionEntity>(&mut conn)
            .optional()
            .map(|row| row.map(Into::into))
            .map_err(Into::into)
    }

    fn update(&self, session: PlaybackSession) -> Result<PlaybackSession, Self::Error> {
        use self::audiobookshelf_playback_sessions::dsl::*;

        let mut conn = self.database.connection()?;
        let entity = PlaybackSessionEntity::from(session);
        diesel::update(audiobookshelf_playback_sessions.filter(id.eq(&entity.id)))
            .set(entity.clone())
            .execute(&mut conn)
            .map_err(PersistenceError::from)?;
        Ok(entity.into())
    }

    fn delete(&self, lookup_id: &str) -> Result<usize, Self::Error> {
        use self::audiobookshelf_playback_sessions::dsl::*;

        let mut conn = self.database.connection()?;
        diesel::delete(audiobookshelf_playback_sessions.filter(id.eq(lookup_id)))
            .execute(&mut conn)
            .map_err(Into::into)
    }
}
