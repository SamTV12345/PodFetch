use crate::dbconfig::schema::*;

use crate::models::favorites::Favorite;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::tag::Tag;
use crate::service::mapping_service::MappingService;
use crate::utils::do_retry::do_retry;
use crate::utils::podcast_builder::PodcastExtra;
use crate::DBType as DbConnection;
use diesel::prelude::{Identifiable, Queryable, QueryableByName, Selectable};
use diesel::sql_types::{Bool, Integer, Nullable, Text};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{
    delete, insert_into, BoolExpressionMethods, JoinOnDsl, OptionalExtension, RunQueryDsl,
};
use utoipa::ToSchema;

use crate::utils::error::{map_db_error, CustomError};
#[derive(
    Queryable,
    Identifiable,
    QueryableByName,
    Selectable,
    Debug,
    PartialEq,
    Clone,
    ToSchema,
    Serialize,
    Deserialize,
    Default,
)]
pub struct Podcast {
    #[diesel(sql_type = Integer)]
    pub(crate) id: i32,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = Text)]
    pub directory_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) rssfeed: String,
    #[diesel(sql_type = Text)]
    pub image_url: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub summary: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub language: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub explicit: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub keywords: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub last_build_date: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub author: Option<String>,
    #[diesel(sql_type = Bool)]
    pub active: bool,
    #[diesel(sql_type = Text)]
    pub original_image_url: String,
    #[diesel(sql_type = Text)]
    pub directory_name: String,
}

impl Podcast {
    pub fn find_by_rss_feed_url(conn: &mut DbConnection, feed_url: &str) -> Option<Podcast> {
        use crate::dbconfig::schema::podcasts::dsl::*;
        podcasts
            .filter(rssfeed.eq(feed_url))
            .first::<Podcast>(conn)
            .optional()
            .expect("Error loading podcast by rss feed url")
    }

    pub fn get_podcasts(
        conn: &mut DbConnection,
        u: String,
    ) -> Result<Vec<PodcastDto>, CustomError> {
        use crate::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::dbconfig::schema::favorites::dsl::podcast_id as f_id;
        use crate::dbconfig::schema::favorites::dsl::username;
        use crate::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::dbconfig::schema::podcasts::id as p_id;
        let result = podcasts
            .left_join(f_db.on(username.eq(&u).and(f_id.eq(p_id))))
            .load::<(Podcast, Option<Favorite>)>(conn)
            .map_err(map_db_error)?;

        let mapped_result = result
            .iter()
            .map(|podcast| {
                let tags = Tag::get_tags_of_podcast(conn, podcast.0.id, &u).unwrap();
                MappingService::map_podcast_to_podcast_dto_with_favorites(podcast, tags)
            })
            .collect::<Vec<PodcastDto>>();
        Ok(mapped_result)
    }

    pub fn get_all_podcasts(conn: &mut DbConnection) -> Result<Vec<Podcast>, CustomError> {
        use crate::dbconfig::schema::podcasts::dsl::podcasts;
        let result = podcasts.load::<Podcast>(conn).map_err(map_db_error)?;
        Ok(result)
    }

    pub fn get_podcast(
        conn: &mut DbConnection,
        podcast_id_to_be_found: i32,
    ) -> Result<Podcast, CustomError> {
        use crate::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::dbconfig::schema::podcasts::id as podcast_id;
        let found_podcast = podcasts
            .filter(podcast_id.eq(podcast_id_to_be_found))
            .first::<Podcast>(conn)
            .optional()
            .map_err(map_db_error)?;

        match found_podcast {
            Some(podcast) => Ok(podcast),
            None => Err(CustomError::NotFound),
        }
    }

