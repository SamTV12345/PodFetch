use std::str::FromStr;
use std::sync::MutexGuard;

use crate::constants::inner_constants::Role;
use crate::models::invite::Invite;
use crate::models::user::{User, UserWithoutPassword};
use crate::service::environment_service::EnvironmentService;
use crate::utils::error::CustomError;
use crate::DBType as DbConnection;
use sha256::digest;

pub struct UserManagementService {}

impl UserManagementService {
    pub fn may_onboard_user(user: User) -> bool {
        Role::from_str(&user.role).unwrap() == Role::Admin
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
        conn: &mut DbConnection,
    ) -> Result<User, CustomError> {
        // Check if the invite is valid
        match Invite::find_invite(invite_id.clone(), conn) {
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
                            Role::from_str(&invite.role).unwrap(),
                            Some(digest(password.clone())),
                            chrono::Utc::now().naive_utc(),
                            invite.explicit_consent,
                        );

                        // This is safe as only when basic auth is enabled, the password is set
                        if Self::is_valid_password(password.clone()) {
                            match User::insert_user(&mut actual_user, conn) {
                                Ok(user) => {
                                    Invite::invalidate_invite(invite_id.clone(), conn)
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
        conn: &mut DbConnection,
        user: User,
    ) -> Result<Invite, CustomError> {
        if Self::may_onboard_user(user) {
            let invite = Invite::insert_invite(&role, explicit_consent_i, conn).expect(
                "Error \
            inserting invite",
            );
            return Ok(invite);
        }
        Err(CustomError::Forbidden)
    }

    pub fn delete_user(user: User, conn: &mut DbConnection) -> Result<usize, CustomError> {
        User::delete_user(&user, conn)
    }

    pub fn update_user(
        user_to_update: User,
        conn: &mut DbConnection,
    ) -> Result<UserWithoutPassword, CustomError> {
        match User::update_role(&user_to_update, conn) {
            Ok(user) => {
                match User::update_explicit_consent(&user_to_update, conn) {
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
        environment_service: MutexGuard<EnvironmentService>,
        conn: &mut DbConnection,
    ) -> Result<String, CustomError> {
        let invite = Invite::find_invite(invite_id, conn)?;
        match invite {
            Some(invite) => Ok(environment_service.clone().server_url + "ui/invite/" + &invite.id),
            None => Err(CustomError::NotFound),
        }
    }

    pub fn get_invite(invite_id: String, conn: &mut DbConnection) -> Result<Invite, CustomError> {
        let invite = Invite::find_invite(invite_id, conn)?;

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

    pub fn get_invites(conn: &mut DbConnection) -> Result<Vec<Invite>, CustomError> {
        match Invite::find_all_invites(conn) {
            Ok(invites) => Ok(invites),
            Err(e) => {
                log::error!("The following error occured when finding an invite {}", e);
                Err(CustomError::NotFound)
            }
        }
    }

    pub fn get_users(
        requester: User,
        conn: &mut DbConnection,
    ) -> Result<Vec<UserWithoutPassword>, CustomError> {
        if !Self::may_onboard_user(requester) {
            return Err(CustomError::Forbidden);
        }

        Ok(User::find_all_users(conn))
    }

    pub fn delete_invite(invite_id: String, conn: &mut DbConnection) -> Result<(), CustomError> {
        let invite = Invite::find_invite(invite_id, conn)?;
        match invite {
            Some(invite) => {
                Invite::delete_invite(invite.id, conn)?;
                Ok(())
            }
            None => Err(CustomError::NotFound),
        }
    }
}
