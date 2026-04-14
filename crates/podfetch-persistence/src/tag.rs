use crate::db::{Database, PersistenceError};
use diesel::BoolExpressionMethods;
use diesel::JoinOnDsl;
use diesel::OptionalExtension;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_domain::tag::{Tag, TagRepository, TagUpdate, TagsPodcast};

diesel::table! {
    tags (id) {
        id -> Text,
        name -> Text,
        user_id -> Integer,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        color -> Text,
    }
}

diesel::table! {
    tags_podcasts (tag_id, podcast_id) {
        tag_id -> Text,
        podcast_id -> Integer,
    }
}

diesel::joinable!(tags_podcasts -> tags (tag_id));
diesel::allow_tables_to_appear_in_same_query!(tags, tags_podcasts);

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = tags)]
struct TagEntity {
    id: String,
    name: String,
    user_id: i32,
    description: Option<String>,
    created_at: chrono::NaiveDateTime,
    color: String,
}

#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = tags_podcasts)]
struct TagsPodcastEntity {
    tag_id: String,
    podcast_id: i32,
}

impl From<TagEntity> for Tag {
    fn from(value: TagEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            user_id: value.user_id,
            description: value.description,
            created_at: value.created_at,
            color: value.color,
        }
    }
}

impl From<Tag> for TagEntity {
    fn from(value: Tag) -> Self {
        Self {
            id: value.id,
            name: value.name,
            user_id: value.user_id,
            description: value.description,
            created_at: value.created_at,
            color: value.color,
        }
    }
}

impl From<TagsPodcastEntity> for TagsPodcast {
    fn from(value: TagsPodcastEntity) -> Self {
        Self {
            tag_id: value.tag_id,
            podcast_id: value.podcast_id,
        }
    }
}

pub struct DieselTagRepository {
    database: Database,
}

impl DieselTagRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl TagRepository for DieselTagRepository {
    type Error = PersistenceError;

    fn create(&self, tag: Tag) -> Result<Tag, Self::Error> {
        diesel::insert_into(tags::table)
            .values(TagEntity::from(tag))
            .get_result::<TagEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn get_tags(&self, user_id_to_find: i32) -> Result<Vec<Tag>, Self::Error> {
        use self::tags::dsl as tags_dsl;

        tags_dsl::tags
            .filter(tags_dsl::user_id.eq(user_id_to_find))
            .load::<TagEntity>(&mut self.database.connection()?)
            .map(|tags| tags.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_tags_of_podcast(
        &self,
        podcast_id: i32,
        user_id_to_find: i32,
    ) -> Result<Vec<Tag>, Self::Error> {
        use self::tags::dsl as tags_dsl;
        use self::tags_podcasts::dsl as tags_podcasts_dsl;

        tags_dsl::tags
            .inner_join(tags_podcasts::table.on(tags_dsl::id.eq(tags_podcasts_dsl::tag_id)))
            .select(tags::all_columns)
            .filter(tags_podcasts_dsl::podcast_id.eq(podcast_id))
            .filter(tags_dsl::user_id.eq(user_id_to_find))
            .load::<TagEntity>(&mut self.database.connection()?)
            .map(|tags: Vec<TagEntity>| tags.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_tag_by_id_and_user_id(
        &self,
        tag_id: &str,
        user_id_to_find: i32,
    ) -> Result<Option<Tag>, Self::Error> {
        use self::tags::dsl as tags_dsl;

        tags_dsl::tags
            .filter(tags_dsl::id.eq(tag_id))
            .filter(tags_dsl::user_id.eq(user_id_to_find))
            .first::<TagEntity>(&mut self.database.connection()?)
            .optional()
            .map(|tag| tag.map(Into::into))
            .map_err(Into::into)
    }

    fn update(&self, tag_id: &str, update: TagUpdate) -> Result<Tag, Self::Error> {
        use self::tags::dsl as tags_dsl;

        diesel::update(tags_dsl::tags.filter(tags_dsl::id.eq(tag_id)))
            .set((
                tags_dsl::name.eq(update.name),
                tags_dsl::description.eq(update.description),
                tags_dsl::color.eq(update.color),
            ))
            .get_result::<TagEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn delete(&self, tag_id: &str) -> Result<(), Self::Error> {
        use self::tags::dsl as tags_dsl;

        diesel::delete(tags_dsl::tags.filter(tags_dsl::id.eq(tag_id)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn add_podcast_to_tag(
        &self,
        tag_id_to_insert: String,
        podcast_id_to_insert: i32,
    ) -> Result<TagsPodcast, Self::Error> {
        diesel::insert_into(tags_podcasts::table)
            .values(TagsPodcastEntity {
                tag_id: tag_id_to_insert,
                podcast_id: podcast_id_to_insert,
            })
            .get_result::<TagsPodcastEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn delete_tag_podcasts(&self, tag_id: &str) -> Result<(), Self::Error> {
        use self::tags_podcasts::dsl as tags_podcasts_dsl;

        diesel::delete(tags_podcasts::table.filter(tags_podcasts_dsl::tag_id.eq(tag_id)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete_tag_podcasts_by_podcast_id_tag_id(
        &self,
        podcast_id: i32,
        tag_id: &str,
    ) -> Result<(), Self::Error> {
        use self::tags_podcasts::dsl as tags_podcasts_dsl;

        diesel::delete(
            tags_podcasts::table.filter(
                tags_podcasts_dsl::podcast_id
                    .eq(podcast_id)
                    .and(tags_podcasts_dsl::tag_id.eq(tag_id)),
            ),
        )
        .execute(&mut self.database.connection()?)
        .map(|_| ())
        .map_err(Into::into)
    }

    fn delete_tag_podcasts_by_podcast_id(&self, podcast_id: i32) -> Result<(), Self::Error> {
        use self::tags_podcasts::dsl as tags_podcasts_dsl;

        diesel::delete(tags_podcasts::table.filter(tags_podcasts_dsl::podcast_id.eq(podcast_id)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }
}
