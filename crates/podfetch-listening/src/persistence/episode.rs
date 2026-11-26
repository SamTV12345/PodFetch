use crate::domain::episode::{Episode, EpisodeRepository, Listener, NewEpisode};
use chrono::NaiveDateTime;
use common_infrastructure::db::{Database, PersistenceError};
use diesel::prelude::*;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use std::collections::HashMap;
use uuid::Uuid;

diesel::table! {
    episodes (id) {
        id -> Text,
        user_id -> Text,
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

#[derive(Queryable, Selectable, Insertable, Debug, Clone, PartialEq, Eq)]
#[diesel(table_name = episodes)]
struct EpisodeRow {
    id: String,
    user_id: String,
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

impl EpisodeRow {
    fn into_domain(self, listener: &Listener) -> Episode {
        Episode {
            id: Uuid::parse_str(&self.id).expect("valid uuid in db"),
            user_id: listener.id,
            username: listener.username.clone(),
            device: self.device,
            podcast: self.podcast,
            episode: self.episode,
            timestamp: self.timestamp,
            guid: self.guid,
            action: self.action,
            started: self.started,
            position: self.position,
            total: self.total,
        }
    }
}

fn listener_of(episode: &Episode) -> Listener {
    Listener::new(episode.user_id, episode.username.clone())
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
        let row = EpisodeRow {
            id: Uuid::now_v7().to_string(),
            user_id: episode.listener.id.to_string(),
            device: episode.device,
            podcast: episode.podcast,
            episode: episode.episode,
            timestamp: episode.timestamp,
            guid: episode.guid,
            action: episode.action,
            started: episode.started,
            position: episode.position,
            total: episode.total,
        };
        diesel::insert_into(episodes::table)
            .values(row)
            .get_result::<EpisodeRow>(&mut self.database.connection()?)
            .map(|row| row.into_domain(&episode.listener))
            .map_err(Into::into)
    }

    fn find_duplicate_action(&self, episode: &Episode) -> Result<Option<Episode>, Self::Error> {
        episodes::table
            .filter(
                episodes::user_id
                    .eq(episode.user_id.to_string())
                    .and(episodes::timestamp.eq(episode.timestamp))
                    .and(episodes::device.eq(&episode.device))
                    .and(episodes::podcast.eq(&episode.podcast))
                    .and(episodes::episode.eq(&episode.episode)),
            )
            .first::<EpisodeRow>(&mut self.database.connection()?)
            .optional()
            .map(|row| row.map(|row| row.into_domain(&listener_of(episode))))
            .map_err(Into::into)
    }

    fn insert_episode(&self, episode: &Episode) -> Result<Episode, Self::Error> {
        if let Some(existing) = self.find_duplicate_action(episode)? {
            return Ok(existing);
        }
        self.create(NewEpisode {
            listener: listener_of(episode),
            device: episode.device.clone(),
            podcast: episode.podcast.clone(),
            episode: episode.episode.clone(),
            timestamp: episode.timestamp,
            guid: episode.guid.clone(),
            action: episode.action.clone(),
            started: episode.started,
            position: episode.position,
            total: episode.total,
        })
    }

    fn find_by_device_and_guid(
        &self,
        listener: &Listener,
        device: &str,
        guid: &str,
    ) -> Result<Option<Episode>, Self::Error> {
        episodes::table
            .filter(episodes::user_id.eq(listener.id.to_string()))
            .filter(episodes::device.eq(device))
            .filter(episodes::guid.eq(guid))
            .order_by(episodes::timestamp.desc())
            .first::<EpisodeRow>(&mut self.database.connection()?)
            .optional()
            .map(|row| row.map(|row| row.into_domain(listener)))
            .map_err(Into::into)
    }

    fn find_latest_by_guid(
        &self,
        listener: &Listener,
        guid: &str,
    ) -> Result<Option<Episode>, Self::Error> {
        episodes::table
            .filter(episodes::user_id.eq(listener.id.to_string()))
            .filter(episodes::guid.eq(guid))
            .order_by(episodes::timestamp.desc())
            .first::<EpisodeRow>(&mut self.database.connection()?)
            .optional()
            .map(|row| row.map(|row| row.into_domain(listener)))
            .map_err(Into::into)
    }

    fn find_latest_by_guids(
        &self,
        listener: &Listener,
        guids: &[String],
    ) -> Result<Vec<Episode>, Self::Error> {
        let conn = &mut self.database.connection()?;
        let mut latest: HashMap<String, EpisodeRow> = HashMap::new();
        // Chunked to stay below the bind parameter limit.
        for chunk in guids.chunks(500) {
            let rows = episodes::table
                .filter(episodes::user_id.eq(listener.id.to_string()))
                .filter(episodes::guid.eq_any(chunk))
                .load::<EpisodeRow>(conn)?;
            for row in rows {
                let Some(guid) = row.guid.clone() else {
                    continue;
                };
                match latest.get(&guid) {
                    Some(current) if current.timestamp >= row.timestamp => {}
                    _ => {
                        latest.insert(guid, row);
                    }
                }
            }
        }
        Ok(latest
            .into_values()
            .map(|row| row.into_domain(listener))
            .collect())
    }

    fn find_actions(
        &self,
        listener: &Listener,
        since: Option<NaiveDateTime>,
        device: Option<&str>,
        podcast: Option<&str>,
        default_device: Option<&str>,
    ) -> Result<Vec<Episode>, Self::Error> {
        let mut query = episodes::table
            .filter(episodes::user_id.eq(listener.id.to_string()))
            .into_boxed();
        if let Some(since) = since {
            query = query.filter(episodes::timestamp.ge(since));
        }
        if let Some(device) = device {
            query = match default_device {
                Some(default_device) => query.filter(
                    episodes::device
                        .eq(device.to_owned())
                        .or(episodes::device.eq(default_device.to_owned())),
                ),
                None => query.filter(episodes::device.eq(device.to_owned())),
            };
        }
        if let Some(podcast) = podcast {
            query = query.filter(episodes::podcast.eq(podcast.to_owned()));
        }
        query
            .order_by(episodes::timestamp.desc())
            .load::<EpisodeRow>(&mut self.database.connection()?)
            .map(|rows| {
                rows.into_iter()
                    .map(|row| row.into_domain(listener))
                    .collect()
            })
            .map_err(Into::into)
    }

    fn find_last_played(&self, listener: &Listener) -> Result<Vec<Episode>, Self::Error> {
        let (latest, other) = diesel::alias!(episodes as latest, episodes as other);
        let user_id = listener.id.to_string();
        // The newest timestamp per episode URL of this user.
        let newest = other
            .select(diesel::dsl::max(other.field(episodes::timestamp)))
            .filter(
                other
                    .field(episodes::episode)
                    .eq(latest.field(episodes::episode)),
            )
            .filter(other.field(episodes::user_id).eq(user_id.clone()))
            .group_by(other.field(episodes::episode));
        latest
            .filter(latest.field(episodes::user_id).eq(user_id))
            .filter(latest.field(episodes::timestamp).nullable().eq_any(newest))
            .filter(latest.field(episodes::action).eq("play"))
            .order_by(latest.field(episodes::timestamp).desc())
            .select(latest.fields(episodes::all_columns))
            .load::<EpisodeRow>(&mut self.database.connection()?)
            .map(|rows| {
                rows.into_iter()
                    .map(|row| row.into_domain(listener))
                    .collect()
            })
            .map_err(Into::into)
    }

    fn guids_with_actions(&self, listener: &Listener) -> Result<Vec<String>, Self::Error> {
        episodes::table
            .filter(episodes::user_id.eq(listener.id.to_string()))
            .filter(episodes::guid.is_not_null())
            .select(episodes::guid.assume_not_null())
            .distinct()
            .load::<String>(&mut self.database.connection()?)
            .map_err(Into::into)
    }

    fn update_position(
        &self,
        id: Uuid,
        position: i32,
        timestamp: NaiveDateTime,
    ) -> Result<(), Self::Error> {
        diesel::update(episodes::table.filter(episodes::id.eq(id.to_string())))
            .set((
                episodes::started.eq(position),
                episodes::position.eq(position),
                episodes::timestamp.eq(timestamp),
            ))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_progress(
        &self,
        id: Uuid,
        position: i32,
        total: i32,
        timestamp: NaiveDateTime,
    ) -> Result<(), Self::Error> {
        diesel::update(episodes::table.filter(episodes::id.eq(id.to_string())))
            .set((
                episodes::action.eq("play"),
                episodes::started.eq(position),
                episodes::position.eq(position),
                episodes::total.eq(total),
                episodes::timestamp.eq(timestamp),
            ))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete_by_user(&self, user_id: Uuid) -> Result<usize, Self::Error> {
        diesel::delete(episodes::table.filter(episodes::user_id.eq(user_id.to_string())))
            .execute(&mut self.database.connection()?)
            .map_err(Into::into)
    }

    fn delete_by_podcast_feed(&self, podcast_feed: &str) -> Result<usize, Self::Error> {
        diesel::delete(episodes::table.filter(episodes::podcast.eq(podcast_feed)))
            .execute(&mut self.database.connection()?)
            .map_err(Into::into)
    }
}
