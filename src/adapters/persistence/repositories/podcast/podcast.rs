use diesel::{delete, insert_into, BoolExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::podcasts;
use crate::adapters::persistence::model::favorite::favorites::FavoriteEntity;
use crate::adapters::persistence::model::podcast::podcast::PodcastEntity;
use crate::domain::models::podcast::podcast::Podcast;
use crate::domain::models::tag::tag::Tag;
use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};
use crate::utils::podcast_builder::PodcastExtra;

pub struct PodcastRepositoryImpl;

use diesel::ExpressionMethods;
impl PodcastRepositoryImpl {
    pub fn get_podcast(
        podcast_id_to_be_found: i32,
    ) -> Result<Option<Podcast>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::adapters::persistence::dbconfig::schema::podcasts::id as podcast_id;
        podcasts
            .filter(podcast_id.eq(podcast_id_to_be_found))
            .first::<PodcastEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
            .map(|p|p.map(|p|p.into()))
    }

    pub fn delete_podcast(
        podcast_id_to_find: i32,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        let _ = delete(podcasts.filter(id.eq(podcast_id_to_find)))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_podcast_by_track_id(
        podcast_id: i32,
    ) -> Result<Option<Podcast>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::directory_id;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        let optional_podcast = podcasts
            .filter(directory_id.eq(podcast_id.to_string()))
            .first::<PodcastEntity>(&mut get_connection())
            .optional()
            .map(|p|p.map(|p|p.into()))
            .map_err(map_db_error)?;

        Ok(optional_podcast)
    }

    pub fn find_by_rss_feed_url(feed_url: &str) -> Result<Option<Podcast>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        podcasts
            .filter(rssfeed.eq(feed_url))
            .first::<PodcastEntity>(&mut get_connection())
            .optional()
            .map(|p|p.map(|p|p.into()))
            .map_err(map_db_error)
    }

    pub fn get_podcasts(
        u: String,
    ) -> Result<Vec<(Podcast, Tag)>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::podcast_id as f_id;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::username;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::adapters::persistence::dbconfig::schema::podcasts::id as p_id;
        podcasts
            .left_join(f_db.on(username.eq(&u).and(f_id.eq(p_id))))
            .load::<(PodcastEntity, Option<FavoriteEntity>)>(&mut get_connection())
            .map_err(map_db_error)
            .map(|r|r.into_iter().map(|e,p|(e.into(), p.into()))).collect::<Vec<Podcast, Tag>>()
    }


    pub fn get_all_podcasts() -> Result<Vec<Podcast>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        podcasts.load::<PodcastEntity>(&mut get_connection()).map_err(map_db_error)
            .map(|r|r.into_iter().map(|e|e.into()).collect::<Vec<Podcast>>())
    }

    pub fn add_podcast_to_database(
        collection_name: String,
        collection_id: String,
        feed_url: String,
        image_url_1: String,
        directory_name_to_insert: String,
    ) -> Result<Podcast, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::{
            directory_id, image_url, name as podcast_name, rssfeed,
        };
        use crate::adapters::persistence::dbconfig::schema::podcasts::{directory_name, original_image_url};

        let inserted_podcast = insert_into(podcasts::table)
            .values((
                directory_id.eq(collection_id.to_string()),
                podcast_name.eq(collection_name.to_string()),
                rssfeed.eq(feed_url.to_string()),
                image_url.eq(image_url_1.to_string()),
                original_image_url.eq(image_url_1.to_string()),
                directory_name.eq(directory_name_to_insert.to_string()),
            ))
            .get_result::<PodcastEntity>(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(inserted_podcast.into())
    }

    pub fn get_podcast_by_rss_feed(
        rss_feed_1: &str,
        conn: &mut crate::adapters::persistence::dbconfig::DBType,
    ) -> Result<Podcast, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;

        podcasts
            .filter(rssfeed.eq(rss_feed_1))
            .first::<PodcastEntity>(conn)
            .map_err(map_db_error)
            .map(|p|p.into())
    }

    pub fn get_podcast_by_directory_id(
        podcast_id: &str
    ) -> Result<Option<Podcast>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::directory_id;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts as dsl_podcast;
        dsl_podcast
            .filter(directory_id.eq(podcast_id))
            .first::<PodcastEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
            .map(|p|p.map(|p|p.into()))
    }

    pub fn update_podcast_fields(
        podcast_extra: PodcastExtra,
    ) -> Result<usize, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;

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
                .execute(&mut get_connection())
        })
            .map_err(map_db_error)
    }

    pub fn update_podcast_active(
        podcast_id: i32,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;

        let found_podcast = match Self::get_podcast(podcast_id)? {
            Some(podcast) => podcast,
            None => return Err(CustomError::NotFound),
        };

        diesel::update(podcasts.filter(id.eq(podcast_id)))
            .set(active.eq(!found_podcast.active))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn update_original_image_url(
        original_image_url_to_set: &str,
        podcast_id_to_find: i32,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        do_retry(|| {
            diesel::update(podcasts.filter(id.eq(podcast_id_to_find)))
                .set(original_image_url.eq(original_image_url_to_set))
                .execute(&mut get_connection())
        })
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn update_podcast_urls_on_redirect(
        podcast_id_to_update: i32,
        new_url: String,
    ) {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::id as pid;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;

        diesel::update(podcasts.filter(pid.eq(podcast_id_to_update)))
            .set(rssfeed.eq(new_url))
            .execute(&mut get_connection())
            .expect("Error updating podcast episode");
    }

    pub fn update_podcast(podcast: Podcast) -> Result<Podcast, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        let updated_podcast = diesel::update(podcasts.filter(id.eq(podcast.id)))
            .set(podcast.into())
            .get_result::<PodcastEntity>(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(updated_podcast.into())
    }

}