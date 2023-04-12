use std::str::FromStr;
use std::sync::MutexGuard;
use actix_web::http::StatusCode;
use diesel::{SqliteConnection};
use diesel::serialize::ToSql;
use crate::exception::exceptions::{PodFetchError, PodFetchErrorTrait};
use crate::models::invite::Invite;
use crate::models::user::{User, UserWithoutPassword};
use crate::service::environment_service::EnvironmentService;
use sha256::{digest};
use crate::constants::constants::Role;
use crate::schema::users::explicit_consent;

pub struct UserManagementService{

}

impl UserManagementService {
    pub fn may_add_podcast(user: User)->bool{
        Role::from_str(&user.role).unwrap() == Role::Uploader|| Role::from_str(&user.role).unwrap() ==
            Role::Admin
    }

    pub fn may_onboard_user(user: User)->bool{
        Role::from_str(&user.role).unwrap() == Role::Admin
    }

    pub fn may_delete_user(user: User)->bool{
        Role::from_str(&user.role).unwrap() == Role::Admin
    }

    pub fn may_update_role(user: User)->bool{
        Role::from_str(&user.role).unwrap() == Role::Admin
    }

    pub fn is_valid_password(password: String) ->bool{
        let mut has_whitespace = false;
        let mut has_upper = false;
        let mut has_lower = false;
        let mut has_digit = false;

        for c in password.chars() {
            has_whitespace |= c.is_whitespace();
            has_lower |= c.is_lowercase();
            has_upper |= c.is_uppercase();
            has_digit |= c.is_digit(10);
        }

        !has_whitespace && has_upper && has_lower && has_digit && password.len() >= 8
    }

    /**
     * Performs the onboarding with a valid
     */
    pub fn onboard_user(username: String, password: String, invite_id: String, conn: &mut
    SqliteConnection)->Result<User,
        PodFetchError>{

            // Check if the invite is valid
        return match Invite::find_invite(invite_id.clone(), conn) {
            Ok(invite) => {
                match invite {
                    Some(invite) => {
                        if invite.accepted_at.is_some() {
                            return Err(PodFetchError::new("Invite already accepted",
                                                          StatusCode::BAD_REQUEST));
                        }


                        let mut actual_user = User::new(1, username, Role::from_str(&invite.role).unwrap(),
                                                        Some
                            (digest(password.clone())), chrono::Utc::now().naive_utc(), invite.explicit_consent);


                        // This is safe as only when basic auth is enabled, the password is set
                        if Self::is_valid_password(password.clone()) {
                            match User::insert_user(&mut actual_user, conn) {
                                Ok(user) => {
                                    Invite::invalidate_invite(invite_id.clone(), conn).expect
                                    ("Error invalidating invite");
                                    Ok(user)
                                }
                                Err(e) => {
                                    log::error!("The following error occured when inserting a user {}",e);
                                    Err(PodFetchError::new("Error inserting User", StatusCode::INTERNAL_SERVER_ERROR))
                                }
                            }
                        } else {
                            Err(PodFetchError::new("Password is not valid", StatusCode::CONFLICT))
                        }
                    }
                    None => {
                        Err(PodFetchError::new("Invite code not found",
                                               StatusCode::NOT_FOUND))
                    }
                }
            }
            Err(e) => {
                log::error!("The following error occured when finding an invite {}",e);
                Err(PodFetchError::new("Invite code not found", StatusCode::NOT_FOUND))
            }
        }
    }

    pub fn create_invite(role: Role, explicit_consent_i: bool, conn: &mut SqliteConnection, user:
    User)
        -> Result<Invite,
        PodFetchError> {
        if Self::may_onboard_user(user){
            let invite = Invite::insert_invite(&role,explicit_consent_i, conn).expect("Error \
            inserting invite");
            return Ok(invite)
        }
        Err(PodFetchError::no_permissions_to_onboard_user())
    }

    pub fn delete_user(user: User, requester: User, conn: &mut SqliteConnection)->Result<(), PodFetchError>{

        if Self::may_delete_user(requester){
            User::delete_user(&user, conn).expect("Error deleting User");
            return Ok(())
        }
        Err(PodFetchError::no_permission_to_delete_user())
    }

