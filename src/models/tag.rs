use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::tags;
use crate::execute_with_conn;
use crate::utils::error::ErrorSeverity::Critical;
use crate::utils::error::{map_db_error, CustomError};
use chrono::{NaiveDateTime, Utc};
use diesel::sql_types::{Nullable, Text, Timestamp};
use diesel::{
    AsChangeset, Insertable, JoinOnDsl, OptionalExtension, Queryable, QueryableByName, RunQueryDsl,
    Table,
};
use utoipa::ToSchema;

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
pub struct Tag {
    #[diesel(sql_type = Text)]
    pub(crate) id: String,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = Timestamp)]
    pub created_at: NaiveDateTime,
    #[diesel(sql_type = Text)]
    pub color: String,
}

impl Tag {
    pub fn new(name: String, description: Option<String>, color: String, username: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            username,
            description,
            created_at: Utc::now().naive_utc(),
            color,
        }
    }

    pub fn insert_tag(&self) -> Result<Tag, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;

        execute_with_conn!(|conn| diesel::insert_into(tags)
            .values(self)
            .get_result(conn)
            .map_err(|e| map_db_error(e, Critical)))
    }

    pub fn delete_tag(tag_id: &str) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        diesel::delete(tags.filter(id.eq(tag_id)))
            .execute(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        Ok(())
    }

    pub fn get_tag_by_id_and_username(
        tag_id: &str,
        username_to_search: &str,
    ) -> Result<Option<Tag>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        tags.filter(id.eq(tag_id))
            .filter(username.eq(username_to_search))
            .first::<Tag>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))
    }

    pub fn update_tag(
        tag_id: &str,
        name_new: String,
        description_new: Option<String>,
        color_new: String,
    ) -> Result<Tag, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        diesel::update(tags.filter(id.eq(tag_id)))
            .set((
                name.eq(name_new),
                description.eq(description_new),
                color.eq(color_new),
            ))
            .get_result::<Tag>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))
    }

    pub fn get_tags(username_to_search: String) -> Result<Vec<Tag>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        tags.filter(username.eq(username_to_search))
            .load::<Tag>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))
    }

    pub fn get_tags_of_podcast(
        podcast_id_to_search: i32,
        username_to_search: &str,
    ) -> Result<Vec<Tag>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::tags::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::tags_podcasts::dsl as t_podcasts_dsl;
        use crate::adapters::persistence::dbconfig::schema::tags_podcasts::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::tags_podcasts::table as t_podcasts;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        tags.inner_join(t_podcasts.on(id.eq(t_podcasts_dsl::tag_id)))
            .select(tags::all_columns())
            .filter(podcast_id.eq(podcast_id_to_search))
            .filter(username.eq(username_to_search))
            .load::<Tag>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))
    }
}
