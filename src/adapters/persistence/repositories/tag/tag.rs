use diesel::{JoinOnDsl, OptionalExtension, RunQueryDsl, Table};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::model::tag::tag::TagEntity;
use crate::domain::models::tag::tag::Tag;
use crate::execute_with_conn;
use crate::utils::error::{map_db_error, CustomError};

pub struct TagRepositoryImpl;


impl TagRepositoryImpl {
    pub fn insert_tag(tag: Tag) -> Result<Tag, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        let tag_entity = tag.into();

        execute_with_conn!(
                    |conn|diesel::insert_into(tags)
            .values(tag_entity)
            .get_result::<TagEntity>(conn)
            .map_err(map_db_error)
            .map(|e| e.into())
        )
    }

    pub fn delete_tag(tag_id: &str) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        diesel::delete(tags.filter(id.eq(tag_id)))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_tag_by_id_and_username(tag_id: &str, username_to_search:
    &str) -> Result<Option<Tag>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        tags.filter(id.eq(tag_id))
            .filter(username.eq(username_to_search))
            .first::<TagEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
            .map(|e|e.map(|e|e.into()))
    }

    pub fn update_tag(tag_id: &str, name_new: String, description_new: Option<String>, color_new:
    String) -> Result<Tag, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        diesel::update(tags.filter(id.eq(tag_id)))
            .set((name.eq(name_new), description.eq(description_new), color.eq(color_new)))
            .get_result::<TagEntity>(&mut get_connection())
            .map_err(map_db_error)
            .map(|e| e.into())
            .map(|e| e.into())
    }

    pub fn get_tags(username_to_search: String) -> Result<Vec<Tag>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        tags
            .filter(username.eq(username_to_search))
            .load::<TagEntity>(&mut get_connection())
            .map_err(map_db_error)
            .map(|e| e.into_iter().map(|e| e.into()).collect())
    }

    pub fn get_tags_of_podcast(podcast_id_to_search: i32,
                               username_to_search: &str) -> Result<Vec<Tag>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::tags_podcasts::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::tags_podcasts::table as t_podcasts;
        use crate::adapters::persistence::dbconfig::schema::tags_podcasts::dsl as t_podcasts_dsl;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        tags
            .inner_join(t_podcasts.on(id.eq(t_podcasts_dsl::tag_id)))
            .select(tags::all_columns())
            .filter(podcast_id.eq(podcast_id_to_search))
            .filter(username.eq(username_to_search))
            .load::<TagEntity>(&mut get_connection())
            .map_err(map_db_error)
            .map(|e| e.into_iter().map(|e| e.into()).collect())
    }
}