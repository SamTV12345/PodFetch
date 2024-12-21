use std::str::FromStr;

use crate::constants::inner_constants::Role;
use crate::service::environment_service::EnvironmentService;
use crate::utils::error::CustomError;
use sha256::digest;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::application::services::user::invite::InviteService;
use crate::application::services::user::user::UserService;
use crate::domain::models::user::user::User;

pub struct UserManagementService {}

impl UserManagementService {
    pub fn may_onboard_user(user: User) -> bool {
        user.role == Role::Admin
    }

    pub fn is_valid_password(password: String) -> bool {
        let mut has_whitespace = false;
        let mut has_upper = false;
        let mut has_lower = false;
        let mut has_digit = false;

        for c in password.chars() {
            has_whitespace |= c.is_whitespace();
            has_lower |= c.is_lowercase();
            has_upper |= c.is_uppercase();
            has_digit |= c.is_ascii_digit();
        }

        !has_whitespace && has_upper && has_lower && has_digit && password.len() >= 8
    }

    /**
     * Performs the onboarding with a valid
     */
    pub fn onboard_user(
        username: String,
        password: String,
        invite_id: String,
    ) -> Result<User, CustomError> {
        // Check if the invite is valid
        match InviteService::find_invite(&invite_id) {
            Ok(invite) => {
                match invite {
                    Some(invite) => {
                        if invite.accepted_at.is_some() {
                            return Err(CustomError::Conflict(
                                "Invite already accepted".to_string(),
                            ));
                        }

                        let mut actual_user = User::new(
                            1,
                            username,
                            invite.role,
                            Some(digest(password.clone())),
                            chrono::Utc::now().naive_utc(),
                            invite.explicit_consent,
                        );

                        // This is safe as only when basic auth is enabled, the password is set
                        if Self::is_valid_password(password.clone()) {
                            match User::insert_user(&mut actual_user) {
                                Ok(user) => {
                                    Invite::invalidate_invite(invite_id.clone())
                                        .expect("Error invalidating invite");
                                    Ok(user)
                                }
                                Err(e) => {
                                    log::error!(
                                        "The following error occured when inserting a user {}",
                                        e
                                    );
                                    Err(CustomError::Unknown)
                                }
                            }
                        } else {
                            Err(CustomError::Conflict("Password is not valid".to_string()))
                        }
                    }
                    None => Err(CustomError::NotFound),
                }
            }
            Err(e) => {
                log::error!("The following error occured when finding an invite {}", e);
                Err(CustomError::NotFound)
            }
        }
    }

    pub fn create_invite(
        role: Role,
        explicit_consent_i: bool,
        user: User,
    ) -> Result<Invite, CustomError> {
        if Self::may_onboard_user(user) {
            let invite = Invite::insert_invite(&role, explicit_consent_i).expect(
                "Error \
            inserting invite",
            );
            return Ok(invite);
        }
        Err(CustomError::Forbidden)
    }

    pub fn delete_user(user: User) -> Result<usize, CustomError> {
        UserService::delete_user(&user)
    }

    pub fn update_user(
        user_to_update: User,
    ) -> Result<UserWithoutPassword, CustomError> {
        match User::update_role(&user_to_update) {
            Ok(user) => {
                match User::update_explicit_consent(&user_to_update) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("The following error occured when updating a user {}", e);
                        return Err(CustomError::Unknown);
                    }
                }
                Ok(user)
            }
            Err(e) => {
                log::error!("The following error occured when updating a user {}", e);
                Err(CustomError::Unknown)
            }
        }
    }

    pub fn get_invite_link(
        invite_id: String,
        environment_service: &EnvironmentService,
    ) -> Result<String, CustomError> {
        let invite = Invite::find_invite(invite_id)?;
        match invite {
            Some(invite) => Ok(environment_service.clone().server_url + "ui/invite/" + &invite.id),
            None => Err(CustomError::NotFound),
        }
    }

    pub fn get_invite(invite_id: String) -> Result<Invite, CustomError> {
        let invite = Invite::find_invite(invite_id)?;

        match invite {
            Some(invite) => {
                if invite.accepted_at.is_some() {
                    return Err(CustomError::Conflict("Invite already accepted".to_string()));
                }
                Ok(invite)
            }
            None => Err(CustomError::NotFound),
        }
    }

    pub fn get_invites() -> Result<Vec<Invite>, CustomError> {
        match Invite::find_all_invites() {
            Ok(invites) => Ok(invites),
            Err(e) => {
                log::error!("The following error occured when finding an invite {}", e);
                Err(CustomError::NotFound)
            }
        }
    }

    pub fn get_users(
        requester: User,
    ) -> Result<Vec<UserWithoutPassword>, CustomError> {
        if !Self::may_onboard_user(requester) {
            return Err(CustomError::Forbidden);
        }

        Ok(User::find_all_users(&mut get_connection()))
    }

    pub fn delete_invite(invite_id: String) -> Result<(), CustomError> {
        let invite = Invite::find_invite(invite_id)?;
        match invite {
            Some(invite) => {
                Invite::delete_invite(invite.id)?;
                Ok(())
            }
            None => Err(CustomError::NotFound),
        }
    }
}
