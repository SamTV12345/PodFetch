use crate::sponsorblock::domain::{SkipPreferences, SponsorSegment, SponsorblockRepository};
use chrono::NaiveDateTime;
use common_infrastructure::db::{Database, PersistenceError};
use common_infrastructure::error::CustomError;
use diesel::prelude::*;
use podfetch_kernel::UserId;

diesel::table! {
    episode_sponsor_segments (id) {
        id -> Text,
        episode_id -> Text,
        uuid -> Text,
        category -> Text,
        action_type -> Text,
        start_ms -> BigInt,
        end_ms -> BigInt,
        votes -> Integer,
        locked -> Bool,
        duration_mismatch -> Bool,
        fetched_at -> Timestamp,
    }
}

diesel::table! {
    sponsorblock_user_settings (user_id) {
        user_id -> Text,
        enabled -> Bool,
        skip_sponsor -> Bool,
        skip_selfpromo -> Bool,
        skip_interaction -> Bool,
        skip_intro -> Bool,
        skip_outro -> Bool,
        skip_preview -> Bool,
        skip_filler -> Bool,
        skip_music_offtopic -> Bool,
    }
}

#[derive(Queryable, Insertable, Debug, Clone, PartialEq)]
#[diesel(table_name = episode_sponsor_segments)]
struct SegmentRow {
    id: String,
    episode_id: String,
    uuid: String,
    category: String,
    action_type: String,
    start_ms: i64,
    end_ms: i64,
    votes: i32,
    locked: bool,
    duration_mismatch: bool,
    fetched_at: NaiveDateTime,
}

impl SegmentRow {
    fn new(episode_id: &str, segment: &SponsorSegment) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            episode_id: episode_id.to_string(),
            uuid: segment.uuid.clone(),
            category: segment.category.clone(),
            action_type: segment.action_type.clone(),
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            votes: segment.votes,
            locked: segment.locked,
            duration_mismatch: segment.duration_mismatch,
            fetched_at: segment.fetched_at,
        }
    }
}

impl From<SegmentRow> for SponsorSegment {
    fn from(row: SegmentRow) -> Self {
        Self {
            uuid: row.uuid,
            category: row.category,
            action_type: row.action_type,
            start_ms: row.start_ms,
            end_ms: row.end_ms,
            votes: row.votes,
            locked: row.locked,
            duration_mismatch: row.duration_mismatch,
            fetched_at: row.fetched_at,
        }
    }
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, PartialEq)]
#[diesel(table_name = sponsorblock_user_settings)]
struct PreferencesRow {
    user_id: String,
    enabled: bool,
    skip_sponsor: bool,
    skip_selfpromo: bool,
    skip_interaction: bool,
    skip_intro: bool,
    skip_outro: bool,
    skip_preview: bool,
    skip_filler: bool,
    skip_music_offtopic: bool,
}

impl PreferencesRow {
    fn new(user_id: UserId, p: &SkipPreferences) -> Self {
        Self {
            user_id: user_id.to_string(),
            enabled: p.enabled,
            skip_sponsor: p.skip_sponsor,
            skip_selfpromo: p.skip_selfpromo,
            skip_interaction: p.skip_interaction,
            skip_intro: p.skip_intro,
            skip_outro: p.skip_outro,
            skip_preview: p.skip_preview,
            skip_filler: p.skip_filler,
            skip_music_offtopic: p.skip_music_offtopic,
        }
    }
}

impl From<PreferencesRow> for SkipPreferences {
    fn from(row: PreferencesRow) -> Self {
        Self {
            enabled: row.enabled,
            skip_sponsor: row.skip_sponsor,
            skip_selfpromo: row.skip_selfpromo,
            skip_interaction: row.skip_interaction,
            skip_intro: row.skip_intro,
            skip_outro: row.skip_outro,
            skip_preview: row.skip_preview,
            skip_filler: row.skip_filler,
            skip_music_offtopic: row.skip_music_offtopic,
        }
    }
}

pub struct DieselSponsorblockRepository {
    database: Database,
}

impl DieselSponsorblockRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl SponsorblockRepository for DieselSponsorblockRepository {
    fn replace_segments(
        &self,
        episode_id: &str,
        segments: &[SponsorSegment],
    ) -> Result<(), CustomError> {
        use episode_sponsor_segments::dsl as s;
        let rows: Vec<SegmentRow> = segments
            .iter()
            .map(|segment| SegmentRow::new(episode_id, segment))
            .collect();
        let mut conn = self.database.connection()?;
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(s::episode_sponsor_segments.filter(s::episode_id.eq(episode_id)))
                .execute(conn)?;
            for row in &rows {
                diesel::insert_into(s::episode_sponsor_segments)
                    .values(row)
                    .execute(conn)?;
            }
            Ok(())
        })
        .map_err(PersistenceError::from)?;
        Ok(())
    }

    fn forget_segments(&self, episode_ids: &[String]) -> Result<(), CustomError> {
        use episode_sponsor_segments::dsl as s;
        diesel::delete(s::episode_sponsor_segments.filter(s::episode_id.eq_any(episode_ids)))
            .execute(&mut self.database.connection()?)
            .map_err(PersistenceError::from)?;
        Ok(())
    }

    fn segments(&self, episode_id: &str) -> Result<Vec<SponsorSegment>, CustomError> {
        use episode_sponsor_segments::dsl as s;
        let rows = s::episode_sponsor_segments
            .filter(s::episode_id.eq(episode_id))
            .order(s::start_ms.asc())
            .load::<SegmentRow>(&mut self.database.connection()?)
            .map_err(PersistenceError::from)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    fn preferences(&self, user_id: UserId) -> Result<Option<SkipPreferences>, CustomError> {
        use sponsorblock_user_settings::dsl as u;
        let row = u::sponsorblock_user_settings
            .filter(u::user_id.eq(user_id.to_string()))
            .first::<PreferencesRow>(&mut self.database.connection()?)
            .optional()
            .map_err(PersistenceError::from)?;
        Ok(row.map(Into::into))
    }

    fn save_preferences(
        &self,
        user_id: UserId,
        preferences: &SkipPreferences,
    ) -> Result<(), CustomError> {
        use sponsorblock_user_settings::dsl as u;
        let row = PreferencesRow::new(user_id, preferences);
        let mut conn = self.database.connection()?;
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let updated =
                diesel::update(u::sponsorblock_user_settings.filter(u::user_id.eq(&row.user_id)))
                    .set(&row)
                    .execute(conn)?;
            if updated == 0 {
                diesel::insert_into(u::sponsorblock_user_settings)
                    .values(&row)
                    .execute(conn)?;
            }
            Ok(())
        })
        .map_err(PersistenceError::from)?;
        Ok(())
    }

    fn delete_preferences(&self, user_id: UserId) -> Result<(), CustomError> {
        use sponsorblock_user_settings::dsl as u;
        diesel::delete(u::sponsorblock_user_settings.filter(u::user_id.eq(user_id.to_string())))
            .execute(&mut self.database.connection()?)
            .map_err(PersistenceError::from)?;
        Ok(())
    }
}
