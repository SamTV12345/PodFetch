use crate::adapters::persistence::dbconfig::db::Database;
use crate::adapters::persistence::dbconfig::schema::tags;
use crate::adapters::persistence::dbconfig::schema::tags::dsl as tags_dsl;
use crate::adapters::persistence::dbconfig::schema::tags_podcasts;
use crate::adapters::persistence::dbconfig::schema::tags_podcasts::dsl as tags_podcasts_dsl;
use crate::models::tag::Tag;
use crate::models::tags_podcast::TagsPodcast;
use crate::utils::error::ErrorSeverity::Critical;
use crate::utils::error::{CustomError, map_db_error};
use diesel::BoolExpressionMethods;
use diesel::OptionalExtension;
use diesel::RunQueryDsl;
use diesel::{ExpressionMethods, QueryDsl};

pub trait TagRepository: Send + Sync {
    fn create(&self, tag: Tag) -> Result<Tag, CustomError>;
    fn get_tags(&self, username: &str) -> Result<Vec<Tag>, CustomError>;
    fn get_tag_by_id_and_username(
        &self,
        tag_id: &str,
        username: &str,
    ) -> Result<Option<Tag>, CustomError>;
    fn update(&self, tag_id: &str, update: TagUpdate) -> Result<Tag, CustomError>;
    fn delete(&self, tag_id: &str) -> Result<(), CustomError>;
    fn add_podcast_to_tag(
        &self,
        tag_id_to_insert: String,
        podcast_id_to_insert: i32,
    ) -> Result<TagsPodcast, CustomError>;
    fn delete_tag_podcasts(&self, tag_id: &str) -> Result<(), CustomError>;
    fn delete_tag_podcasts_by_podcast_id_tag_id(
        &self,
        podcast_id: i32,
        tag_id: &str,
    ) -> Result<(), CustomError>;
}

pub struct DieselTagRepository {
    database: Database,
}

impl DieselTagRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[derive(Clone)]
pub struct TagUpdate {
    pub name: String,
    pub description: Option<String>,
    pub color: String,
}

impl TagRepository for DieselTagRepository {
    fn create(&self, tag: Tag) -> Result<Tag, CustomError> {
        let mut conn = self.database.connection()?;

        diesel::insert_into(tags::table)
            .values(&tag)
            .get_result(&mut conn)
            .map_err(|e| map_db_error(e, Critical))
    }

    fn get_tags(&self, username: &str) -> Result<Vec<Tag>, CustomError> {
        let mut conn = self.database.connection()?;

        tags_dsl::tags
            .filter(tags_dsl::username.eq(username))
            .load::<Tag>(&mut conn)
            .map_err(|e| map_db_error(e, Critical))
    }

    fn get_tag_by_id_and_username(
        &self,
        tag_id: &str,
        username: &str,
    ) -> Result<Option<Tag>, CustomError> {
        let mut conn = self.database.connection()?;

        tags_dsl::tags
            .filter(tags_dsl::id.eq(tag_id))
            .filter(tags_dsl::username.eq(username))
            .first::<Tag>(&mut conn)
            .optional()
            .map_err(|e| map_db_error(e, Critical))
    }

    fn update(&self, tag_id: &str, update: TagUpdate) -> Result<Tag, CustomError> {
        let mut conn = self.database.connection()?;

        diesel::update(tags_dsl::tags.filter(tags_dsl::id.eq(tag_id)))
            .set((
                tags_dsl::name.eq(update.name),
                tags_dsl::description.eq(update.description),
                tags_dsl::color.eq(update.color),
            ))
            .get_result::<Tag>(&mut conn)
            .map_err(|e| map_db_error(e, Critical))
    }

    fn delete(&self, tag_id: &str) -> Result<(), CustomError> {
        let mut conn = self.database.connection()?;

        diesel::delete(tags_dsl::tags.filter(tags_dsl::id.eq(tag_id)))
            .execute(&mut conn)
            .map(|_| ())
            .map_err(|e| map_db_error(e, Critical))
    }

    fn add_podcast_to_tag(
        &self,
        tag_id_to_insert: String,
        podcast_id_to_insert: i32,
    ) -> Result<TagsPodcast, CustomError> {
        let mut conn = self.database.connection()?;
        let new_tag_podcast = TagsPodcast {
            tag_id: tag_id_to_insert,
            podcast_id: podcast_id_to_insert,
        };

        diesel::insert_into(tags_podcasts::table)
            .values(&new_tag_podcast)
            .get_result(&mut conn)
            .map_err(|e| map_db_error(e, Critical))
    }

    fn delete_tag_podcasts(&self, tag_id: &str) -> Result<(), CustomError> {
        let mut conn = self.database.connection()?;

        diesel::delete(
            tags_podcasts::table.filter(tags_podcasts_dsl::tag_id.eq(tag_id)),
        )
        .execute(&mut conn)
        .map(|_| ())
        .map_err(|e| map_db_error(e, Critical))
    }

    fn delete_tag_podcasts_by_podcast_id_tag_id(
        &self,
        podcast_id: i32,
        tag_id: &str,
    ) -> Result<(), CustomError> {
        let mut conn = self.database.connection()?;

        diesel::delete(
            tags_podcasts::table.filter(
                tags_podcasts_dsl::podcast_id
                    .eq(podcast_id)
                    .and(tags_podcasts_dsl::tag_id.eq(tag_id)),
            ),
        )
        .execute(&mut conn)
        .map(|_| ())
        .map_err(|e| map_db_error(e, Critical))
    }
}
