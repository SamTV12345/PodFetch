use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::playlists;
use crate::models::playlist_item::PlaylistItem;
use crate::utils::error::ErrorSeverity::{Critical, Debug, Info, Warning};
use crate::utils::error::{CustomError, CustomErrorInner, map_db_error};
use crate::{DBType as DbConnection, execute_with_conn};
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use diesel::{Queryable, QueryableByName};
use podfetch_domain::user::User;
use podfetch_web::playlist::PlaylistDtoPost;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Insertable, QueryableByName, Clone, ToSchema)]
pub struct Playlist {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Integer)]
    pub user_id: i32,
}

impl Playlist {
    #[allow(clippy::redundant_closure_call)]
    pub fn insert_playlist(&self, conn: &mut DbConnection) -> Result<Playlist, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        let res = playlists
            .filter(name.eq(self.name.clone()))
            .first::<Playlist>(conn)
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res);
        }

        execute_with_conn!(|conn| diesel::insert_into(playlists)
            .values(self)
            .get_result::<Playlist>(conn)
            .map_err(|e| map_db_error(e, Critical)))
    }

    pub fn delete_playlist(
        playlist_id_1: String,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        diesel::delete(playlists.filter(id.eq(playlist_id_1)))
            .execute(conn)
            .map_err(|e| map_db_error(e, Critical))?;
        Ok(())
    }

    pub fn get_playlist_by_id(playlist_id_1: String) -> Result<Playlist, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        let res = playlists
            .filter(id.eq(playlist_id_1))
            .first::<Playlist>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res);
        }

        Err(CustomErrorInner::NotFound(Debug).into())
    }

    pub fn get_playlist_by_user_and_id(
        playlist_id_1: String,
        user: User,
    ) -> Result<Playlist, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        let res = playlists
            .filter(id.eq(playlist_id_1))
            .filter(user_id.eq(user.id))
            .first::<Playlist>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res);
        }

        Err(CustomErrorInner::NotFound(Debug).into())
    }

    pub fn get_playlists(user_id1: i32) -> Result<Vec<Playlist>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        playlists
            .filter(user_id.eq(user_id1))
            .load::<Playlist>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))
    }

    pub fn create_new_playlist(
        playlist_dto: PlaylistDtoPost,
        user: User,
    ) -> Result<Playlist, CustomError> {
        let playlist_to_insert = Playlist {
            id: Uuid::new_v4().to_string(),
            name: playlist_dto.name.clone(),
            user_id: user.id,
        };
        let inserted_playlist = playlist_to_insert.insert_playlist(&mut get_connection())?;

        playlist_dto.items.iter().enumerate().for_each(|(i, x)| {
            let playlist_item_to_insert = PlaylistItem {
                playlist_id: inserted_playlist.id.clone(),
                episode: x.episode,
                position: i as i32,
            };
            playlist_item_to_insert
                .insert_playlist_item(&mut get_connection())
                .expect("Error inserting playlist item");
        });
        Ok(inserted_playlist)
    }

    pub fn update_playlist_fields(
        playlist_id_1: String,
        playlist_dto: PlaylistDtoPost,
        user_id_1: User,
    ) -> Result<usize, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;
        diesel::update(
            playlists
                .filter(id.eq(playlist_id_1))
                .filter(user_id.eq(user_id_1.id)),
        )
        .set(name.eq(playlist_dto.name))
        .execute(&mut get_connection())
        .map_err(|e| map_db_error(e, Critical))
    }

    pub fn update_playlist(
        playlist_dto: PlaylistDtoPost,
        playlist_id: String,
        user: User,
    ) -> Result<Playlist, CustomError> {
        let playlist_to_be_updated = Self::get_playlist_by_id(playlist_id.clone())?;

        if playlist_to_be_updated.user_id != user.id {
            return Err(CustomErrorInner::Forbidden(Info).into());
        }

        Self::update_playlist_fields(playlist_id.clone(), playlist_dto.clone(), user.clone())?;

        // deletes all old playlist entries
        PlaylistItem::delete_playlist_item_by_playlist_id(playlist_id.clone())?;

        // inserts new playlist entries
        playlist_dto.items.iter().enumerate().for_each(|(i, x)| {
            let playlist_item_to_insert = PlaylistItem {
                playlist_id: playlist_id.clone(),
                episode: x.episode,
                position: i as i32,
            };
            playlist_item_to_insert
                .insert_playlist_item(&mut get_connection())
                .expect("Error inserting playlist item");
        });

        Self::get_playlist_by_id(playlist_id)
    }

    pub fn delete_playlist_by_id(playlist_id: String, user_id1: i32) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        let playlist_to_delete = Playlist::get_playlist_by_id(playlist_id.clone())?;

        if playlist_to_delete.user_id != user_id1 {
            return Err(CustomErrorInner::Forbidden(Info).into());
        }

        PlaylistItem::delete_playlist_item_by_playlist_id(playlist_id.clone())?;
        diesel::delete(
            playlists
                .filter(id.eq(playlist_id))
                .filter(user_id.eq(user_id1)),
        )
        .execute(&mut get_connection())
        .map_err(|e| map_db_error(e, Critical))?;
        Ok(())
    }

    pub fn delete_playlist_item(
        playlist_id_1: String,
        episode_id: i32,
        user_id: i32,
    ) -> Result<(), CustomError> {
        let found_podcast = Self::get_playlist_by_id(playlist_id_1.clone())?;

        if found_podcast.user_id != user_id {
            return Err(CustomErrorInner::Forbidden(Warning).into());
        }

        use crate::adapters::persistence::dbconfig::schema::playlist_items::dsl::*;

        diesel::delete(
            playlist_items
                .filter(playlist_id.eq(playlist_id))
                .filter(episode.eq(episode_id)),
        )
        .execute(&mut get_connection())
        .map_err(|e| map_db_error(e, Critical))?;
        Ok(())
    }
}
