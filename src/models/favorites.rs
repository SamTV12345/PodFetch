use crate::DBType as DbConnection;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::favorites;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::tag::Tag;
use crate::models::user::User;
use crate::utils::error::ErrorSeverity::Critical;
use crate::utils::error::{CustomError, map_db_error};
use diesel::insert_into;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Text};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Queryable,
    Associations,
    Debug,
    PartialEq,
    QueryableByName,
    Serialize,
    Deserialize,
    Insertable,
    Clone,
    AsChangeset,
    ToSchema,
)]
#[diesel(belongs_to(Podcast, foreign_key = podcast_id))]
#[diesel(belongs_to(User, foreign_key = username))]
pub struct Favorite {
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Integer)]
    pub podcast_id: i32,
    #[diesel(sql_type = Bool)]
    pub favored: bool,
}

impl Favorite {
    pub fn delete_by_username(
        username1: String,
        conn: &mut DbConnection,
    ) -> Result<(), diesel::result::Error> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::*;
        diesel::delete(favorites.filter(username.eq(username1))).execute(conn)?;
        Ok(())
    }

    pub fn update_podcast_favor(
        podcast_id_1: &i32,
        favor: bool,
        username_1: &str,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favored as favor_column;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::podcast_id;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::username;

        let res = f_db
            .filter(podcast_id.eq(podcast_id_1).and(username.eq(username_1)))
            .first::<Favorite>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        match res {
            Some(..) => {
                diesel::update(
                    f_db.filter(podcast_id.eq(podcast_id_1).and(username.eq(username_1))),
                )
                .set(favor_column.eq(favor))
                .execute(&mut get_connection())
                .map_err(|e| map_db_error(e, Critical))?;
                Ok(())
            }
            None => {
                insert_into(f_db)
                    .values((
                        podcast_id.eq(podcast_id_1),
                        username.eq(username_1),
                        favor_column.eq(favor),
                    ))
                    .execute(&mut get_connection())
                    .map_err(|e| map_db_error(e, Critical))?;
                Ok(())
            }
        }
    }

    pub fn get_favored_podcast_by_username_and_podcast_id(
        username1: &str,
        podcast_id1: i32,
    ) -> Result<Option<Favorite>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::*;
        let res = favorites
            .filter(username.eq(username1).and(podcast_id.eq(podcast_id1)))
            .first::<Favorite>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;
        Ok(res)
    }

    pub fn get_favored_podcasts(found_username: String) -> Result<Vec<PodcastDto>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favored as favor_column;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::username as user_favor;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts as dsl_podcast;

        let result: Vec<(Podcast, Favorite)> = dsl_podcast
            .inner_join(f_db)
            .filter(favor_column.eq(true).and(user_favor.eq(&found_username)))
            .load::<(Podcast, Favorite)>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;

        let mapped_result = result
            .iter()
            .map(|podcast| {
                let tags = Tag::get_tags_of_podcast(podcast.0.id, &found_username).unwrap();
                (podcast.0.clone(), Some(podcast.1.clone()), tags).into()
            })
            .collect::<Vec<PodcastDto>>();
        Ok(mapped_result)
    }

    pub fn search_podcasts_favored(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        designated_username: &str,
    ) -> Result<Vec<(Podcast, Favorite)>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::id as podcastsid;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;

        let mut query = podcasts
            .inner_join(podcast_episodes.on(podcastsid.eq(podcast_id)))
            .inner_join(
                favorites::table.on(podcastsid
                    .eq(favorites::dsl::podcast_id)
                    .and(favorites::dsl::username.eq(designated_username))),
            )
            .into_boxed();

        match latest_pub {
            OrderOption::Title => {
                use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::name as podcasttitle;
                match order {
                    OrderCriteria::Asc => {
                        query = query.order_by(podcasttitle.asc());
                    }
                    OrderCriteria::Desc => {
                        query = query.order_by(podcasttitle.desc());
                    }
                }
            }
            OrderOption::PublishedDate => match order {
                OrderCriteria::Asc => {
                    query = query.order_by(date_of_recording.asc());
                }
                OrderCriteria::Desc => {
                    query = query.order_by(date_of_recording.desc());
                }
            },
        }

        if title.is_some() {
            use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::name as podcasttitle;
            query = query.filter(podcasttitle.like(format!("%{}%", title.unwrap())));
        }

        let mut matching_podcast_ids = vec![];
        let pr = query
            .load::<(Podcast, PodcastEpisode, Favorite)>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        let distinct_podcasts: Vec<(Podcast, Favorite)> = pr
            .iter()
            .filter(|c| {
                if matching_podcast_ids.contains(&c.0.id) {
                    return false;
                }
                matching_podcast_ids.push(c.0.id);
                true
            })
            .map(|c| (c.clone().0, c.clone().2))
            .collect::<Vec<(Podcast, Favorite)>>();
        Ok(distinct_podcasts)
    }

    pub fn search_podcasts(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        designated_username: &str,
    ) -> Result<Vec<(Podcast, Option<Favorite>)>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::podcast_id as f_id;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::username as f_username;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::id as podcastsid;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;

        let mut query = podcasts
            .inner_join(podcast_episodes.on(podcastsid.eq(podcast_id)))
            .left_join(f_db.on(f_username.eq(designated_username).and(f_id.eq(podcast_id))))
            .into_boxed();

        match latest_pub {
            OrderOption::Title => {
                use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::name as podcasttitle;
                match order {
                    OrderCriteria::Asc => {
                        query = query.order_by(podcasttitle.asc());
                    }
                    OrderCriteria::Desc => {
                        query = query.order_by(podcasttitle.desc());
                    }
                }
            }
            OrderOption::PublishedDate => match order {
                OrderCriteria::Asc => {
                    query = query.order_by(date_of_recording.asc());
                }
                OrderCriteria::Desc => {
                    query = query.order_by(date_of_recording.desc());
                }
            },
        }

        define_sql_function!(fn lower(x: Text) -> Text);

        if let Some(title) = title {
            use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::name as podcasttitle;
            query = query.filter(lower(podcasttitle).like(format!("%{}%", title.to_lowercase())));
        }

        let mut matching_podcast_ids = vec![];
        let pr = query
            .load::<(Podcast, PodcastEpisode, Option<Favorite>)>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        let distinct_podcasts = pr
            .iter()
            .filter(|c| {
                if matching_podcast_ids.contains(&c.0.id) {
                    return false;
                }
                matching_podcast_ids.push(c.0.id);
                true
            })
            .map(|c| (c.clone().0, c.clone().2))
            .collect::<Vec<(Podcast, Option<Favorite>)>>();
        Ok(distinct_podcasts)
    }
}
