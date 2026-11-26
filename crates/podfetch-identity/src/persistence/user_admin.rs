use crate::domain::user_admin::{ManagedUser, UserAdminRepository};
use common_infrastructure::db::{Database, PersistenceError};
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use uuid::Uuid;

diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        role -> Text,
        password -> Nullable<Text>,
        explicit_consent -> Bool,
        created_at -> Timestamp,
        api_key -> Nullable<Text>,
        country -> Nullable<Text>,
        language -> Nullable<Text>,
    }
}

#[derive(Queryable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = users)]
struct UserEntity {
    id: String,
    username: String,
    role: String,
    password: Option<String>,
    explicit_consent: bool,
    created_at: chrono::NaiveDateTime,
    api_key: Option<String>,
    country: Option<String>,
    language: Option<String>,
}

impl From<UserEntity> for ManagedUser {
    fn from(value: UserEntity) -> Self {
        Self {
            id: Uuid::parse_str(&value.id).expect("valid uuid in db"),
            username: value.username,
            role: value.role,
            password: value.password,
            explicit_consent: value.explicit_consent,
            created_at: value.created_at,
            api_key: value.api_key,
            country: value.country,
            language: value.language,
        }
    }
}

impl From<ManagedUser> for UserEntity {
    fn from(value: ManagedUser) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            role: value.role,
            password: value.password,
            explicit_consent: value.explicit_consent,
            created_at: value.created_at,
            api_key: value.api_key,
            country: value.country,
            language: value.language,
        }
    }
}

pub struct DieselUserAdminRepository {
    database: Database,
}

impl DieselUserAdminRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl UserAdminRepository for DieselUserAdminRepository {
    type Error = PersistenceError;

    fn create(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        use self::users::dsl::*;

        let mut conn = self.database.connection()?;
        diesel::insert_into(users)
            .values((
                id.eq(uuid::Uuid::now_v7().to_string()),
                username.eq(user.username),
                role.eq(user.role),
                password.eq(user.password),
                explicit_consent.eq(user.explicit_consent),
                created_at.eq(chrono::Utc::now().naive_utc()),
                api_key.eq(user.api_key),
                country.eq(user.country),
                language.eq(user.language),
            ))
            .get_result::<UserEntity>(&mut conn)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn find_by_id(&self, id_to_find: Uuid) -> Result<Option<ManagedUser>, Self::Error> {
        use self::users::dsl::*;

        let mut conn = self.database.connection()?;
        users
            .filter(id.eq(id_to_find.to_string()))
            .first::<UserEntity>(&mut conn)
            .optional()
            .map(|user| user.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_username(&self, username_to_find: &str) -> Result<Option<ManagedUser>, Self::Error> {
        use self::users::dsl::*;

        let mut conn = self.database.connection()?;
        users
            .filter(username.eq(username_to_find))
            .first::<UserEntity>(&mut conn)
            .optional()
            .map(|user| user.map(Into::into))
            .map_err(Into::into)
    }

    fn ensure_with_id(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        use self::users::dsl::*;

        let mut conn = self.database.connection()?;
        let entity = UserEntity::from(user);

        // Check-then-insert keeps this idempotent without relying on
        // backend-specific upsert (`on_conflict`), which is not supported
        // uniformly across the `MultiConnection` backends. The seed runs once at
        // startup on a single thread, so the gap between the two queries is safe.
        if let Some(existing) = users
            .filter(id.eq(entity.id.clone()))
            .first::<UserEntity>(&mut conn)
            .optional()?
        {
            return Ok(existing.into());
        }

        diesel::insert_into(users)
            .values(&entity)
            .get_result::<UserEntity>(&mut conn)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn find_by_api_key(&self, api_key_to_find: &str) -> Result<Option<ManagedUser>, Self::Error> {
        use self::users::dsl::*;

        let mut conn = self.database.connection()?;
        users
            .filter(api_key.eq(api_key_to_find))
            .first::<UserEntity>(&mut conn)
            .optional()
            .map(|user| user.map(Into::into))
            .map_err(Into::into)
    }

    fn find_all(&self) -> Result<Vec<ManagedUser>, Self::Error> {
        use self::users::dsl::*;

        let mut conn = self.database.connection()?;
        users
            .load::<UserEntity>(&mut conn)
            .map(|loaded_users| loaded_users.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn update(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        use self::users::dsl::*;

        let mut conn = self.database.connection()?;
        let entity = UserEntity::from(user.clone());
        diesel::update(users.filter(id.eq(user.id.to_string())))
            .set(entity)
            .get_result::<UserEntity>(&mut conn)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn delete_by_username(&self, username_to_delete: &str) -> Result<(), Self::Error> {
        use self::users::dsl::*;

        let mut conn = self.database.connection()?;
        diesel::delete(users.filter(username.eq(username_to_delete)))
            .execute(&mut conn)
            .map(|_| ())
            .map_err(Into::into)
    }
}

/// Deletes the user named `username_to_delete` on `conn`, normally inside the
/// transaction that also publishes [`crate::events::UserDeleted`]. Returns the
/// id of the deleted user, `None` when there was none.
pub fn delete_by_username_on(
    conn: &mut common_infrastructure::db::DBType,
    username_to_delete: &str,
) -> Result<Option<Uuid>, diesel::result::Error> {
    use self::users::dsl::*;

    let Some(found) = users
        .filter(username.eq(username_to_delete))
        .select(id)
        .first::<String>(conn)
        .optional()?
    else {
        return Ok(None);
    };
    diesel::delete(users.filter(id.eq(&found))).execute(conn)?;
    Ok(Uuid::parse_str(&found).ok())
}
