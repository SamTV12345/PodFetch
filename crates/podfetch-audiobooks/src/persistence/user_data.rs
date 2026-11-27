//! Everything this context stores per user, for the user cleanup.

use super::listening_session::audiobookshelf_listening_sessions;
use super::media_progress::audiobookshelf_media_progress;
use super::playback_session::audiobookshelf_playback_sessions;
use common_infrastructure::db::{Database, PersistenceError};
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

/// Deletes progress, playback sessions and listening sessions of a user in
/// one transaction; returns how many rows were removed.
pub fn delete_user_data(database: &Database, user_id: Uuid) -> Result<usize, PersistenceError> {
    let user_id = user_id.to_string();
    let mut conn = database.connection()?;
    conn.transaction(|conn| {
        let progress = diesel::delete(
            audiobookshelf_media_progress::table
                .filter(audiobookshelf_media_progress::user_id.eq(&user_id)),
        )
        .execute(conn)?;
        let playback = diesel::delete(
            audiobookshelf_playback_sessions::table
                .filter(audiobookshelf_playback_sessions::user_id.eq(&user_id)),
        )
        .execute(conn)?;
        let listening = diesel::delete(
            audiobookshelf_listening_sessions::table
                .filter(audiobookshelf_listening_sessions::user_id.eq(&user_id)),
        )
        .execute(conn)?;
        Ok::<_, diesel::result::Error>(progress + playback + listening)
    })
    .map_err(Into::into)
}
