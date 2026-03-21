use crate::DBType as DbConnection;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::favorites;
use crate::adapters::persistence::dbconfig::schema::tags_podcasts::dsl::tags_podcasts;
use crate::mappers::podcast_dto_mapper::map_podcast_with_context_to_dto;
use crate::models::podcasts::Podcast;
use crate::service::tag_service::TagService;
use crate::utils::error::ErrorSeverity::Critical;
use crate::utils::error::{CustomError, map_db_error};
use diesel::insert_into;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Text};
use indexmap::IndexMap;
use podfetch_domain::ordering::{OrderCriteria, OrderOption};
use podfetch_domain::tag::{Tag, TagsPodcast};
use podfetch_domain::user::User;
use podfetch_web::podcast::PodcastDto;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(Queryable, Clone)]
struct JoinedTagsPodcast {
    tag_id: String,
    podcast_id: i32,
}

#[derive(Queryable, Clone)]
struct JoinedTag {
    id: String,
    name: String,
    username: String,
    description: Option<String>,
    created_at: chrono::NaiveDateTime,
    color: String,
}

impl From<JoinedTag> for Tag {
    fn from(value: JoinedTag) -> Self {
        Self {
            id: value.id,
            name: value.name,
            username: value.username,
            description: value.description,
            created_at: value.created_at,
            color: value.color,
        }
    }
}

impl From<JoinedTagsPodcast> for TagsPodcast {
    fn from(value: JoinedTagsPodcast) -> Self {
        Self {
            tag_id: value.tag_id,
            podcast_id: value.podcast_id,
        }
    }
}

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

type SearchPodcastType = Vec<(Podcast, Option<Favorite>, Vec<Tag>)>;

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

    pub fn get_favored_podcasts(requester: &User) -> Result<Vec<PodcastDto>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favored as favor_column;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::username as user_favor;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts as dsl_podcast;

        let result: Vec<(Podcast, Favorite)> = dsl_podcast
            .inner_join(f_db)
            .filter(
                favor_column
                    .eq(true)
                    .and(user_favor.eq(&requester.username)),
            )
            .load::<(Podcast, Favorite)>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;

        let mapped_result = result
            .iter()
            .map(|podcast| {
                let tags = TagService::default_service()
                    .get_tags_of_podcast(podcast.0.id, &requester.username)
                    .unwrap();
                map_podcast_with_context_to_dto(
                    podcast.0.clone(),
                    Some(podcast.1.clone()),
                    tags,
                    requester,
                )
            })
            .collect::<Vec<PodcastDto>>();
        Ok(mapped_result)
    }

    pub fn search_podcasts_favored(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        designated_username: &str,
    ) -> Result<Vec<(Podcast, Favorite, Vec<Tag>)>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::id as podcastsid;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::tags_podcasts as t_join_table;

        let mut query = podcasts
            .inner_join(
                favorites::table.on(podcastsid
                    .eq(favorites::dsl::podcast_id)
                    .and(favorites::dsl::username.eq(designated_username))),
            )
            .left_join(tags_podcasts.on(podcastsid.eq(t_join_table::dsl::podcast_id)))
            .left_join(
                crate::adapters::persistence::dbconfig::schema::tags::table.on(
                    crate::adapters::persistence::dbconfig::schema::tags_podcasts::dsl::tag_id
                        .eq(crate::adapters::persistence::dbconfig::schema::tags::dsl::id)
                        .and(
                            crate::adapters::persistence::dbconfig::schema::tags::dsl::username
                                .eq(designated_username),
                        ),
                ),
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
                    query = query.order_by(last_build_date.asc());
                }
                OrderCriteria::Desc => {
                    query = query.order_by(last_build_date.desc());
                }
            },
        }

        if let Some(title) = title {
            use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::name as podcasttitle;
            query = query.filter(podcasttitle.like(format!("%{}%", title)));
        }

        let mut matching_podcast_ids: BTreeMap<i32, (Podcast, Favorite, Vec<Tag>)> =
            BTreeMap::new();
        let pr = query
            .load::<(
                Podcast,
                Favorite,
                Option<JoinedTagsPodcast>,
                Option<JoinedTag>,
            )>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        pr.iter().for_each(|c| {
            if let Some(existing) = matching_podcast_ids.get_mut(&c.0.id) {
                if let Some(tag) = &c.3
                    && !existing.2.iter().any(|t| t.id == tag.id)
                {
                    existing.2.push(tag.clone().into());
                }
            } else {
                let mut tags = vec![];
                if let Some(tag) = &c.3 {
                    tags.push(tag.clone().into());
                }
                matching_podcast_ids.insert(c.0.id, (c.0.clone(), c.1.clone(), tags));
            }
        });

        Ok(matching_podcast_ids.values().cloned().collect())
    }

    pub fn search_podcasts(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        designated_username: &str,
    ) -> Result<SearchPodcastType, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favorites as f_db;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::podcast_id as f_id;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::username as f_username;

        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::id as podcastsid;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;

        let mut query = podcasts
            .left_join(f_db.on(f_username.eq(designated_username).and(f_id.eq(podcastsid))))
            .left_join(tags_podcasts.on(podcastsid.eq(
                crate::adapters::persistence::dbconfig::schema::tags_podcasts::dsl::podcast_id,
            )))
            .left_join(
                crate::adapters::persistence::dbconfig::schema::tags::table.on(
                    crate::adapters::persistence::dbconfig::schema::tags_podcasts::dsl::tag_id
                        .eq(crate::adapters::persistence::dbconfig::schema::tags::dsl::id)
                        .and(
                            crate::adapters::persistence::dbconfig::schema::tags::dsl::username
                                .eq(designated_username),
                        ),
                ),
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
                    query = query.order_by(last_build_date.asc());
                }
                OrderCriteria::Desc => {
                    query = query.order_by(last_build_date.desc());
                }
            },
        }

        define_sql_function!(fn lower(x: Text) -> Text);

        if let Some(title) = title {
            use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::name as podcasttitle;
            query = query.filter(lower(podcasttitle).like(format!("%{}%", title.to_lowercase())));
        }

        let mut matching_podcast_ids: IndexMap<i32, (Podcast, Option<Favorite>, Vec<Tag>)> =
            IndexMap::new();
        let pr = query
            .load::<(
                Podcast,
                Option<Favorite>,
                Option<JoinedTagsPodcast>,
                Option<JoinedTag>,
            )>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        pr.iter().for_each(|c| {
            if let Some(existing) = matching_podcast_ids.get_mut(&c.0.id) {
                if let Some(tag) = &c.3
                    && !existing.2.iter().any(|t| t.id == tag.id)
                {
                    existing.2.push(tag.clone().into());
                }
            } else {
                let mut tags = vec![];
                if let Some(tag) = &c.3 {
                    tags.push(tag.clone().into());
                }
                matching_podcast_ids.insert(c.0.id, (c.0.clone(), c.1.clone(), tags));
            }
        });
        Ok(matching_podcast_ids.values().cloned().collect())
    }
}
