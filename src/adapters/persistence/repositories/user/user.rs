use std::io::Error;
use actix_web::HttpResponse;
use diesel::{OptionalExtension, RunQueryDsl};
use diesel::associations::HasTable;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::users;
use crate::adapters::persistence::model::user::user::UserEntity;
use crate::constants::inner_constants::{Role, BASIC_AUTH, ENVIRONMENT_SERVICE, OIDC_AUTH, STANDARD_USER, USERNAME};
use crate::domain::models::user::user::User;
use crate::utils::environment_variables::is_env_var_present_and_true;
use crate::utils::error::{map_db_error, CustomError};

pub struct UserRepository;
use diesel::ExpressionMethods;
use diesel::QueryDsl;

impl UserRepository {
    pub fn find_by_username(
        username_to_find: &str,
    ) -> Result<Option<User>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;
        let env_service = ENVIRONMENT_SERVICE.get().unwrap();
        if let Some(res) = env_service.username.clone() {
            if res == username_to_find {
                return Ok(Option::from(Self::create_admin_user()));
            }
        }

        users
            .filter(username.eq(username_to_find))
            .first::<UserEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
            .map(|user| user.map(|u| u.into()))
    }

    pub fn insert_user(user: User) -> Result<User, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;

        diesel::insert_into(users::table())
            .values((
                username.eq(user.username.clone()),
                role.eq(user.role.to_string()),
                password.eq(user.password.clone()),
                created_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .get_result::<User>(&mut get_connection())
            .map_err(map_db_error)
    }

    pub fn delete_user(user: User) -> Result<usize, CustomError> {
        diesel::delete(users::table.filter(users::id.eq(user.id)))
            .execute(&mut get_connection())
            .map_err(map_db_error)
    }

    pub fn update_role(
        user: User
    ) -> Result<User, CustomError> {
        diesel::update(users::table.filter(users::id.eq(user.id)))
            .set(users::role.eq(user.role.to_string()))
            .get_result::<UserEntity>(&mut get_connection())
            .map_err(map_db_error)
            .map(|u| u.into())
    }

    pub fn update_explicit_consent(
        user: User
    ) -> Result<User, diesel::result::Error> {
        diesel::update(users::table.filter(users::id.eq(user.id)))
            .set(users::explicit_consent.eq(user.explicit_consent))
            .get_result::<UserEntity>(&mut get_connection())
            .map_err(map_db_error)?
            .into()
    }

    pub(crate) fn create_admin_user() -> User {
        let env_service = ENVIRONMENT_SERVICE.get().unwrap();
        let password: Option<String> = env_service.password.clone();
        let username = env_service.username.clone();
        User {
            id: 9999,
            username: username.unwrap_or(STANDARD_USER.to_string()),
            role: Role::Admin,
            password,
            explicit_consent: true,
            created_at: Default::default(),
            api_key: env_service.api_key_admin.clone(),
        }
    }

    pub fn find_all_users() ->
                                                                                     Result<Vec<User>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;

        users.load::<UserEntity>(&mut get_connection()).map_err(map_db_error).map(|users| {
            users.into_iter().map(|u| u.into()).collect()
        })
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
        Ok(auth_header.unwrap().to_str().unwrap().parse().unwrap())
    }

    pub fn check_if_admin(
        username: &Option<String>,
    ) -> Result<(), CustomError> {
        if let Some(username_unwrapped) = username {
            let found_user = Self::find_by_username(username_unwrapped)?;

            if let Some(user) = found_user {
                if user.role != Role::Admin {
                    return Err(CustomError::Forbidden);
                }
                return Ok(());
            }
        }
        Err(CustomError::Forbidden)
    }

    pub fn delete_by_username(
        username_to_search: String,
        conn: &mut crate::adapters::persistence::dbconfig::DBType,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;
        diesel::delete(users.filter(username.eq(username_to_search)))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn update_user(user: User) -> Result<User, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user.id)))
            .set(user)
            .get_result(&mut get_connection())
            .map_err(map_db_error)
    }



    pub fn get_user_by_userid(user_id: i32) -> Result<User, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;
        let user = users
            .filter(id.eq(user_id))
            .first::<User>(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;
        if user.is_none() {
            return Err(CustomError::NotFound);
        }
        Ok(user.unwrap())
    }



    pub fn find_by_api_key(
        api_key_to_find: &str,
    ) -> Result<Option<User>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;

        users
            .filter(api_key.eq(api_key_to_find))
            .first::<User>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
    }

    pub fn update_api_key_of_user(
        username_to_update: &str,
        api_key_to_update: String,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;

        diesel::update(users.filter(username.eq(username_to_update)))
            .set(api_key.eq(api_key_to_update))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;

        Ok(())
    }

    pub fn check_if_api_key_exists(api_key_to_find: &str) -> Result<bool, CustomError> {
        if api_key_to_find.is_empty() {
            return Ok(false);
        }

        let env_service = ENVIRONMENT_SERVICE.get().unwrap();


        if let Some(res) = env_service.api_key_admin.clone() {
            if !res.is_empty() && res == api_key_to_find {
                return Ok(true);
            }
        }

        let result = Self::find_by_api_key(api_key_to_find);

        match result {
            Ok(user) => {
                if let Some(user) = user {
                    return Ok(user.api_key.is_some());
                }
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }
}