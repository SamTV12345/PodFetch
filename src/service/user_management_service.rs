use actix_web::http::StatusCode;
use diesel::{SqliteConnection};
use crate::constants::constants::{ROLE_ADMIN, ROLE_UPLOADER};
use crate::exception::exceptions::{PodFetchError, PodFetchErrorTrait};
use crate::models::invite::Invite;
use crate::models::user::User;
use crate::service::environment_service::EnvironmentService;
use sha256::{digest, try_digest};

pub struct UserManagementService{

}

impl UserManagementService {
    pub fn may_add_podcast(user: User)->bool{
        user.role == ROLE_UPLOADER|| user.role == ROLE_ADMIN
    }

    pub fn may_onboard_user(user: User)->bool{
        user.role == ROLE_ADMIN
    }

    pub fn may_delete_user(user: User)->bool{
        user.role == ROLE_ADMIN
    }

    pub fn may_update_role(user: User)->bool{
        user.role == ROLE_ADMIN
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
        return match Invite::find_invite(invite_id, conn) {
            Ok(invite) => {
                match invite {
                    Some(invite) => {
                        let mut actual_user = User::new(1, username, invite.role, Some
                            (digest(password.clone())), chrono::Utc::now().naive_utc());


                        // This is safe as only when basic auth is enabled, the password is set
                        if Self::is_valid_password(password.clone()) {
                            match User::insert_user(&mut actual_user, conn) {
                                Ok(user) => {
                                    Ok(user)
                                }
                                Err(e) => {
                                    log::error!("The following error occured when inserting a user {}",e);
                                    Err(PodFetchError::new("Error inserting User", StatusCode::INTERNAL_SERVER_ERROR))
                                }
                            }
                        } else {
                            Err(PodFetchError::new("Password is not valid", StatusCode::BAD_REQUEST))
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

    pub fn create_invite(role: String){

    }

    pub fn delete_user(user: User, requester: User, conn: &mut SqliteConnection)->Result<(), PodFetchError>{

        if Self::may_delete_user(requester){
            User::delete_user(&user, conn).expect("Error deleting User");
            return Ok(())
        }
        Err(PodFetchError::no_permission_to_delete_user())
    }

    pub fn update_role(user: User, requester: User, conn: &mut SqliteConnection)->Result<(), PodFetchError>{

        if Self::may_update_role(requester){
            User::update_role(&user, conn).expect("Error updating User");
        }
        Err(PodFetchError::no_permission_to_delete_user())
    }


    pub fn get_invite_link(invite_id: String, requester: User, environment_service: EnvironmentService,
                           conn: &mut SqliteConnection)->Result<String, PodFetchError>{
        match Self::may_onboard_user(requester){
            true=>{},
            false=>{return Err(PodFetchError::no_permission_to_onboard_user())}
        }

        match Invite::find_invite(invite_id, conn){
            Ok(invite) => {
                match invite {
                    Some(invite)=>{
                        Ok(environment_service.server_url+"/ui/invite?invite="+&invite.id)
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