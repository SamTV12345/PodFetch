use diesel::{define_sql_function, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
use diesel::sql_types::Text;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::favorites;
use crate::adapters::persistence::model::favorite::favorites::FavoriteEntity;
use crate::adapters::persistence::model::podcast::podcast::PodcastEntity;
use crate::adapters::persistence::model::podcast_episode::podcast_episode::PodcastEpisodeEntity;
use crate::adapters::persistence::repositories::tag::tag::TagRepositoryImpl;
use crate::application::repositories::favorite_repository::FavoriteRepository;
use crate::domain::models::favorite::favorite::Favorite;
use crate::domain::models::order_criteria::{OrderCriteria, OrderOption};
use crate::domain::models::podcast::podcast::Podcast;
use crate::domain::models::tag::tag::Tag;
use crate::utils::error::{map_db_error, CustomError};

pub struct FavoriteRepositoryImpl;


impl FavoriteRepository for FavoriteRepositoryImpl {
    fn search_podcasts(order: OrderCriteria, title: Option<String>, latest_pub: OrderOption, designated_username: &str) -> Result<Vec<(Podcast, Option<Favorite>)>, CustomError> {
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
            .load::<(Podcast, PodcastEpisode, Option<crate::adapters::persistence::model::favorite::favorites::FavoriteEntity>)>(&mut get_connection())
            .map_err(map_db_error)?;
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
            .collect::<Vec<(Podcast, Option<FavoriteEntity>)>>();

        let mapped_podcast = distinct_podcasts.iter().map(|c|{
            return (
                c.0.clone(),
                match c.1.clone() {
                    Some(f) => Some(f.into()),
                    None => None
                }
                )
        }).collect();

        Ok(mapped_podcast)
    }

    fn get_favored_podcasts(found_username: &str) -> Result<Vec<(Podcast, Vec<Tag>)>,
        CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favored as favor_column;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::username as user_favor;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts as dsl_podcast;

        let result: Vec<(Podcast, Favorite)> = dsl_podcast
            .inner_join(f_db)
            .filter(favor_column.eq(true).and(user_favor.eq(&found_username)))
            .load::<(PodcastEntity, FavoriteEntity)>(&mut get_connection())
            .map_err(map_db_error)
            .map(|c|c.into_iter().map(|c|(c.0.into(), c.1.into())).collect())?;

        let mapped_result = result
            .iter()
            .map(|podcast| {
                let tags = TagRepositoryImpl::get_tags_of_podcast(podcast.0.id, &found_username).unwrap();
                (podcast, tags)
            })
            .collect::<Vec<(Podcast, Vec<Tag>)>>();
        Ok(mapped_result)
    }

    fn search_podcasts_favored(order: OrderCriteria,
                                           title: Option<String>,
                                           latest_pub: OrderOption,
                                           designated_username: &str) -> Result<Vec<(Podcast,
                                                                                     Favorite)>,
        CustomError> {
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
            .load::<(PodcastEntity, PodcastEpisodeEntity, FavoriteEntity)>(&mut get_connection())
            .map_err(map_db_error)?;
        let distinct_podcasts: Vec<(Podcast, Favorite)> = pr
            .iter()
            .filter(|c| {
                if matching_podcast_ids.contains(&c.0.id) {
                    return false;
                }
                matching_podcast_ids.push(c.0.id);
                true
            })
            .map(|c| (c.clone().0, c.clone().2.into()))
            .collect::<Vec<(Podcast, Favorite)>>();
        Ok(distinct_podcasts)
    }
}