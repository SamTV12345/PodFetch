use crate::db::{Database, PersistenceError};
use diesel::prelude::*;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl,
};
use indexmap::IndexMap;
use podfetch_domain::favorite::{
    FavoredPodcastSearchResult, Favorite, FavoriteRepository, PodcastSearchResult,
    PodcastWithFavorite,
};
use podfetch_domain::ordering::{OrderCriteria, OrderOption};
use podfetch_domain::podcast::Podcast;
use podfetch_domain::tag::Tag;
use std::collections::BTreeMap;

diesel::table! {
    favorites (user_id, podcast_id) {
        user_id -> Integer,
        podcast_id -> Integer,
        favored -> Bool,
    }
}

diesel::table! {
    podcasts (id) {
        id -> Integer,
        name -> Text,
        directory_id -> Text,
        rssfeed -> Text,
        image_url -> Text,
        summary -> Nullable<Text>,
        language -> Nullable<Text>,
        explicit -> Nullable<Text>,
        keywords -> Nullable<Text>,
        last_build_date -> Nullable<Text>,
        author -> Nullable<Text>,
        active -> Bool,
        original_image_url -> Text,
        directory_name -> Text,
        download_location -> Nullable<Text>,
        guid -> Nullable<Text>,
        added_by -> Nullable<Integer>,
    }
}

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

diesel::allow_tables_to_appear_in_same_query!(favorites, podcasts, tags, tags_podcasts);

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, PartialEq, Eq)]
#[diesel(table_name = favorites)]
pub struct FavoriteEntity {
    pub user_id: i32,
    pub podcast_id: i32,
    pub favored: bool,
}

#[derive(Queryable, Debug, Clone)]
struct PodcastEntity {
    id: i32,
    name: String,
    directory_id: String,
    rssfeed: String,
    image_url: String,
    summary: Option<String>,
    language: Option<String>,
    explicit: Option<String>,
    keywords: Option<String>,
    last_build_date: Option<String>,
    author: Option<String>,
    active: bool,
    original_image_url: String,
    directory_name: String,
    download_location: Option<String>,
    guid: Option<String>,
    added_by: Option<i32>,
}

#[derive(Queryable, Clone)]
#[allow(dead_code)]
struct JoinedTagsPodcast {
    tag_id: String,
    podcast_id: i32,
}

#[derive(Queryable, Clone)]
struct JoinedTag {
    id: String,
    name: String,
    user_id: i32,
    description: Option<String>,
    created_at: chrono::NaiveDateTime,
    color: String,
}

impl From<FavoriteEntity> for Favorite {
    fn from(entity: FavoriteEntity) -> Self {
        Self {
            user_id: entity.user_id,
            podcast_id: entity.podcast_id,
            favored: entity.favored,
        }
    }
}

impl From<Favorite> for FavoriteEntity {
    fn from(favorite: Favorite) -> Self {
        Self {
            user_id: favorite.user_id,
            podcast_id: favorite.podcast_id,
            favored: favorite.favored,
        }
    }
}

impl From<PodcastEntity> for Podcast {
    fn from(entity: PodcastEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            directory_id: entity.directory_id,
            rssfeed: entity.rssfeed,
            image_url: entity.image_url,
            summary: entity.summary,
            language: entity.language,
            explicit: entity.explicit,
            keywords: entity.keywords,
            last_build_date: entity.last_build_date,
            author: entity.author,
            active: entity.active,
            original_image_url: entity.original_image_url,
            directory_name: entity.directory_name,
            download_location: entity.download_location,
            guid: entity.guid,
            added_by: entity.added_by,
        }
    }
}

