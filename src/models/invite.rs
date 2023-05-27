use std::io::Error;
use chrono::NaiveDateTime;
use crate::dbconfig::schema::invites;
use utoipa::ToSchema;
use diesel::{Queryable, Insertable, Identifiable, RunQueryDsl, QueryDsl, OptionalExtension};
use diesel::associations::HasTable;
use diesel::ExpressionMethods;
use uuid::Uuid;
use crate::constants::constants::Role;
use crate::DbConnection;

#[derive(Queryable, Insertable, Identifiable, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Invite{
    pub id: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub accepted_at: Option<NaiveDateTime>,
    pub explicit_consent: bool,
    pub expires_at: NaiveDateTime
}


impl Invite{
    pub fn new(id: String, role: String, created_at: NaiveDateTime, accepted_at:
    Option<NaiveDateTime>, expires_at: NaiveDateTime, explicit_consent_i: bool) -> Self {
        Invite {
            id,
            role,
            created_at,
            accepted_at,
            explicit_consent:explicit_consent_i,
            expires_at
        }
    }

    pub fn insert_invite(role_to_insert: &Role, explicit_consent_to_insert: bool, conn: &mut
    DbConnection) -> Result<Invite,
        Error> {
        use crate::dbconfig::schema::invites::dsl::*;

        let now = chrono::Utc::now().naive_utc();

        let created_invite = diesel::insert_into(invites::table())
            .values(
                (
                    id.eq(Uuid::new_v4().to_string()),
                    role.eq(role_to_insert.to_string()),
                    explicit_consent.eq(explicit_consent_to_insert),
                    created_at.eq(now),
                    expires_at.eq(now + chrono::Duration::days(7)))
            )
            .get_result::<Invite>(conn)
            .unwrap();

        Ok(created_invite)
    }

    pub fn find_invite(id: String, conn: &mut DbConnection) -> Result<Option<Invite>, diesel::result::Error> {
        invites::table
            .filter(invites::id.eq(id))
            .first::<Invite>(conn)
            .optional()
    }

    pub fn find_all_invites(conn: &mut DbConnection) -> Result<Vec<Invite>, diesel::result::Error> {
        invites::table
            .load::<Invite>(conn)
    }

    pub fn invalidate_invite(invite_id: String, conn: &mut DbConnection) -> Result<(),
        diesel::result::Error> {
        use crate::dbconfig::schema::invites::dsl::*;

        diesel::update(invites.filter(id.eq(invite_id)))
            .set(accepted_at.eq(chrono::Utc::now().naive_utc()))
            .execute(conn)?;

        Ok(())
    }

    pub fn delete_invite(invite_id: String, conn: &mut DbConnection) -> Result<(), diesel::result::Error> {
        use crate::dbconfig::schema::invites::dsl::*;

        diesel::delete(invites.filter(id.eq(invite_id)))
            .execute(conn)?;

        Ok(())
    }
}