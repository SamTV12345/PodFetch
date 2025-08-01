use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::constants::inner_constants::{Role, ENVIRONMENT_SERVICE};
use crate::models::invite::Invite;
use crate::models::user::{User, UserWithoutPassword};
use crate::utils::error::ErrorSeverity::{Critical, Debug, Error, Info, Warning};
use crate::utils::error::{CustomError, CustomErrorInner, ErrorSeverity};
use sha256::digest;

pub struct UserManagementService {}

impl UserManagementService {
    pub fn may_onboard_user(user: User) -> bool {
        Role::try_from(user.role).unwrap() == Role::Admin
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
        match Invite::find_invite(invite_id.clone()) {
            Ok(invite) => {
                match invite {
                    Some(invite) => {
                        if invite.accepted_at.is_some() {
                            return Err(CustomErrorInner::Conflict(
                                "Invite already accepted".to_string(),
                                Warning,
                            )
                            .into());
                        }

                        let mut actual_user = User::new(
                            1,
                            username,
                            Role::try_from(invite.role)?,
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
                                        "The following error occured when inserting a user {e}"
                                    );
                                    Err(CustomErrorInner::Unknown(Critical).into())
                                }
                            }
                        } else {
                            Err(CustomErrorInner::Conflict(
                                "Password is not valid".to_string(),
                                Warning,
                            )
                            .into())
                        }
                    }
                    None => Err(CustomErrorInner::NotFound(Debug).into()),
                }
            }
            Err(e) => {
                log::error!("The following error occured when finding an invite {e}");
                Err(CustomErrorInner::NotFound(Debug).into())
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
        Err(CustomErrorInner::Forbidden(Warning).into())
    }

    pub fn delete_user(user: User) -> Result<usize, CustomError> {
        User::delete_user(&user)
    }

    pub fn update_user(user_to_update: User) -> Result<UserWithoutPassword, CustomError> {
        match User::update_role(&user_to_update) {
            Ok(user) => {
                match User::update_explicit_consent(&user_to_update) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("The following error occured when updating a user {e}");
                        return Err(CustomErrorInner::Unknown(Warning).into());
                    }
                }
                Ok(user)
            }
            Err(e) => {
                log::error!("The following error occured when updating a user {e}");
                Err(CustomErrorInner::Unknown(Error).into())
            }
        }
    }

    pub fn get_invite_link(invite_id: String) -> Result<String, CustomError> {
        let invite = Invite::find_invite(invite_id)?;
        match invite {
            Some(invite) => Ok(format!(
                "{}{}{}",
                ENVIRONMENT_SERVICE.server_url, "ui/invite/", &invite.id
            )),
            None => Err(CustomErrorInner::NotFound(Error).into()),
        }
    }

    pub fn get_invite(invite_id: String) -> Result<Invite, CustomError> {
        let invite = Invite::find_invite(invite_id)?;

        match invite {
            Some(invite) => {
                if invite.accepted_at.is_some() {
                    return Err(CustomErrorInner::Conflict(
                        "Invite already accepted".to_string(),
                        ErrorSeverity::Info,
                    )
                    .into());
                }
                Ok(invite)
            }
            None => Err(CustomErrorInner::NotFound(Info).into()),
        }
    }

    pub fn get_invites() -> Result<Vec<Invite>, CustomError> {
        match Invite::find_all_invites() {
            Ok(invites) => Ok(invites),
            Err(e) => {
                log::error!("The following error occured when finding an invite {e}");
                Err(CustomErrorInner::NotFound(Info).into())
            }
        }
    }

    pub fn get_users(requester: User) -> Result<Vec<UserWithoutPassword>, CustomError> {
        if !Self::may_onboard_user(requester) {
            return Err(CustomErrorInner::Forbidden(Warning).into());
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
            None => Err(CustomErrorInner::NotFound(Debug).into()),
        }
    }
}