impl From<JoinedTag> for Tag {
    fn from(value: JoinedTag) -> Self {
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

pub struct DieselFavoriteRepository {
    database: Database,
}

impl DieselFavoriteRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl FavoriteRepository for DieselFavoriteRepository {
    type Error = PersistenceError;

    fn upsert(&self, favorite: Favorite) -> Result<(), Self::Error> {
        let entity = FavoriteEntity::from(favorite);

        let existing = favorites::table
            .filter(
                favorites::user_id
                    .eq(entity.user_id)
                    .and(favorites::podcast_id.eq(entity.podcast_id)),
            )
            .first::<FavoriteEntity>(&mut self.database.connection()?)
            .optional()?;

        match existing {
            Some(_) => diesel::update(
                favorites::table.filter(
                    favorites::user_id
                        .eq(entity.user_id)
                        .and(favorites::podcast_id.eq(entity.podcast_id)),
                ),
            )
            .set(favorites::favored.eq(entity.favored))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into),
            None => diesel::insert_into(favorites::table)
                .values(&entity)
                .execute(&mut self.database.connection()?)
                .map(|_| ())
                .map_err(Into::into),
        }
    }

    fn find_by_user_id_and_podcast_id(
        &self,
        user_id: i32,
        podcast_id: i32,
    ) -> Result<Option<Favorite>, Self::Error> {
        favorites::table
            .filter(
                favorites::user_id
                    .eq(user_id)
                    .and(favorites::podcast_id.eq(podcast_id)),
            )
            .first::<FavoriteEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_favored_by_user_id(&self, user_id: i32) -> Result<Vec<Favorite>, Self::Error> {
        favorites::table
            .filter(
                favorites::user_id
                    .eq(user_id)
                    .and(favorites::favored.eq(true)),
            )
            .load::<FavoriteEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id: i32) -> Result<(), Self::Error> {
        diesel::delete(favorites::table.filter(favorites::user_id.eq(user_id)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_podcast_favor(
        &self,
        podcast_id: i32,
        favor: bool,
        user_id: i32,
    ) -> Result<(), Self::Error> {
        let existing = favorites::table
            .filter(
                favorites::podcast_id
                    .eq(podcast_id)
                    .and(favorites::user_id.eq(user_id)),
            )
            .first::<FavoriteEntity>(&mut self.database.connection()?)
            .optional()?;

        match existing {
            Some(_) => diesel::update(
                favorites::table.filter(
                    favorites::podcast_id
                        .eq(podcast_id)
                        .and(favorites::user_id.eq(user_id)),
                ),
            )
            .set(favorites::favored.eq(favor))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into),
            None => diesel::insert_into(favorites::table)
                .values((
                    favorites::podcast_id.eq(podcast_id),
                    favorites::user_id.eq(user_id),
                    favorites::favored.eq(favor),
                ))
                .execute(&mut self.database.connection()?)
                .map(|_| ())
                .map_err(Into::into),
        }
    }

    fn get_favored_podcasts(
        &self,
        user_id: i32,
    ) -> Result<Vec<PodcastWithFavorite>, Self::Error> {
        podcasts::table
            .inner_join(favorites::table.on(podcasts::id.eq(favorites::podcast_id)))
            .filter(
                favorites::favored
                    .eq(true)
                    .and(favorites::user_id.eq(user_id)),
            )
            .load::<(PodcastEntity, FavoriteEntity)>(&mut self.database.connection()?)
            .map(|results| {
                results
                    .into_iter()
                    .map(|(podcast, favorite)| PodcastWithFavorite {
                        podcast: podcast.into(),
                        favorite: favorite.into(),
                    })
                    .collect()
            })
            .map_err(Into::into)
    }

    fn search_podcasts_favored(
        &self,
        order: OrderCriteria,
        title: Option<String>,
        order_option: OrderOption,
        user_id: i32,
    ) -> Result<Vec<FavoredPodcastSearchResult>, Self::Error> {
        let mut conn = self.database.connection()?;

        let mut query = podcasts::table
            .inner_join(
                favorites::table.on(podcasts::id
                    .eq(favorites::podcast_id)
                    .and(favorites::user_id.eq(user_id))),
            )
            .left_join(tags_podcasts::table.on(podcasts::id.eq(tags_podcasts::podcast_id)))
            .left_join(
                tags::table.on(tags_podcasts::tag_id
                    .eq(tags::id)
                    .and(tags::user_id.eq(user_id))),
            )
            .into_boxed();

        match order_option {
            OrderOption::Title => match order {
                OrderCriteria::Asc => {
                    query = query.order_by(podcasts::name.asc());
                }
                OrderCriteria::Desc => {
                    query = query.order_by(podcasts::name.desc());
                }
            },
            OrderOption::PublishedDate => match order {
                OrderCriteria::Asc => {
                    query = query.order_by(podcasts::last_build_date.asc());
                }
                OrderCriteria::Desc => {
                    query = query.order_by(podcasts::last_build_date.desc());
                }
            },
        }

        if let Some(title) = title {
            query = query.filter(podcasts::name.like(format!("%{}%", title)));
        }

        let results = query.load::<(
            PodcastEntity,
            FavoriteEntity,
            Option<JoinedTagsPodcast>,
            Option<JoinedTag>,
        )>(&mut conn)?;

        let mut matching_podcast_ids: BTreeMap<i32, FavoredPodcastSearchResult> = BTreeMap::new();
        for (podcast, favorite, _tags_podcast, tag) in results {
            if let Some(existing) = matching_podcast_ids.get_mut(&podcast.id) {
                if let Some(tag) = tag
                    && !existing.tags.iter().any(|t| t.id == tag.id)
                {
                    existing.tags.push(tag.into());
                }
            } else {
                let mut tags = vec![];
                if let Some(tag) = tag {
                    tags.push(tag.into());
                }
                matching_podcast_ids.insert(
                    podcast.id,
                    FavoredPodcastSearchResult {
                        podcast: podcast.into(),
                        favorite: favorite.into(),
                        tags,
                    },
                );
            }
        }

        Ok(matching_podcast_ids.values().cloned().collect())
    }

    fn search_podcasts(
        &self,
        order: OrderCriteria,
        title: Option<String>,
        order_option: OrderOption,
        user_id: i32,
    ) -> Result<Vec<PodcastSearchResult>, Self::Error> {
        diesel::define_sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

        let mut conn = self.database.connection()?;

        let mut query = podcasts::table
            .left_join(
                favorites::table.on(favorites::user_id
                    .eq(user_id)
                    .and(favorites::podcast_id.eq(podcasts::id))),
            )
            .left_join(tags_podcasts::table.on(podcasts::id.eq(tags_podcasts::podcast_id)))
            .left_join(
                tags::table.on(tags_podcasts::tag_id
                    .eq(tags::id)
                    .and(tags::user_id.eq(user_id))),
            )
            .into_boxed();

        match order_option {
            OrderOption::Title => match order {
                OrderCriteria::Asc => {
                    query = query.order_by(podcasts::name.asc());
                }
                OrderCriteria::Desc => {
                    query = query.order_by(podcasts::name.desc());
                }
            },
            OrderOption::PublishedDate => match order {
                OrderCriteria::Asc => {
                    query = query.order_by(podcasts::last_build_date.asc());
                }
                OrderCriteria::Desc => {
                    query = query.order_by(podcasts::last_build_date.desc());
                }
            },
        }

        if let Some(title) = title {
            query = query.filter(lower(podcasts::name).like(format!("%{}%", title.to_lowercase())));
        }

        let results = query.load::<(
            PodcastEntity,
            Option<FavoriteEntity>,
            Option<JoinedTagsPodcast>,
            Option<JoinedTag>,
        )>(&mut conn)?;

        let mut matching_podcast_ids: IndexMap<i32, PodcastSearchResult> = IndexMap::new();
        for (podcast, favorite, _tags_podcast, tag) in results {
            if let Some(existing) = matching_podcast_ids.get_mut(&podcast.id) {
                if let Some(tag) = tag
                    && !existing.tags.iter().any(|t| t.id == tag.id)
                {
                    existing.tags.push(tag.into());
                }
            } else {
                let mut tags = vec![];
                if let Some(tag) = tag {
                    tags.push(tag.into());
                }
                matching_podcast_ids.insert(
                    podcast.id,
                    PodcastSearchResult {
                        podcast: podcast.into(),
                        favorite: favorite.map(Into::into),
                        tags,
                    },
                );
            }
        }

        Ok(matching_podcast_ids.values().cloned().collect())
    }
}
