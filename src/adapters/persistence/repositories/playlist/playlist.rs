use diesel::{OptionalExtension, RunQueryDsl};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::model::playlist::playlist::PlaylistEntity;
use crate::domain::models::playlist::playlist::Playlist;
use crate::execute_with_conn;
use crate::utils::error::{map_db_error, CustomError};

pub struct PlaylistRepositoryImpl;

use diesel::QueryDsl;
use diesel::ExpressionMethods;
impl PlaylistRepositoryImpl {
    pub fn insert_playlist(playlist: Playlist) -> Result<Playlist, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        let entity = PlaylistEntity::from(playlist);

        let res = playlists
            .filter(name.eq(entity.name.clone()))
            .first::<PlaylistEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res.into());
        }


        execute_with_conn!(|conn| diesel::insert_into(playlists)
            .values(entity)
            .get_result::<PlaylistEntity>(conn)
            .map_err(map_db_error))
    }
}