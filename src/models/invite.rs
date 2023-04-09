use chrono::NaiveDateTime;
use crate::schema::invites;
use utoipa::ToSchema;
use diesel::{Queryable, Insertable, Identifiable, AsChangeset, SqliteConnection, RunQueryDsl, QueryDsl, OptionalExtension};
use diesel::associations::HasTable;
use diesel::ExpressionMethods;
use uuid::Uuid;

#[derive(Queryable, Insertable, AsChangeset, Identifiable, Serialize, Deserialize, Clone, ToSchema)]
pub struct Invite{
    pub id: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub accepted_at: Option<NaiveDateTime>,
    pub expires_at: NaiveDateTime
}


impl Invite{
    pub fn new(id: String, role: String, created_at: NaiveDateTime, accepted_at: Option<NaiveDateTime>, expires_at: NaiveDateTime) -> Self {
        Invite {
            id,
            role,
            created_at,
            accepted_at,
            expires_at
        }
    }

    pub fn insert_invite(&self, conn: &mut SqliteConnection) -> Result<Invite, diesel::result::Error> {
        use crate::schema::invites::dsl::*;

        let now = chrono::Utc::now().naive_utc();

        diesel::insert_into(invites::table())
            .values(
                (
                    id.eq(Uuid::new_v4().to_string()),
                    role.eq(self.role.clone()),
                    created_at.eq(now),
                    expires_at.eq(now + chrono::Duration::days(7)))
            )
            .get_result(conn)
    }

    pub fn find_invite(id: String, conn: &mut SqliteConnection) -> Result<Option<Invite>, diesel::result::Error> {
        invites::table
            .filter(invites::id.eq(id))
            .first::<Invite>(conn)
            .optional()
    }
}