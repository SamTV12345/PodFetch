use chrono::{NaiveDateTime, Utc};
use diesel::{AsChangeset, Insertable, JoinOnDsl, OptionalExtension, Queryable, QueryableByName, QueryDsl, RunQueryDsl};
use utoipa::ToSchema;
use crate::dbconfig::DBType as DbConnection;
use crate::utils::error::{CustomError, map_db_error};
use diesel::sql_types::{Text,Nullable, Timestamp };

use crate::dbconfig::schema::tags;
use crate::execute_with_conn;
use crate::models::favorites::Favorite;
use crate::models::podcasts::Podcast;
use crate::models::tags_podcast::TagsPodcast;

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
    name: String,
    #[diesel(sql_type = Text)]
    username: String,
    #[diesel(sql_type = Nullable<Text>)]
    description: Option<String>,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
    #[diesel(sql_type = Text)]
    color: String,
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

    pub fn insert_tag(&self, conn: &mut DbConnection) -> Result<Tag, CustomError> {
        use crate::dbconfig::schema::tags::dsl::*;
        execute_with_conn!(conn,
                    |conn|diesel::insert_into(tags)
            .values(self)
            .get_result(conn)
            .map_err(map_db_error)
        )
    }

    pub fn delete_tag(conn: &mut DbConnection, tag_id: &str) -> Result<(), CustomError> {
        use crate::dbconfig::schema::tags::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        diesel::delete(tags.filter(id.eq(tag_id)))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_tag_by_id_and_username(conn: &mut DbConnection, tag_id: &str, username_to_search:
    &str) -> Result<Option<Tag>, CustomError> {
        use crate::dbconfig::schema::tags::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        tags.filter(id.eq(tag_id))
            .filter(username.eq(username_to_search))
            .first::<Tag>(conn)
            .optional()
            .map_err(map_db_error)
    }

    pub fn update_tag(conn: &mut DbConnection, tag_id: &str, name_new: String, description_new: Option<String>, color_new: String) -> Result<Tag, CustomError> {
        use crate::dbconfig::schema::tags::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        diesel::update(tags.filter(id.eq(tag_id)))
            .set((name.eq(name_new), description.eq(description_new), color.eq(color_new)))
            .get_result::<Tag>(conn)
            .map_err(map_db_error)
    }

    pub fn get_tags(conn: &mut DbConnection, username_to_search: String) -> Result<Vec<(Tag,
                                                                                        TagsPodcast, Podcast, Option<Favorite>)>, CustomError> {
        use crate::dbconfig::schema::tags::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;

        tags
            .inner_join(crate::dbconfig::schema::tags_podcasts::table.on(crate::dbconfig::schema::tags_podcasts::tag_id.eq(crate::dbconfig::schema::tags::id)))
            .inner_join(crate::dbconfig::schema::podcasts::table.on(crate::dbconfig::schema::podcasts::id.eq(crate::dbconfig::schema::tags_podcasts::podcast_id)))
            .left_join(crate::dbconfig::schema::favorites::table.on(crate::dbconfig::schema::favorites::podcast_id.eq(crate::dbconfig::schema::podcasts::id)))
            .filter(username.eq(username_to_search))
            .load::<(Tag, TagsPodcast, Podcast, Option<Favorite>)>(conn)
            .map_err(map_db_error)
    }
}
