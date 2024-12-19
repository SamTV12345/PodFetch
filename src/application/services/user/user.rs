use crate::adapters::persistence::repositories::user::user::UserRepository;
use crate::constants::inner_constants::{Role, ENVIRONMENT_SERVICE};
use crate::domain::models::user::user::User;
use crate::utils::error::CustomError;

pub struct UserService;


impl UserService {
    pub fn insert_user(user: User) -> Result<User, CustomError> {
        let env_service = ENVIRONMENT_SERVICE.get().unwrap();
        if let Some(res) = env_service.username.clone() {
            if res == user.username {
                return Err(CustomError::Conflict(
                    "Username already exists".to_string(),
                ))
            }
        }

        if let Some(_) = UserRepository::find_by_username(&user.username)? {
            return Err(CustomError::Conflict(
                "Username already exists".to_string(),
            ));
        }

        UserRepository::insert_user(user)
    }

    pub fn is_privileged_user(user: User) -> bool {
        user.role.eq(&Role::Admin) || user.role.eq(&Role::Uploader)
    }

    pub fn is_admin(user: User) -> bool {
        user.role.eq(&Role::Admin)
    }

    pub fn check_if_api_key_exists(api_key: &str) -> Result<bool, CustomError> {
        return UserRepository::check_if_api_key_exists(api_key)
    }
}