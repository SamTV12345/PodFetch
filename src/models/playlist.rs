use crate::controllers::playlist_controller::{PlaylistDto, PlaylistDtoPost};
use crate::controllers::podcast_episode_controller::PodcastEpisodeWithHistory;
use crate::adapters::persistence::dbconfig::schema::playlists;
use crate::models::episode::Episode;
use crate::models::playlist_item::PlaylistItem;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::user::User;
use crate::utils::error::{map_db_error, CustomError};
use crate::{execute_with_conn, DBType as DbConnection};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{Queryable, QueryableByName};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::adapters::persistence::dbconfig::db::get_connection;

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
            .map_err(map_db_error)?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res);
        }
        
        
        execute_with_conn!(|conn| diesel::insert_into(playlists)
            .values(self)
            .get_result::<Playlist>(conn)
            .map_err(map_db_error))
    }

    pub fn delete_playlist(
        playlist_id_1: String,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        diesel::delete(playlists.filter(id.eq(playlist_id_1)))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_playlist_by_id(
        playlist_id_1: String,
    ) -> Result<Playlist, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        let res = playlists
            .filter(id.eq(playlist_id_1))
            .first::<Playlist>(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res);
        }

        Err(CustomError::NotFound)
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
            .map_err(map_db_error)?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res);
        }

        Err(CustomError::NotFound)
    }

    pub fn get_playlists(
        user_id1: i32,
    ) -> Result<Vec<Playlist>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        playlists
            .filter(user_id.eq(user_id1))
            .load::<Playlist>(&mut get_connection())
            .map_err(map_db_error)
    }

    pub fn create_new_playlist(
        playlist_dto: PlaylistDtoPost,
        user: User,
    ) -> Result<PlaylistDto, CustomError> {
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

        let items = PlaylistItem::get_playlist_items_by_playlist_id(
            inserted_playlist.id.clone(),
            user,
        )?;
        let playlist_dto_returned = inserted_playlist.to_playlist_dto(items);
        Ok(playlist_dto_returned)
    }

    fn to_playlist_dto(
        &self,
        item: Vec<(PlaylistItem, PodcastEpisode, Option<Episode>)>,
    ) -> PlaylistDto {
        let item = item
            .iter()
            .map(|(_, y, z)| PodcastEpisodeWithHistory {
                podcast_episode: y.clone(),
                podcast_history_item: z.clone(),
            })
            .collect::<Vec<PodcastEpisodeWithHistory>>();

        PlaylistDto {
            id: self.id.clone(),
            name: self.name.clone(),
            items: item,
        }
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
        .map_err(map_db_error)
    }

    pub fn update_playlist(
        playlist_dto: PlaylistDtoPost,
        playlist_id: String,
        user: User,
    ) -> Result<PlaylistDto, CustomError> {
        let playlist_to_be_updated = Self::get_playlist_by_id(playlist_id.clone())?;

        if playlist_to_be_updated.user_id != user.id {
            return Err(CustomError::Forbidden);
        }

        Self::update_playlist_fields(
            playlist_id.clone(),
            playlist_dto.clone(),
            user.clone(),
        )?;

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

        let updated_playlist = Self::get_playlist_by_id(playlist_id.clone())?;
        Self::get_playlist_dto(playlist_id, updated_playlist, user)
    }

    pub(crate) fn get_playlist_dto(
        playlist_id: String,
        playlist: Playlist,
        user: User,
    ) -> Result<PlaylistDto, CustomError> {
        if playlist.user_id != user.id {
            return Err(CustomError::Forbidden);
        }
        let items_in_playlist =
            PlaylistItem::get_playlist_items_by_playlist_id(playlist_id, user)?;

        Ok(playlist.to_playlist_dto(items_in_playlist))
    }

    pub fn delete_playlist_by_id(
        playlist_id: String,
        user_id1: i32,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlists::dsl::*;

        let playlist_to_delete = Playlist::get_playlist_by_id(playlist_id.clone())?;

        if playlist_to_delete.user_id != user_id1 {
            return Err(CustomError::Forbidden);
        }

        PlaylistItem::delete_playlist_item_by_playlist_id(playlist_id.clone())?;
        diesel::delete(
            playlists
                .filter(id.eq(playlist_id))
                .filter(user_id.eq(user_id1)),
        )
        .execute(&mut get_connection())
        .map_err(map_db_error)?;
        Ok(())
    }

    pub async fn delete_playlist_item(
        playlist_id_1: String,
        episode_id: i32,
        user_id: i32,
    ) -> Result<(), CustomError> {
        let found_podcast = Self::get_playlist_by_id(playlist_id_1.clone())?;

        if found_podcast.user_id != user_id {
            return Err(CustomError::Forbidden);
        }

        use crate::adapters::persistence::dbconfig::schema::playlist_items::dsl::*;

        diesel::delete(
            playlist_items
                .filter(playlist_id.eq(playlist_id))
                .filter(episode.eq(episode_id)),
        )
        .execute(&mut get_connection())
        .map_err(map_db_error)?;
        Ok(())
    }
}
