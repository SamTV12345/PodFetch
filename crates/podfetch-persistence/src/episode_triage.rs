use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable, Selectable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::episode_triage::{EpisodeTriage, EpisodeTriageRepository, TriageStatus};
use uuid::Uuid;

diesel::table! {
    episode_triages (user_id, episode_id) {
        user_id -> Text,
        episode_id -> Text,
        status -> Text,
        updated_at -> Timestamp,
    }
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = episode_triages)]
struct EpisodeTriageEntity {
    user_id: String,
    episode_id: String,
    status: String,
    updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = episode_triages)]
struct EpisodeTriageInsertEntity {
    user_id: String,
    episode_id: String,
    status: String,
    updated_at: NaiveDateTime,
}

impl From<EpisodeTriageEntity> for EpisodeTriage {
    fn from(value: EpisodeTriageEntity) -> Self {
        Self {
            user_id: Uuid::parse_str(&value.user_id).expect("valid uuid in db"),
            episode_id: Uuid::parse_str(&value.episode_id).expect("valid uuid in db"),
            // A row whose status no longer parses (only possible after a manual
            // DB edit, given the CHECK constraint) is treated as dismissed so
            // it stays out of the inbox rather than crashing the listing.
            status: TriageStatus::from_string(&value.status).unwrap_or(TriageStatus::Dismissed),
            updated_at: value.updated_at,
        }
    }
}

impl From<EpisodeTriage> for EpisodeTriageInsertEntity {
    fn from(value: EpisodeTriage) -> Self {
        Self {
            user_id: value.user_id.to_string(),
            episode_id: value.episode_id.to_string(),
            status: value.status.as_str().to_string(),
            updated_at: value.updated_at,
        }
    }
}

pub struct DieselEpisodeTriageRepository {
    database: Database,
}

impl DieselEpisodeTriageRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl EpisodeTriageRepository for DieselEpisodeTriageRepository {
    type Error = PersistenceError;

    fn get(&self, user_id: Uuid, episode_id: Uuid) -> Result<Option<EpisodeTriage>, Self::Error> {
        use self::episode_triages::dsl as et_dsl;
        use self::episode_triages::table as et_table;

        et_table
            .filter(et_dsl::user_id.eq(user_id.to_string()))
            .filter(et_dsl::episode_id.eq(episode_id.to_string()))
            .first::<EpisodeTriageEntity>(&mut self.database.connection()?)
            .optional()
            .map(|triage| triage.map(Into::into))
            .map_err(Into::into)
    }

    fn upsert(&self, triage: EpisodeTriage) -> Result<(), Self::Error> {
        use self::episode_triages::dsl as et_dsl;
        use self::episode_triages::table as et_table;

        let user_id = triage.user_id.to_string();
        let episode_id = triage.episode_id.to_string();
        let existing = et_table
            .filter(et_dsl::user_id.eq(user_id.clone()))
            .filter(et_dsl::episode_id.eq(episode_id.clone()))
            .first::<EpisodeTriageEntity>(&mut self.database.connection()?)
            .optional()?;

        match existing {
            Some(_) => diesel::update(
                et_table
                    .filter(et_dsl::user_id.eq(user_id.clone()))
                    .filter(et_dsl::episode_id.eq(episode_id.clone())),
            )
            .set((
                et_dsl::status.eq(triage.status.as_str()),
                et_dsl::updated_at.eq(triage.updated_at),
            ))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into),
            None => diesel::insert_into(et_table)
                .values(EpisodeTriageInsertEntity::from(triage))
                .execute(&mut self.database.connection()?)
                .map(|_| ())
                .map_err(Into::into),
        }
    }

    fn delete(&self, user_id: Uuid, episode_id: Uuid) -> Result<usize, Self::Error> {
        use self::episode_triages::dsl as et_dsl;
        use self::episode_triages::table as et_table;

        diesel::delete(
            et_table
                .filter(et_dsl::user_id.eq(user_id.to_string()))
                .filter(et_dsl::episode_id.eq(episode_id.to_string())),
        )
        .execute(&mut self.database.connection()?)
        .map_err(Into::into)
    }

    fn delete_by_episode_id(&self, episode_id: Uuid) -> Result<usize, Self::Error> {
        use self::episode_triages::dsl as et_dsl;
        use self::episode_triages::table as et_table;

        diesel::delete(et_table.filter(et_dsl::episode_id.eq(episode_id.to_string())))
            .execute(&mut self.database.connection()?)
            .map_err(Into::into)
    }

    fn list_episode_ids_by_status(
        &self,
        user_id: Uuid,
        status: TriageStatus,
    ) -> Result<Vec<Uuid>, Self::Error> {
        use self::episode_triages::dsl as et_dsl;
        use self::episode_triages::table as et_table;

        et_table
            .filter(et_dsl::user_id.eq(user_id.to_string()))
            .filter(et_dsl::status.eq(status.as_str()))
            .select(et_dsl::episode_id)
            .load::<String>(&mut self.database.connection()?)
            .map(|ids| {
                ids.into_iter()
                    .filter_map(|id| Uuid::parse_str(&id).ok())
                    .collect()
            })
            .map_err(Into::into)
    }

    fn list_triaged_episode_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, Self::Error> {
        use self::episode_triages::dsl as et_dsl;
        use self::episode_triages::table as et_table;

        et_table
            .filter(et_dsl::user_id.eq(user_id.to_string()))
            .select(et_dsl::episode_id)
            .load::<String>(&mut self.database.connection()?)
            .map(|ids| {
                ids.into_iter()
                    .filter_map(|id| Uuid::parse_str(&id).ok())
                    .collect()
            })
            .map_err(Into::into)
    }
}
