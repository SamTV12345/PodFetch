use crate::db::{Database, PersistenceError};
use chrono::{Duration, Utc};
use diesel::ExpressionMethods;
use diesel::prelude::{Identifiable, Insertable, Queryable};
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::invite::{Invite, InviteRepository};
use uuid::Uuid;

diesel::table! {
    invites (id) {
        id -> Text,
        role -> Text,
        created_at -> Timestamp,
        accepted_at -> Nullable<Timestamp>,
        explicit_consent -> Bool,
        expires_at -> Timestamp,
    }
}

#[derive(Queryable, Insertable, Identifiable, Clone)]
#[diesel(table_name = invites)]
struct InviteEntity {
    id: String,
    role: String,
    created_at: chrono::NaiveDateTime,
    accepted_at: Option<chrono::NaiveDateTime>,
    explicit_consent: bool,
    expires_at: chrono::NaiveDateTime,
}

impl From<InviteEntity> for Invite {
    fn from(value: InviteEntity) -> Self {
        Self {
            id: value.id,
            role: value.role,
            created_at: value.created_at,
            accepted_at: value.accepted_at,
            explicit_consent: value.explicit_consent,
            expires_at: value.expires_at,
        }
    }
}

pub struct DieselInviteRepository {
    database: Database,
}

impl DieselInviteRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl InviteRepository for DieselInviteRepository {
    type Error = PersistenceError;

    fn create(&self, role_to_insert: &str, explicit_consent_to_insert: bool) -> Result<Invite, Self::Error> {
        use self::invites::dsl::*;

        let now = Utc::now().naive_utc();
        let mut conn = self.database.connection()?;

        diesel::insert_into(invites)
            .values((
                id.eq(Uuid::new_v4().to_string()),
                role.eq(role_to_insert.to_string()),
                explicit_consent.eq(explicit_consent_to_insert),
                created_at.eq(now),
                expires_at.eq(now + Duration::days(7)),
            ))
            .get_result::<InviteEntity>(&mut conn)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn find_by_id(&self, invite_id: &str) -> Result<Option<Invite>, Self::Error> {
        use self::invites::dsl::*;
        let mut conn = self.database.connection()?;

        invites
            .filter(id.eq(invite_id))
            .first::<InviteEntity>(&mut conn)
            .optional()
            .map(|invite| invite.map(Into::into))
            .map_err(Into::into)
    }

    fn find_all(&self) -> Result<Vec<Invite>, Self::Error> {
        use self::invites::dsl::*;
        let mut conn = self.database.connection()?;

        invites
            .load::<InviteEntity>(&mut conn)
            .map(|loaded_invites| loaded_invites.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn invalidate(&self, invite_id: &str) -> Result<(), Self::Error> {
        use self::invites::dsl::*;
        let mut conn = self.database.connection()?;

        diesel::update(invites.filter(id.eq(invite_id)))
            .set(accepted_at.eq(Utc::now().naive_utc()))
            .execute(&mut conn)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn delete(&self, invite_id: &str) -> Result<(), Self::Error> {
        use self::invites::dsl::*;
        let mut conn = self.database.connection()?;

        diesel::delete(invites.filter(id.eq(invite_id)))
            .execute(&mut conn)
            .map(|_| ())
            .map_err(Into::into)
    }
}
