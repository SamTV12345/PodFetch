use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable, Selectable};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_domain::listening_event::{
    ListeningEvent, ListeningEventRepository, NewListeningEvent,
};

diesel::table! {
    listening_events (id) {
        id -> Integer,
        user_id -> Integer,
        device -> Text,
        podcast_episode_id -> Text,
        podcast_id -> Integer,
        podcast_episode_db_id -> Integer,
        delta_seconds -> Integer,
        start_position -> Integer,
        end_position -> Integer,
        listened_at -> Timestamp,
    }
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = listening_events)]
struct ListeningEventEntity {
    id: i32,
    user_id: i32,
    device: String,
    podcast_episode_id: String,
    podcast_id: i32,
    podcast_episode_db_id: i32,
    delta_seconds: i32,
    start_position: i32,
    end_position: i32,
    listened_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = listening_events)]
struct NewListeningEventEntity {
    user_id: i32,
    device: String,
    podcast_episode_id: String,
    podcast_id: i32,
    podcast_episode_db_id: i32,
    delta_seconds: i32,
    start_position: i32,
    end_position: i32,
    listened_at: NaiveDateTime,
}

impl From<ListeningEventEntity> for ListeningEvent {
    fn from(value: ListeningEventEntity) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            device: value.device,
            podcast_episode_id: value.podcast_episode_id,
            podcast_id: value.podcast_id,
            podcast_episode_db_id: value.podcast_episode_db_id,
            delta_seconds: value.delta_seconds,
            start_position: value.start_position,
            end_position: value.end_position,
            listened_at: value.listened_at,
        }
    }
}

impl From<NewListeningEvent> for NewListeningEventEntity {
    fn from(value: NewListeningEvent) -> Self {
        Self {
            user_id: value.user_id,
            device: value.device,
            podcast_episode_id: value.podcast_episode_id,
            podcast_id: value.podcast_id,
            podcast_episode_db_id: value.podcast_episode_db_id,
            delta_seconds: value.delta_seconds,
            start_position: value.start_position,
            end_position: value.end_position,
            listened_at: value.listened_at,
        }
    }
}

pub struct DieselListeningEventRepository {
    database: Database,
}

impl DieselListeningEventRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl ListeningEventRepository for DieselListeningEventRepository {
    type Error = PersistenceError;

    fn create(&self, event: NewListeningEvent) -> Result<ListeningEvent, Self::Error> {
        use self::listening_events::dsl::listening_events;

        diesel::insert_into(listening_events)
            .values(NewListeningEventEntity::from(event))
            .get_result::<ListeningEventEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn get_by_user_and_range(
        &self,
        user_id_to_search: i32,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
    ) -> Result<Vec<ListeningEvent>, Self::Error> {
        use self::listening_events::dsl as le_dsl;
        use self::listening_events::table as le_table;

        let mut query = le_table
            .filter(le_dsl::user_id.eq(user_id_to_search))
            .into_boxed();

        if let Some(from) = from {
            query = query.filter(le_dsl::listened_at.ge(from));
        }

        if let Some(to) = to {
            query = query.filter(le_dsl::listened_at.le(to));
        }

        query
            .order_by(le_dsl::listened_at.asc())
            .load::<ListeningEventEntity>(&mut self.database.connection()?)
            .map(|events| events.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id_to_search: i32) -> Result<usize, Self::Error> {
        use self::listening_events::dsl as le_dsl;
        use self::listening_events::table as le_table;

        diesel::delete(le_table.filter(le_dsl::user_id.eq(user_id_to_search)))
            .execute(&mut self.database.connection()?)
            .map_err(Into::into)
    }

    fn delete_by_podcast_id(&self, podcast_id_to_delete: i32) -> Result<usize, Self::Error> {
        use self::listening_events::dsl as le_dsl;
        use self::listening_events::table as le_table;

        diesel::delete(le_table.filter(le_dsl::podcast_id.eq(podcast_id_to_delete)))
            .execute(&mut self.database.connection()?)
            .map_err(Into::into)
    }
}