    pub fn update_role(user: User, requester: User, conn: &mut SqliteConnection)->Result<UserWithoutPassword,
        PodFetchError>{
        println!("Rolle: {}", requester.role);
        if Self::may_update_role(requester){
            return match User::update_role(&user, conn) {
                Ok(user) => {
                    Ok(user)
                }
                Err(e) => {
                    log::error!("The following error occured when updating a user {}",e);
                    Err(PodFetchError::new("Error updating User",
                                           StatusCode::INTERNAL_SERVER_ERROR))
                }
            }
        }
        Err(PodFetchError::no_permission_to_update_user_role())
    }


    pub fn get_invite_link(invite_id: String, requester: User, environment_service: MutexGuard<EnvironmentService>,
                           conn: &mut SqliteConnection) ->Result<String, PodFetchError>{
        match Self::may_onboard_user(requester){
            true=>{},
            false=>{return Err(PodFetchError::no_permission_to_onboard_user())}
        }

        match Invite::find_invite(invite_id, conn){
            Ok(invite) => {
                match invite {
                    Some(invite)=>{
                        Ok(environment_service.clone().server_url+"ui/invite/"+&invite.id)
                    }
                    None=>{
                        Err(PodFetchError::new("Invite code not found",
                                               StatusCode::NOT_FOUND))
                    }
                }
            }
            Err(e) => {
                log::error!("The following error occured when finding an invite {}",e);
                Err(PodFetchError::new("Invite code not found", StatusCode::NOT_FOUND))
            }
        }
    }


    pub fn get_invite(invite_id: String, conn: &mut SqliteConnection)->Result<Invite, PodFetchError>{
        match Invite::find_invite(invite_id, conn){
            Ok(invite) => {
                match invite {
                    Some(invite)=>{
                        if invite.accepted_at.is_some(){
                            return Err(PodFetchError::new("Invite already accepted",
                                                          StatusCode::BAD_REQUEST));
                        }
                        Ok(invite)
                    }
                    None=>{
                        Err(PodFetchError::new("Invite code not found",
                                               StatusCode::NOT_FOUND))
                    }
                }
            }
            Err(e) => {
                log::error!("The following error occured when finding an invite {}",e);
                Err(PodFetchError::new("Invite code not found", StatusCode::NOT_FOUND))
            }
        }
    }

    pub fn get_invites(requester: User, conn: &mut SqliteConnection)->Result<Vec<Invite>, PodFetchError>{
        match Self::may_onboard_user(requester){
            true=>{},
            false=>{return Err(PodFetchError::no_permission_to_onboard_user())}
        }

        match Invite::find_all_invites(conn){
            Ok(invites) => {
                Ok(invites)
            }
            Err(e) => {
                log::error!("The following error occured when finding an invite {}",e);
                Err(PodFetchError::new("Invite code not found", StatusCode::NOT_FOUND))
            }
        }
    }

    pub fn get_users(requester: User, conn: &mut SqliteConnection)-> Result<Vec<UserWithoutPassword>, PodFetchError> {
        if !Self::may_onboard_user(requester) {
            return Err(PodFetchError::no_permissions_to_onboard_user())
        }

        return Ok(User::find_all_users(conn))
    }

    pub fn delete_invite(invite_id: String, requester: User, conn: &mut SqliteConnection)->Result<(), PodFetchError>{
        if !Self::may_onboard_user(requester) {
            return Err(PodFetchError::no_permissions_to_onboard_user())
        }

        match Invite::find_invite(invite_id, conn){
            Ok(invite) => {
                match invite {
                    Some(invite)=>{
                        Invite::delete_invite(invite.id, conn).expect("Error deleting invite");
                        Ok(())
                    }
                    None=>{
                        Err(PodFetchError::new("Invite code not found",
                                               StatusCode::NOT_FOUND))
                    }
                }
            }
            Err(e) => {
                log::error!("The following error occured when finding an invite {}",e);
                Err(PodFetchError::new("Invite code not found", StatusCode::NOT_FOUND))
            }
        }
    }
}