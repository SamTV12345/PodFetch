use std::io::Error;
use diesel::{OptionalExtension, RunQueryDsl};
use diesel::associations::HasTable;
use uuid::Uuid;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::invites;
use crate::adapters::persistence::model::invite::invite::InviteEntity;
use crate::constants::inner_constants::Role;
use crate::domain::models::invite::invite::Invite;
use crate::utils::error::{map_db_error, CustomError};

pub struct InviteRepository;
use diesel::ExpressionMethods;

impl InviteRepository {
    pub fn insert_invite(
        role_to_insert: &Role,
        explicit_consent_to_insert: bool,
    ) -> Result<InviteEntity, Error> {
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
            .get_result::<InviteEntity>(&mut get_connection())?;

        Ok(created_invite)
    }

    pub fn find_invite(id: &str) -> Result<Option<Invite>, CustomError> {
        invites::table
            .filter(invites::id.eq(id))
            .first::<InviteEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
            .map(|invite| invite.map(|i| i.into()))
    }

    pub fn find_all_invites() -> Result<Vec<Invite>, diesel::result::Error> {
        invites::table.load::<InviteEntity>(&mut get_connection())
            .map(|invites| invites.into_iter().map(|i| i.into()).collect())
    }

    pub fn invalidate_invite(
        invite_id: String,
    ) -> Result<(), diesel::result::Error> {
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
            .map_err(map_db_error)?;

        Ok(())
    }
}