use crate::db::{Database, PersistenceError};
use crate::podcast::podcasts;
use crate::podcast_episode::podcast_episodes;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::episode::{Episode, EpisodeRepository, LastWatchedEpisode, NewEpisode};
use podfetch_domain::podcast::Podcast;
use podfetch_domain::podcast_episode::PodcastEpisode;

diesel::table! {
    episodes (id) {
        id -> Integer,
        username -> Text,
        device -> Text,
        podcast -> Text,
        episode -> Text,
        timestamp -> Timestamp,
        guid -> Nullable<Text>,
        action -> Text,
        started -> Nullable<Integer>,
        position -> Nullable<Integer>,
        total -> Nullable<Integer>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(episodes, podcasts, podcast_episodes);

#[derive(Queryable, Selectable, Debug, Clone, PartialEq, Eq)]
#[diesel(table_name = episodes)]
pub struct EpisodeEntity {
    pub id: i32,
    pub username: String,
    pub device: String,
    pub podcast: String,
    pub episode: String,
    pub timestamp: NaiveDateTime,
    pub guid: Option<String>,
    pub action: String,
    pub started: Option<i32>,
    pub position: Option<i32>,
    pub total: Option<i32>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = episodes)]
struct NewEpisodeEntity {
    username: String,
    device: String,
    podcast: String,
    episode: String,
    timestamp: NaiveDateTime,
    guid: Option<String>,
    action: String,
    started: Option<i32>,
    position: Option<i32>,
    total: Option<i32>,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = podcast_episodes)]
struct PodcastEpisodeEntity {
    id: i32,
    podcast_id: i32,
    episode_id: String,
    name: String,
    url: String,
    date_of_recording: String,
    image_url: String,
    total_time: i32,
    description: String,
    download_time: Option<NaiveDateTime>,
    guid: String,
    deleted: bool,
    file_episode_path: Option<String>,
    file_image_path: Option<String>,
    episode_numbering_processed: bool,
    download_location: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = podcasts)]
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
}

impl From<EpisodeEntity> for Episode {
    fn from(entity: EpisodeEntity) -> Self {
        Self {
            id: entity.id,
            username: entity.username,
            device: entity.device,
            podcast: entity.podcast,
            episode: entity.episode,
            timestamp: entity.timestamp,
            guid: entity.guid,
            action: entity.action,
            started: entity.started,
            position: entity.position,
            total: entity.total,
        }
    }
}

impl From<Episode> for EpisodeEntity {
    fn from(episode: Episode) -> Self {
        Self {
            id: episode.id,
            username: episode.username,
            device: episode.device,
            podcast: episode.podcast,
            episode: episode.episode,
            timestamp: episode.timestamp,
            guid: episode.guid,
            action: episode.action,
            started: episode.started,
            position: episode.position,
            total: episode.total,
        }
    }
}

impl From<NewEpisode> for NewEpisodeEntity {
    fn from(episode: NewEpisode) -> Self {
        Self {
            username: episode.username,
            device: episode.device,
            podcast: episode.podcast,
            episode: episode.episode,
            timestamp: episode.timestamp,
            guid: episode.guid,
            action: episode.action,
            started: episode.started,
            position: episode.position,
            total: episode.total,
        }
    }
}

impl From<&Episode> for NewEpisodeEntity {
    fn from(episode: &Episode) -> Self {
        Self {
            username: episode.username.clone(),
            device: episode.device.clone(),
            podcast: episode.podcast.clone(),
            episode: episode.episode.clone(),
            timestamp: episode.timestamp,
            guid: episode.guid.clone(),
            action: episode.action.clone(),
            started: episode.started,
            position: episode.position,
            total: episode.total,
        }
    }
}

impl From<PodcastEpisodeEntity> for PodcastEpisode {
    fn from(entity: PodcastEpisodeEntity) -> Self {
        Self {
            id: entity.id,
            podcast_id: entity.podcast_id,
            episode_id: entity.episode_id,
            name: entity.name,
            url: entity.url,
            date_of_recording: entity.date_of_recording,
            image_url: entity.image_url,
            total_time: entity.total_time,
            description: entity.description,
            download_time: entity.download_time,
            guid: entity.guid,
            deleted: entity.deleted,
            file_episode_path: entity.file_episode_path,
            file_image_path: entity.file_image_path,
            episode_numbering_processed: entity.episode_numbering_processed,
            download_location: entity.download_location,
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
        }
    }
}