    pub fn delete_podcast(
        conn: &mut DbConnection,
        podcast_id_to_find: i32,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::podcasts::dsl::*;
        let _ = delete(podcasts.filter(id.eq(podcast_id_to_find)))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_podcast_by_track_id(
        conn: &mut DbConnection,
        podcast_id: i32,
    ) -> Result<Option<Podcast>, CustomError> {
        use crate::dbconfig::schema::podcasts::directory_id;
        use crate::dbconfig::schema::podcasts::dsl::podcasts;
        let optional_podcast = podcasts
            .filter(directory_id.eq(podcast_id.to_string()))
            .first::<Podcast>(conn)
            .optional()
            .map_err(map_db_error)?;

        Ok(optional_podcast)
    }

    pub fn add_podcast_to_database(
        conn: &mut DbConnection,
        collection_name: String,
        collection_id: String,
        feed_url: String,
        image_url_1: String,
        directory_name_to_insert: String,
    ) -> Result<Podcast, CustomError> {
        use crate::dbconfig::schema::podcasts::{
            directory_id, image_url, name as podcast_name, rssfeed,
        };
        use crate::dbconfig::schema::podcasts::{directory_name, original_image_url};

        let inserted_podcast = insert_into(podcasts::table)
            .values((
                directory_id.eq(collection_id.to_string()),
                podcast_name.eq(collection_name.to_string()),
                rssfeed.eq(feed_url.to_string()),
                image_url.eq(image_url_1.to_string()),
                original_image_url.eq(image_url_1.to_string()),
                directory_name.eq(directory_name_to_insert.to_string()),
            ))
            .get_result::<Podcast>(conn)
            .map_err(map_db_error)?;
        Ok(inserted_podcast)
    }

    pub fn get_podcast_by_rss_feed(
        rss_feed_1: String,
        conn: &mut DbConnection,
    ) -> Result<Podcast, CustomError> {
        use crate::dbconfig::schema::podcasts::dsl::*;

        podcasts
            .filter(rssfeed.eq(rss_feed_1))
            .first::<Podcast>(conn)
            .map_err(map_db_error)
    }

    pub fn get_podcast_by_directory_id(
        podcast_id: &str,
        conn: &mut DbConnection,
    ) -> Result<Option<Podcast>, CustomError> {
        use crate::dbconfig::schema::podcasts::dsl::directory_id;
        use crate::dbconfig::schema::podcasts::dsl::podcasts as dsl_podcast;
        let result = dsl_podcast
            .filter(directory_id.eq(podcast_id))
            .first::<Podcast>(conn)
            .optional()
            .map_err(map_db_error)?;
        Ok(result)
    }

    pub fn query_for_podcast(
        query: &str,
        conn: &mut DbConnection,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use diesel::TextExpressionMethods;
        let result = podcast_episodes
            .filter(
                name.like(format!("%{}%", query))
                    .or(description.like(format!("%{}%", query))),
            )
            .load::<PodcastEpisode>(conn)
            .map_err(map_db_error)?;
        Ok(result)
    }

    pub fn update_podcast_fields(
        podcast_extra: PodcastExtra,
        conn: &mut DbConnection,
    ) -> Result<usize, CustomError> {
        use crate::dbconfig::schema::podcasts::dsl::*;

        do_retry(|| {
            diesel::update(podcasts)
                .filter(id.eq(podcast_extra.clone().id))
                .set((
                    author.eq(podcast_extra.clone().author),
                    keywords.eq(podcast_extra.clone().keywords),
                    explicit.eq(podcast_extra.clone().explicit.to_string()),
                    language.eq(podcast_extra.clone().language),
                    summary.eq(podcast_extra.clone().description),
                    last_build_date.eq(podcast_extra.clone().last_build_date),
                ))
                .execute(conn)
        })
        .map_err(map_db_error)
    }

    pub fn update_podcast_active(
        conn: &mut DbConnection,
        podcast_id: i32,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::podcasts::dsl::*;

        let found_podcast = Podcast::get_podcast(conn, podcast_id)?;

        diesel::update(podcasts.filter(id.eq(podcast_id)))
            .set(active.eq(!found_podcast.active))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn update_original_image_url(
        original_image_url_to_set: &str,
        podcast_id_to_find: i32,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::podcasts::dsl::*;
        do_retry(|| {
            diesel::update(podcasts.filter(id.eq(podcast_id_to_find)))
                .set(original_image_url.eq(original_image_url_to_set))
                .execute(conn)
        })
        .map_err(map_db_error)?;
        Ok(())
    }

    pub fn update_podcast_urls_on_redirect(
        podcast_id_to_update: i32,
        new_url: String,
        conn: &mut DbConnection,
    ) {
        use crate::dbconfig::schema::podcasts::dsl::id as pid;
        use crate::dbconfig::schema::podcasts::dsl::*;

        diesel::update(podcasts.filter(pid.eq(podcast_id_to_update)))
            .set(rssfeed.eq(new_url))
            .execute(conn)
            .expect("Error updating podcast episode");
    }
}
