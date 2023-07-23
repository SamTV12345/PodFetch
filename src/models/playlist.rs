use diesel::{Queryable, QueryableByName};
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text};
use crate::dbconfig::schema::playlists;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use crate::DbConnection;
use crate::utils::error::{CustomError, map_db_error};
use diesel::RunQueryDsl;
use uuid::Uuid;
use crate::controllers::playlist_controller::{PlaylistDto, PlaylistDtoPost};
use crate::models::playlist_item::PlaylistItem;
use crate::models::podcast_episode::PodcastEpisode;

#[derive(Serialize, Deserialize, Queryable,Insertable, QueryableByName, Clone, ToSchema)]
pub struct Playlist {
    #[diesel(sql_type = Text)]
    pub id : String,
    #[diesel(sql_type = Text)]
    pub name : String,
    #[diesel(sql_type = Integer)]
    pub user_id: i32
}

impl Playlist {
    pub fn insert_playlist(&self, conn: &mut DbConnection) -> Result<Playlist, CustomError> {
        use crate::dbconfig::schema::playlists::dsl::*;

        let res = playlists.filter(name.eq(self.clone().name))
            .first::<Playlist>(conn)
            .optional()
            .map_err(map_db_error)?;

        if res.is_some() {
            return Ok(res.unwrap())
        }

        Ok(diesel::insert_into(playlists)
            .values(self)
            .get_result::<Playlist>(conn)
            .map_err(map_db_error)?)
    }

    pub fn delete_playlist(playlist_id_1: String, conn: &mut DbConnection) -> Result<(),
        CustomError> {
        use crate::dbconfig::schema::playlists::dsl::*;

        diesel::delete(playlists.filter(id.eq(playlist_id_1)))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_playlist_by_id(playlist_id_1: String, conn: &mut DbConnection) -> Result<Playlist,
        CustomError> {
        use crate::dbconfig::schema::playlists::dsl::*;

        let res = playlists.filter(id.eq(playlist_id_1))
            .first::<Playlist>(conn)
            .optional()
            .map_err(map_db_error)?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res)
        }

        Err(CustomError::NotFound)
    }

    pub fn get_playlists(conn: &mut DbConnection) -> Result<Vec<Playlist>, CustomError> {
        use crate::dbconfig::schema::playlists::dsl::*;

        Ok(playlists.load::<Playlist>(conn)
            .map_err(map_db_error)?)
    }

    pub fn create_new_playlist(conn: &mut DbConnection, playlist_dto: PlaylistDtoPost, user_id: i32)
                               -> Result<PlaylistDto, CustomError> {
        let playlist_to_insert = Playlist {
            id: Uuid::new_v4().to_string(),
            name: playlist_dto.name.clone(),
            user_id,
        };
        let inserted_playlist = playlist_to_insert.insert_playlist(conn)?;

        playlist_dto.items.iter().enumerate().for_each(|(i, x)| {
            let playlist_item_to_insert = PlaylistItem {
                playlist_id: inserted_playlist.id.clone(),
                episode: x.episode,
                position: i as i32
            };
            playlist_item_to_insert.insert_playlist_item(conn).expect("Error inserting playlist item");
        });

        let items = PlaylistItem::get_playlist_items_by_playlist_id(inserted_playlist.id.clone(),
                                                                conn)?;
        let playlist_dto_returned = inserted_playlist.to_playlist_dto(items, conn);
        Ok(playlist_dto_returned)
    }

    fn to_playlist_dto(self, item: Vec<PlaylistItem>, conn: &mut DbConnection) -> PlaylistDto {
        let episodes = item.iter().map(|x| PodcastEpisode::get_podcast_episode_by_internal_id
            (conn, x.episode).expect("").expect("")).collect::<Vec<PodcastEpisode>>();

        PlaylistDto {
            id: self.id,
            name: self.name,
            items: episodes
        }
    }

    pub fn update_playlist_fields(playlist_id_1: String, playlist_dto: PlaylistDtoPost, conn: &mut
    DbConnection, user_id_1: i32) ->
                                  Result<usize, CustomError> {
        use crate::dbconfig::schema::playlists::dsl::*;
        Ok(diesel::update(playlists
            .filter(id.eq(playlist_id_1))
            .filter(user_id.eq(user_id_1)))
            .set(
                name.eq(playlist_dto.name)
            )
            .execute(conn)
            .map_err(map_db_error)?)
    }

    pub fn update_playlist(conn: &mut DbConnection, playlist_dto: PlaylistDtoPost, playlist_id: String,
                           user_id: i32)
                           -> Result<PlaylistDto, CustomError> {
        let playlist_to_be_updated = Self::get_playlist_by_id(playlist_id.clone(), conn)?;

        if playlist_to_be_updated.user_id != user_id {
            return Err(CustomError::Forbidden)
        }

        Self::update_playlist_fields(playlist_id.clone(), playlist_dto.clone(),
                                     conn, user_id)?;

        // deletes all old playlist entries
        PlaylistItem::delete_playlist_item_by_playlist_id(playlist_id.clone(), conn)?;

        // inserts new playlist entries
        playlist_dto.items.iter().enumerate().for_each(|(i, x)| {
            let playlist_item_to_insert = PlaylistItem {
                playlist_id: playlist_id.clone(),
                episode: x.episode,
                position: i as i32
            };
            playlist_item_to_insert.insert_playlist_item(conn).expect("Error inserting playlist item");
        });


        Self::get_podcast_dto(playlist_id, conn, playlist_to_be_updated)
    }

    fn get_podcast_dto(playlist_id: String, conn: &mut DbConnection, playlist: Playlist) ->
                                                                                         Result<PlaylistDto, CustomError> {
        let items_in_playlist = PlaylistItem::get_playlist_items_by_playlist_id(playlist_id, conn)?;

        Ok(playlist.to_playlist_dto(items_in_playlist, conn))
    }
}