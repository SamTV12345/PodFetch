use actix_web::{HttpRequest, HttpResponse, post,get, Responder, web};
use crate::DbPool;
use crate::models::invite::Invite;
use crate::models::user::User;
use crate::service::user_management_service::UserManagementService;

#[derive(Deserialize)]
pub struct UserOnboardingModel{
    invite_id: String,
    username: String,
    password: String
}

#[derive(Deserialize)]
pub struct InvitePostModel{
    role: String
}


#[post("/users")]
pub async fn onboard_user(user_onboarding: web::Json<UserOnboardingModel>, mut conn: web::Data<DbPool>)->impl Responder{
    let user_to_onboard = user_onboarding.into_inner();

    UserManagementService::onboard_user(user_to_onboard.username, user_to_onboard.password,
                                        user_to_onboard.invite_id, &mut *conn.get().unwrap());

    HttpResponse::Ok()
}

#[post("/invites")]
pub async fn create_invite(req: HttpRequest, invite: web::Json<InvitePostModel>)->impl Responder{
    let invite = invite.into_inner();
    HttpResponse::Ok()
}

#[get("/")]
pub async fn test123() ->impl Responder{
    HttpResponse::Ok()
}