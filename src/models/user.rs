use crate::constants::inner_constants::{Role, BASIC_AUTH, ENVIRONMENT_SERVICE, OIDC_AUTH, STANDARD_USER, USERNAME};
use crate::dbconfig::schema::users;
use crate::dbconfig::DBType;
use crate::utils::environment_variables::is_env_var_present_and_true;
use crate::utils::error::{map_db_error, CustomError};
use crate::DBType as DbConnection;
use actix_web::HttpResponse;
use chrono::NaiveDateTime;
use diesel::associations::HasTable;
use diesel::prelude::{Insertable, Queryable};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{AsChangeset, OptionalExtension, RunQueryDsl};
use std::io::Error;
use utoipa::ToSchema;

#[derive(
    Serialize, Deserialize, Queryable, Insertable, Clone, ToSchema, PartialEq, Debug, AsChangeset,
)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub password: Option<String>,
    pub explicit_consent: bool,
    pub created_at: NaiveDateTime,
    pub api_key: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserWithoutPassword {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub explicit_consent: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserWithAPiKey {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub explicit_consent: bool,
    pub api_key: Option<String>,
}

impl User {
    pub fn new(
        id: i32,
        username: String,
        role: Role,
        password: Option<String>,
        created_at: NaiveDateTime,
        explicit_consent: bool,
    ) -> Self {
        User {
            id,
            username,
            role: role.to_string(),
            password,
            created_at,
            explicit_consent,
            api_key: None,
        }
    }

    pub fn find_by_username(
        username_to_find: &str,
        conn: &mut DbConnection,
    ) -> Result<User, CustomError> {
        use crate::dbconfig::schema::users::dsl::*;
        let env_service = ENVIRONMENT_SERVICE.get().unwrap();
        if let Some(res) = env_service.username.clone() {
            if res == username_to_find {
                return Ok(User::create_admin_user());
            }
        }

        let opt_user = users
            .filter(username.eq(username_to_find))
            .first::<User>(conn)
            .optional()
            .map_err(map_db_error)?;
        if let Some(user) = opt_user {
            Ok(user)
        } else {
            Err(CustomError::NotFound)
        }
    }

    pub fn insert_user(&mut self, conn: &mut DbConnection) -> Result<User, Error> {
        use crate::dbconfig::schema::users::dsl::*;
        let env_service = ENVIRONMENT_SERVICE.get().unwrap();
        if let Some(res) = env_service.username.clone() {
            if res == self.username {
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    "Username already exists",
                ));
            }
        }

        let res = diesel::insert_into(users::table())
            .values((
                username.eq(self.username.clone()),
                role.eq(self.role.clone()),
                password.eq(self.password.clone()),
                created_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .get_result::<User>(conn)
            .unwrap();
        Ok(res)
    }

    pub fn delete_user(&self, conn: &mut DbConnection) -> Result<usize, CustomError> {
        diesel::delete(users::table.filter(users::id.eq(self.id)))
            .execute(conn)
            .map_err(map_db_error)
    }

    pub fn update_role(
        &self,
        conn: &mut DbConnection,
    ) -> Result<UserWithoutPassword, diesel::result::Error> {
        let user = diesel::update(users::table.filter(users::id.eq(self.id)))
            .set(users::role.eq(self.role.clone()))
            .get_result::<User>(conn);

        Ok(User::map_to_dto(user.unwrap()))
    }

    pub fn update_explicit_consent(
        &self,
        conn: &mut DbConnection,
    ) -> Result<UserWithoutPassword, diesel::result::Error> {
        let user = diesel::update(users::table.filter(users::id.eq(self.id)))
            .set(users::explicit_consent.eq(self.explicit_consent))
            .get_result::<User>(conn);

        Ok(User::map_to_dto(user?))
    }

    pub(crate) fn create_admin_user() -> User {
        let env_service = ENVIRONMENT_SERVICE.get().unwrap();
        let password: Option<String> = env_service.password.clone();
        let username = env_service.username.clone();
        User {
            id: 9999,
            username: username.unwrap_or(STANDARD_USER.to_string()),
            role: Role::Admin.to_string(),
            password,
            explicit_consent: true,
            created_at: Default::default(),
            api_key: env_service.api_key_admin.clone(),
        }
    }

    pub fn map_to_dto(user: Self) -> UserWithoutPassword {
        UserWithoutPassword {
            id: user.id,
            explicit_consent: user.explicit_consent,
            username: user.username.clone(),
            role: user.role.clone(),
            created_at: user.created_at,
        }
    }

    pub fn map_to_api_dto(user: Self) -> UserWithAPiKey {
        UserWithAPiKey {
            id: user.id,
            explicit_consent: user.explicit_consent,
            username: user.username.clone(),
            role: user.role.clone(),
            created_at: user.created_at,
            api_key: user.api_key.clone(),
        }
    }

    pub fn create_standard_admin_user() -> User {
        let env_service = ENVIRONMENT_SERVICE.get().unwrap();
        let api_admin: Option<String> = env_service.api_key_admin.clone();
        User {
            id: 9999,
            username: STANDARD_USER.to_string(),
            role: Role::Admin.to_string(),
            password: None,
            explicit_consent: true,
            created_at: Default::default(),
            api_key: api_admin,
        }
    }

    pub fn find_all_users(conn: &mut DbConnection) -> Vec<UserWithoutPassword> {
        use crate::dbconfig::schema::users::dsl::*;

        let loaded_users = users.load::<User>(conn).unwrap();
        loaded_users.into_iter().map(User::map_to_dto).collect()
    }

    /**
     * Returns the username from the request header if the BASIC_AUTH environment variable is set to true
     * Otherwise returns None
     */
    pub fn get_username_from_req_header(
        req: &actix_web::HttpRequest,
    ) -> Result<Option<String>, Error> {
        if is_env_var_present_and_true(BASIC_AUTH) || is_env_var_present_and_true(OIDC_AUTH) {
            let auth_header = req.headers().get(USERNAME);
            if auth_header.is_none() {
                return Err(Error::new(std::io::ErrorKind::Other, "Username not found"));
            }
            return Ok(Some(
                auth_header.unwrap().to_str().unwrap().parse().unwrap(),
            ));
        }
        Ok(None)
    }

    pub fn get_gpodder_req_header(req: &actix_web::HttpRequest) -> Result<String, Error> {
        let auth_header = req.headers().get(USERNAME);
        if auth_header.is_none() {
            return Err(Error::new(std::io::ErrorKind::Other, "Username not found"));
        }
        return Ok(auth_header.unwrap().to_str().unwrap().parse().unwrap());
    }

    pub fn check_if_admin_or_uploader(
        username: &Option<String>,
        conn: &mut DbConnection,
    ) -> Result<Option<HttpResponse>, CustomError> {
        if let Some(username) = username {
            let found_user = User::find_by_username(username, conn)?;
            if found_user.role.ne(&Role::Admin.to_string())
                && found_user.role.ne(&Role::Uploader.to_string())
            {
                return Err(CustomError::Forbidden);
            }
        }
        Ok(None)
    }

    pub fn check_if_admin(
        username: &Option<String>,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        if let Some(username_unwrapped) = username {
            let found_user = User::find_by_username(username_unwrapped, conn)?;

            if found_user.role != Role::Admin.to_string() {
                return Err(CustomError::Forbidden);
            }
            return Ok(());
        }
        Err(CustomError::Forbidden)
    }

    pub fn delete_by_username(
        username_to_search: String,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::users::dsl::*;
        diesel::delete(users.filter(username.eq(username_to_search)))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn update_user(user: User, conn: &mut DbConnection) -> Result<User, CustomError> {
        use crate::dbconfig::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user.clone().id)))
            .set(user)
            .get_result(conn)
            .map_err(map_db_error)
    }

    pub fn is_privileged_user(&self) -> bool {
        self.role.eq(&Role::Admin.to_string()) || self.role.eq(&Role::Uploader.to_string())
    }

    pub fn get_user_by_userid(user_id: i32, conn: &mut DbConnection) -> Result<User, CustomError> {
        use crate::dbconfig::schema::users::dsl::*;
        let user = users
            .filter(id.eq(user_id))
            .first::<User>(conn)
            .optional()
            .map_err(map_db_error)?;
        if user.is_none() {
            return Err(CustomError::NotFound);
        }
        Ok(user.unwrap())
    }

    pub fn is_admin(&self) -> bool {
        self.role.eq(&Role::Admin.to_string())
    }

    pub fn find_by_api_key(
        api_key_to_find: String,
        conn: &mut DBType,
    ) -> Result<Option<User>, CustomError> {
        use crate::dbconfig::schema::users::dsl::*;

        users
            .filter(api_key.eq(api_key_to_find))
            .first::<User>(conn)
            .optional()
            .map_err(map_db_error)
    }

    pub fn update_api_key_of_user(
        username_to_update: &str,
        api_key_to_update: String,
        conn: &mut DBType,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::users::dsl::*;

        diesel::update(users.filter(username.eq(username_to_update)))
            .set(api_key.eq(api_key_to_update))
            .execute(conn)
            .map_err(map_db_error)?;

        Ok(())
    }

    pub fn check_if_api_key_exists(api_key_to_find: String, conn: &mut DBType) -> bool {
        if api_key_to_find.is_empty() {
            return false;
        }

        let env_service = ENVIRONMENT_SERVICE.get().unwrap();


        if let Some(res) = env_service.api_key_admin.clone() {
            if !res.is_empty() && res == api_key_to_find {
                return true;
            }
        }

        let result = Self::find_by_api_key(api_key_to_find, conn);

        match result {
            Ok(user) => {
                if let Some(user) = user {
                    return user.api_key.is_some();
                }
                false
            }
            Err(_) => false,
        }
    }
}
