use diesel::{Queryable, QueryableByName};
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use crate::dbconfig::schema::playlist_items;
use crate::DbConnection;
use crate::utils::error::{CustomError, map_db_error};

#[derive(Serialize, Deserialize, Debug,Queryable, QueryableByName,Insertable, Clone, ToSchema)]
pub struct PlaylistItem {
    #[diesel(sql_type = Text)]
    pub playlist_id: String,
    #[diesel(sql_type = Integer)]
    pub episode : i32,
    #[diesel(sql_type = Integer)]
    pub position : i32
}


impl PlaylistItem {
    pub fn insert_playlist_item(&self, conn: &mut DbConnection) -> Result<PlaylistItem, CustomError> {
        use crate::dbconfig::schema::playlist_items::dsl::*;

        let res = playlist_items.filter(playlist_id.eq(self.clone().playlist_id)
            .and(episode.eq(self.clone().episode)))
            .first::<PlaylistItem>(conn)
            .optional()
            .map_err(map_db_error)?;

        if res.is_some() {
            return Ok(res.unwrap())
        }

        Ok(diesel::insert_into(playlist_items)
            .values((
                playlist_id.eq(&self.playlist_id),
                episode.eq(&self.episode),
                position.eq(&self.position)
            ))
            .get_result::<PlaylistItem>(conn)
            .map_err(map_db_error)?)
    }

    pub fn delete_playlist_item(playlist_id_1: String, episode_1: i32, conn: &mut DbConnection) ->
                                                                                           Result<(), diesel::result::Error> {
        use crate::dbconfig::schema::playlist_items::dsl::*;

        diesel::delete(playlist_items.filter(playlist_id.eq(playlist_id_1).and(episode.eq(episode_1)))).execute(conn)?;
        Ok(())
    }

    pub fn delete_playlist_item_by_playlist_id(playlist_id_1: String, conn: &mut DbConnection) ->
                                                                                               Result<(), CustomError> {
        use crate::dbconfig::schema::playlist_items::dsl::*;

        diesel::delete(playlist_items.filter(playlist_id.eq(playlist_id_1))).execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_playlist_items_by_playlist_id(playlist_id_1: String, conn: &mut DbConnection) ->
                                                                                             Result<Vec<PlaylistItem>, CustomError> {
        use crate::dbconfig::schema::playlist_items::dsl::*;

        Ok(playlist_items.filter(playlist_id.eq(playlist_id_1))
            .order(position.asc())
            .load::<PlaylistItem>(conn)
            .map_err(map_db_error)?)
    }
}