pub struct DieselEpisodeRepository {
    database: Database,
}

impl DieselEpisodeRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl EpisodeRepository for DieselEpisodeRepository {
    type Error = PersistenceError;

    fn create(&self, episode: NewEpisode) -> Result<Episode, Self::Error> {
        diesel::insert_into(episodes::table)
            .values(NewEpisodeEntity::from(episode))
            .get_result::<EpisodeEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn insert_episode(&self, episode: &Episode) -> Result<Episode, Self::Error> {
        // Check for existing entry with same timestamp, device, podcast, and episode URL
        let existing = episodes::table
            .filter(
                episodes::timestamp
                    .eq(episode.timestamp)
                    .and(episodes::device.eq(&episode.device))
                    .and(episodes::podcast.eq(&episode.podcast))
                    .and(episodes::episode.eq(&episode.episode)),
            )
            .first::<EpisodeEntity>(&mut self.database.connection()?)
            .optional()?;

        if let Some(entity) = existing {
            return Ok(entity.into());
        }

        // Insert new episode
        diesel::insert_into(episodes::table)
            .values(NewEpisodeEntity::from(episode))
            .get_result::<EpisodeEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn find_by_username_and_episode(
        &self,
        username: &str,
        episode_url: &str,
    ) -> Result<Option<Episode>, Self::Error> {
        episodes::table
            .filter(
                episodes::username
                    .eq(username)
                    .and(episodes::episode.eq(episode_url)),
            )
            .order_by(episodes::timestamp.desc())
            .first::<EpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_username_device_guid(
        &self,
        username: &str,
        device: &str,
        guid: &str,
    ) -> Result<Option<Episode>, Self::Error> {
        episodes::table
            .filter(
                episodes::username
                    .eq(username)
                    .and(episodes::device.eq(device))
                    .and(episodes::guid.eq(guid)),
            )
            .order_by(episodes::timestamp.desc())
            .first::<EpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_actions_by_username(
        &self,
        username: &str,
        since: Option<NaiveDateTime>,
        device: Option<&str>,
        podcast: Option<&str>,
        default_device: Option<&str>,
    ) -> Result<Vec<Episode>, Self::Error> {
        let mut query = episodes::table
            .filter(episodes::username.eq(username))
            .into_boxed();

        if let Some(since_date) = since {
            query = query.filter(episodes::timestamp.ge(since_date));
        }

        if let Some(device_id) = device {
            // If default_device is provided, include both the specified device and default device
            if let Some(default_dev) = default_device {
                query = query.filter(
                    episodes::device
                        .eq(device_id)
                        .or(episodes::device.eq(default_dev)),
                );
            } else {
                query = query.filter(episodes::device.eq(device_id));
            }
        }

        if let Some(podcast_feed) = podcast {
            query = query.filter(episodes::podcast.eq(podcast_feed));
        }

        query
            .order_by(episodes::timestamp.desc())
            .load::<EpisodeEntity>(&mut self.database.connection()?)
            .map(|entities| entities.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_watch_log_by_username_and_episode(
        &self,
        username: &str,
        episode_url: &str,
    ) -> Result<Option<Episode>, Self::Error> {
        use diesel::JoinOnDsl;

        episodes::table
            .inner_join(podcasts::table.on(episodes::podcast.eq(podcasts::rssfeed)))
            .filter(episodes::username.eq(username))
            .filter(episodes::episode.eq(episode_url))
            .order_by(episodes::timestamp.desc())
            .select(EpisodeEntity::as_select())
            .first::<EpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn find_last_watched_episodes(
        &self,
        username: &str,
    ) -> Result<Vec<LastWatchedEpisode>, Self::Error> {
        use diesel::JoinOnDsl;

        let (episodes1, episodes2) = diesel::alias!(episodes as e1, episodes as e2);

        // Subquery to get max timestamp per episode for this user
        let subquery = episodes2
            .select(diesel::dsl::max(episodes2.field(episodes::timestamp)))
            .filter(
                episodes2
                    .field(episodes::episode)
                    .eq(episodes1.field(episodes::episode)),
            )
            .filter(episodes2.field(episodes::username).eq(username))
            .group_by(episodes2.field(episodes::episode));

        // Main query: join podcast_episodes, episodes, and podcasts
        let results = podcast_episodes::table
            .inner_join(
                episodes1.on(podcast_episodes::guid
                    .nullable()
                    .eq(episodes1.field(episodes::guid))),
            )
            .inner_join(podcasts::table.on(podcasts::id.eq(podcast_episodes::podcast_id)))
            .filter(episodes1.field(episodes::username).eq(username))
            .filter(
                episodes1
                    .field(episodes::timestamp)
                    .nullable()
                    .eq_any(subquery),
            )
            .filter(episodes1.field(episodes::action).eq("play"))
            .order_by(episodes1.field(episodes::timestamp).desc())
            .select((
                PodcastEpisodeEntity::as_select(),
                (
                    episodes1.field(episodes::id),
                    episodes1.field(episodes::username),
                    episodes1.field(episodes::device),
                    episodes1.field(episodes::podcast),
                    episodes1.field(episodes::episode),
                    episodes1.field(episodes::timestamp),
                    episodes1.field(episodes::guid),
                    episodes1.field(episodes::action),
                    episodes1.field(episodes::started),
                    episodes1.field(episodes::position),
                    episodes1.field(episodes::total),
                ),
                PodcastEntity::as_select(),
            ))
            .load::<(
                PodcastEpisodeEntity,
                (
                    i32,
                    String,
                    String,
                    String,
                    String,
                    NaiveDateTime,
                    Option<String>,
                    String,
                    Option<i32>,
                    Option<i32>,
                    Option<i32>,
                ),
                PodcastEntity,
            )>(&mut self.database.connection()?)?;

        Ok(results
            .into_iter()
            .map(|(pe, ep_tuple, p)| {
                let episode_entity = EpisodeEntity {
                    id: ep_tuple.0,
                    username: ep_tuple.1,
                    device: ep_tuple.2,
                    podcast: ep_tuple.3,
                    episode: ep_tuple.4,
                    timestamp: ep_tuple.5,
                    guid: ep_tuple.6,
                    action: ep_tuple.7,
                    started: ep_tuple.8,
                    position: ep_tuple.9,
                    total: ep_tuple.10,
                };
                LastWatchedEpisode {
                    podcast_episode: pe.into(),
                    episode_action: episode_entity.into(),
                    podcast: p.into(),
                }
            })
            .collect())
    }

    fn find_watchtime(
        &self,
        episode_id: &str,
        username: &str,
    ) -> Result<Option<Episode>, Self::Error> {
        // First find the podcast episode by episode_id
        let podcast_episode = podcast_episodes::table
            .filter(podcast_episodes::episode_id.eq(episode_id))
            .first::<PodcastEpisodeEntity>(&mut self.database.connection()?)
            .optional()?;

        let Some(pe) = podcast_episode else {
            return Ok(None);
        };

        // Then find the episode action by guid
        episodes::table
            .filter(episodes::username.eq(username))
            .filter(episodes::guid.eq(&pe.guid))
            .order_by(episodes::timestamp.desc())
            .first::<EpisodeEntity>(&mut self.database.connection()?)
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    fn update_position(
        &self,
        id: i32,
        position: i32,
        timestamp: NaiveDateTime,
    ) -> Result<(), Self::Error> {
        diesel::update(episodes::table.filter(episodes::id.eq(id)))
            .set((
                episodes::started.eq(position),
                episodes::position.eq(position),
                episodes::timestamp.eq(timestamp),
            ))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error> {
        diesel::delete(episodes::table.filter(episodes::username.eq(username)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete_by_podcast_feed(&self, podcast_feed: &str) -> Result<(), Self::Error> {
        diesel::delete(episodes::table.filter(episodes::podcast.eq(podcast_feed)))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete_by_podcast_id(&self, podcast_id: i32) -> Result<(), Self::Error> {
        // First find the podcast to get its RSS feed URL
        let found_podcast = podcasts::table
            .filter(podcasts::id.eq(podcast_id))
            .first::<PodcastEntity>(&mut self.database.connection()?)
            .optional()?;

        if let Some(podcast) = found_podcast {
            diesel::delete(episodes::table.filter(episodes::podcast.eq(&podcast.rssfeed)))
                .execute(&mut self.database.connection()?)?;
        }

        Ok(())
    }
}
