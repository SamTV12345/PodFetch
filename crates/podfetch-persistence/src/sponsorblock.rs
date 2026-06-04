use crate::db::get_connection;
use crate::schema::{episode_sponsor_segments, sponsorblock_user_settings};
use chrono::NaiveDateTime;
use common_infrastructure::db::PersistenceError;
use common_infrastructure::error::CustomError;
use diesel::prelude::*;

fn to_custom_err(e: diesel::result::Error) -> CustomError {
    CustomError::from(PersistenceError::from(e))
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone, PartialEq)]
#[diesel(table_name = episode_sponsor_segments)]
pub struct SponsorSegmentEntity {
    pub id: String,
    pub episode_id: String,
    pub uuid: String,
    pub category: String,
    pub action_type: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub votes: i32,
    pub locked: bool,
    pub duration_mismatch: bool,
    pub fetched_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Debug, Clone, PartialEq)]
#[diesel(table_name = sponsorblock_user_settings)]
#[diesel(primary_key(user_id))]
pub struct SponsorblockUserSettingsEntity {
    pub user_id: String,
    pub enabled: bool,
    pub skip_sponsor: bool,
    pub skip_selfpromo: bool,
    pub skip_interaction: bool,
    pub skip_intro: bool,
    pub skip_outro: bool,
    pub skip_preview: bool,
    pub skip_filler: bool,
    pub skip_music_offtopic: bool,
}

pub struct SponsorblockRepository;

impl SponsorblockRepository {
    /// Idempotently replace ALL stored segments for an episode with `segments`.
    pub fn replace_segments_for_episode(
        episode_id_value: &str,
        segments: Vec<SponsorSegmentEntity>,
    ) -> Result<(), CustomError> {
        use self::episode_sponsor_segments::dsl as s;
        let mut conn = get_connection();
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(s::episode_sponsor_segments.filter(s::episode_id.eq(episode_id_value)))
                .execute(conn)?;
            for segment in &segments {
                diesel::insert_into(s::episode_sponsor_segments)
                    .values(segment)
                    .execute(conn)?;
            }
            Ok(())
        })
        .map_err(to_custom_err)
    }

    pub fn get_segments_for_episode(
        episode_id_value: &str,
    ) -> Result<Vec<SponsorSegmentEntity>, CustomError> {
        use self::episode_sponsor_segments::dsl as s;
        s::episode_sponsor_segments
            .filter(s::episode_id.eq(episode_id_value))
            .order(s::start_ms.asc())
            .load::<SponsorSegmentEntity>(&mut get_connection())
            .map_err(to_custom_err)
    }

    pub fn get_user_settings(
        user_id_value: &str,
    ) -> Result<Option<SponsorblockUserSettingsEntity>, CustomError> {
        use self::sponsorblock_user_settings::dsl as u;
        u::sponsorblock_user_settings
            .filter(u::user_id.eq(user_id_value))
            .first::<SponsorblockUserSettingsEntity>(&mut get_connection())
            .optional()
            .map_err(to_custom_err)
    }

    pub fn upsert_user_settings(
        settings: SponsorblockUserSettingsEntity,
    ) -> Result<(), CustomError> {
        use self::sponsorblock_user_settings::dsl as u;
        let mut conn = get_connection();
        let existing = u::sponsorblock_user_settings
            .filter(u::user_id.eq(&settings.user_id))
            .first::<SponsorblockUserSettingsEntity>(&mut conn)
            .optional()
            .map_err(to_custom_err)?;
        match existing {
            Some(_) => {
                diesel::update(
                    u::sponsorblock_user_settings.filter(u::user_id.eq(&settings.user_id)),
                )
                .set(&settings)
                .execute(&mut conn)
                .map_err(to_custom_err)?;
            }
            None => {
                diesel::insert_into(u::sponsorblock_user_settings)
                    .values(&settings)
                    .execute(&mut conn)
                    .map_err(to_custom_err)?;
            }
        }
        Ok(())
    }
}
