use crate::db::{Database, PersistenceError};
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::user_admin::{ManagedUser, UserAdminRepository};

diesel::table! {
    users (id) {
        id -> Integer,
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
    id: i32,
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
            id: value.id,
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
            id: value.id,
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
        diesel::update(users.filter(id.eq(user.id)))
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
