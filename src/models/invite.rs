use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::invites;
use crate::constants::inner_constants::Role;
use crate::utils::error::ErrorSeverity::Critical;
use crate::utils::error::{map_db_error, CustomError};
use chrono::NaiveDateTime;
use diesel::associations::HasTable;
use diesel::ExpressionMethods;
use diesel::{Identifiable, Insertable, OptionalExtension, QueryDsl, Queryable, RunQueryDsl};
use std::io::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Insertable, Identifiable, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Invite {
    pub id: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub accepted_at: Option<NaiveDateTime>,
    pub explicit_consent: bool,
    pub expires_at: NaiveDateTime,
}

impl Invite {
    pub fn new(
        id: String,
        role: String,
        created_at: NaiveDateTime,
        accepted_at: Option<NaiveDateTime>,
        expires_at: NaiveDateTime,
        explicit_consent_i: bool,
    ) -> Self {
        Invite {
            id,
            role,
            created_at,
            accepted_at,
            explicit_consent: explicit_consent_i,
            expires_at,
        }
    }

    pub fn insert_invite(
        role_to_insert: &Role,
        explicit_consent_to_insert: bool,
    ) -> Result<Invite, Error> {
        use crate::adapters::persistence::dbconfig::schema::invites::dsl::*;

        let now = chrono::Utc::now().naive_utc();

        let created_invite = diesel::insert_into(invites::table())
            .values((
                id.eq(Uuid::new_v4().to_string()),
                role.eq(role_to_insert.to_string()),
                explicit_consent.eq(explicit_consent_to_insert),
                created_at.eq(now),
                expires_at.eq(now + chrono::Duration::days(7)),
            ))
            .get_result::<Invite>(&mut get_connection())
            .unwrap();

        Ok(created_invite)
    }

    pub fn find_invite(id: String) -> Result<Option<Invite>, CustomError> {
        invites::table
            .filter(invites::id.eq(id))
            .first::<Invite>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))
    }

    pub fn find_all_invites() -> Result<Vec<Invite>, diesel::result::Error> {
        invites::table.load::<Invite>(&mut get_connection())
    }

    pub fn invalidate_invite(invite_id: String) -> Result<(), diesel::result::Error> {
        use crate::adapters::persistence::dbconfig::schema::invites::dsl::*;

        diesel::update(invites.filter(id.eq(invite_id)))
            .set(accepted_at.eq(chrono::Utc::now().naive_utc()))
            .execute(&mut get_connection())?;

        Ok(())
    }

    pub fn delete_invite(invite_id: String) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::invites::dsl::*;

        diesel::delete(invites.filter(id.eq(invite_id)))
            .execute(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;

        Ok(())
    }
}
