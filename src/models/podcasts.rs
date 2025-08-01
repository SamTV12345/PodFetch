use crate::adapters::persistence::dbconfig::schema::*;

use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::models::favorites::Favorite;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::tag::Tag;
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

use crate::utils::error::ErrorSeverity::{Critical, Warning};
use crate::utils::error::{map_db_error, CustomError, CustomErrorInner};

#[derive(
    Queryable, Identifiable, QueryableByName, Selectable, Debug, PartialEq, Clone, Default,
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
    #[diesel(sql_type = Nullable<Text>)]
    pub download_location: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub guid: Option<String>,
}

impl Podcast {
    pub fn find_by_rss_feed_url(feed_url: &str) -> Option<Podcast> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        podcasts
            .filter(rssfeed.eq(feed_url))
            .first::<Podcast>(&mut get_connection())
            .optional()
            .expect("Error loading podcast by rss feed url")
    }

    pub fn find_by_path(path: &str) -> Result<Option<Podcast>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        podcasts
            .filter(image_url.eq(path))
            .first::<Podcast>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))
    }

    pub fn get_podcasts(u: &str) -> Result<Vec<PodcastDto>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::podcast_id as f_id;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::username;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::adapters::persistence::dbconfig::schema::podcasts::id as p_id;
        let result = podcasts
            .left_join(f_db.on(username.eq(&u).and(f_id.eq(p_id))))
            .load::<(Podcast, Option<Favorite>)>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;

        let mapped_result = result
            .iter()
            .map(|podcast| {
                let tags = Tag::get_tags_of_podcast(podcast.0.id, u).unwrap();
                (podcast.0.clone(), podcast.1.clone(), tags).into()
            })
            .collect::<Vec<PodcastDto>>();
        Ok(mapped_result)
    }

    pub fn get_all_podcasts() -> Result<Vec<Podcast>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        let result = podcasts
            .load::<Podcast>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        Ok(result)
    }

    pub fn get_podcast(podcast_id_to_be_found: i32) -> Result<Podcast, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::adapters::persistence::dbconfig::schema::podcasts::id as podcast_id;
        let found_podcast = podcasts
            .filter(podcast_id.eq(podcast_id_to_be_found))
            .first::<Podcast>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        match found_podcast {
            Some(podcast) => Ok(podcast),
            None => Err(CustomErrorInner::NotFound(Warning).into()),
        }
    }

    pub fn delete_podcast(podcast_id_to_find: i32) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        let _ = delete(podcasts.filter(id.eq(podcast_id_to_find)))
            .execute(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        Ok(())
    }

    pub fn get_podcast_by_track_id(podcast_id: i32) -> Result<Option<Podcast>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::directory_id;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        let optional_podcast = podcasts
            .filter(directory_id.eq(podcast_id.to_string()))
            .first::<Podcast>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        Ok(optional_podcast)
    }

    pub fn add_podcast_to_database(
        collection_name: &str,
        collection_id: &str,
        feed_url: &str,
        image_url_1: &str,
        directory_name_to_insert: &str,
    ) -> Result<Podcast, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::{
            directory_id, image_url, name as podcast_name, rssfeed,
        };
        use crate::adapters::persistence::dbconfig::schema::podcasts::{
            directory_name, original_image_url,
        };

        let inserted_podcast = insert_into(podcasts::table)
            .values((
                directory_id.eq(collection_id),
                podcast_name.eq(collection_name),
                rssfeed.eq(feed_url),
                image_url.eq(image_url_1),
                original_image_url.eq(image_url_1),
                directory_name.eq(directory_name_to_insert),
            ))
            .get_result::<Podcast>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        Ok(inserted_podcast)
    }

    pub fn get_podcast_by_rss_feed(
        rss_feed_1: String,
        conn: &mut DbConnection,
    ) -> Result<Podcast, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;

        podcasts
            .filter(rssfeed.eq(rss_feed_1))
            .first::<Podcast>(conn)
            .map_err(|e| map_db_error(e, Critical))
    }

    pub fn get_podcast_by_directory_id(
        podcast_id: &str,
        conn: &mut DbConnection,
    ) -> Result<Option<Podcast>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::directory_id;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts as dsl_podcast;
        let result = dsl_podcast
            .filter(directory_id.eq(podcast_id))
            .first::<Podcast>(conn)
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;
        Ok(result)
    }

    pub fn query_for_podcast(query: &str) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        use diesel::TextExpressionMethods;
        let result = podcast_episodes
            .filter(
                name.like(format!("%{query}%"))
                    .or(description.like(format!("%{query}%"))),
            )
            .load::<PodcastEpisode>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        Ok(result)
    }

    pub fn update_podcast_fields(podcast_extra: PodcastExtra) -> Result<usize, CustomError> {
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
                    guid.eq(podcast_extra.clone().guid),
                ))
                .execute(&mut get_connection())
        })
        .map_err(|e| map_db_error(e, Critical))
    }

    pub fn update_podcast_active(podcast_id: i32) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;

        let found_podcast = Podcast::get_podcast(podcast_id)?;

        diesel::update(podcasts.filter(id.eq(podcast_id)))
            .set(active.eq(!found_podcast.active))
            .execute(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
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
        .map_err(|e| map_db_error(e, Critical))?;
        Ok(())
    }

    pub fn update_podcast_urls_on_redirect(podcast_id_to_update: i32, new_url: String) {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::id as pid;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;

        diesel::update(podcasts.filter(pid.eq(podcast_id_to_update)))
            .set(rssfeed.eq(new_url))
            .execute(&mut get_connection())
            .expect("Error updating podcast episode");
    }

    pub fn update_podcast_name(
        podcast_id_to_update: i32,
        new_name: &str,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        do_retry(|| {
            diesel::update(podcasts.filter(id.eq(podcast_id_to_update)))
                .set(name.eq(new_name))
                .execute(&mut get_connection())
        })
        .map_err(|e| map_db_error(e, Critical))?;
        Ok(())
    }
}
