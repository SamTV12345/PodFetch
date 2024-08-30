use diesel::{AsChangeset, Insertable, Queryable, QueryableByName};
use utoipa::ToSchema;
use crate::dbconfig::DBType as DbConnection;
use crate::dbconfig::schema::tags_podcasts;
use crate::{execute_with_conn, insert_with_conn};
use crate::utils::error::{CustomError, map_db_error};

#[derive(
Debug,
Serialize,
Deserialize,
QueryableByName,
Queryable,
AsChangeset,
Insertable,
Clone,
ToSchema,
)]
#[diesel(treat_none_as_null = true)]
pub struct TagsPodcast {
    tag_id: String,
    podcast_id: i32
}


impl TagsPodcast {
    pub fn add_podcast_to_tag(tag_id_to_insert: String, podcast_id_to_insert: i32, conn: &mut DbConnection) ->
                                                                                        Result<TagsPodcast, CustomError> {
use crate::dbconfig::schema::tags_podcasts::dsl::*;
        use diesel::RunQueryDsl;
        let new_tag_podcast = TagsPodcast {
            tag_id: tag_id_to_insert,
            podcast_id: podcast_id_to_insert
        };
        execute_with_conn!(conn, |conn|        diesel::insert_into(tags_podcasts)
            .values(&new_tag_podcast)
            .get_result(conn)
            .map_err(map_db_error))
    }

    pub fn delete_tags_by_podcast_id(conn: &mut DbConnection, podcast_id_to_delete: i32) -> Result<(),
        CustomError> {
        use crate::dbconfig::schema::tags_podcasts::dsl::*;
        use crate::dbconfig::schema::tags_podcasts::table as t_podcasts;
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        let _ = insert_with_conn!(conn, |conn| diesel::delete(t_podcasts.filter(podcast_id.eq(podcast_id_to_delete)))
            .execute(conn)
            .map_err(map_db_error));
        Ok(())
    }

    pub fn delete_tag_podcasts(conn: &mut DbConnection, tag_id_to_delete: &str) -> Result<(),
        CustomError> {
        use crate::dbconfig::schema::tags_podcasts::dsl::*;
        use crate::dbconfig::schema::tags_podcasts::table as t_podcasts;
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        let _ = insert_with_conn!(conn, |conn| diesel::delete(t_podcasts.filter(tag_id.eq(tag_id_to_delete)))
            .execute(conn)
            .map_err(map_db_error));
        Ok(())
    }

    pub fn delete_tag_podcasts_by_podcast_id_tag_id(conn: &mut DbConnection,
                                                    podcast_id_to_delete: i32, tag_id_key: &str) ->
                                                                                                                                           Result<(),
        CustomError> {
        use crate::dbconfig::schema::tags_podcasts::dsl::*;
        use crate::dbconfig::schema::tags_podcasts::table as t_podcasts;
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::BoolExpressionMethods;
        let _  = insert_with_conn!(conn, |conn| diesel::delete(t_podcasts.filter(podcast_id.eq
            (podcast_id_to_delete).and(tag_id.eq(tag_id_key))))
            .execute(conn)
            .map_err(map_db_error));
        Ok(())
    }
}