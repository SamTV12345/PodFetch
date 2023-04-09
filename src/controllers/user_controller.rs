use actix_web::{HttpRequest, HttpResponse, post,get, Responder, web};
use actix_web::web::Data;
use crate::constants::constants::{Role, USERNAME};
use crate::DbPool;
use crate::exception::exceptions::PodFetchErrorTrait;
use crate::models::invite::Invite;
use crate::models::user::User;
use crate::service::user_management_service::UserManagementService;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOnboardingModel{
    invite_id: String,
    username: String,
    password: String
}

#[derive(Deserialize)]
pub struct InvitePostModel{
    role: Role
}


#[post("/users/")]
pub async fn onboard_user(user_onboarding: web::Json<UserOnboardingModel>, mut conn: Data<DbPool>)->impl Responder{
    let user_to_onboard = user_onboarding.into_inner();

    let res = UserManagementService::onboard_user(user_to_onboard.username, user_to_onboard
        .password,
                                        user_to_onboard.invite_id, &mut *conn.get().unwrap());

    return match res {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::BadRequest()
            .body(e.name().clone())
    };
}

#[post("/invites")]
pub async fn create_invite(req: HttpRequest, invite: web::Json<InvitePostModel>, conn: Data<DbPool>)
    ->impl
Responder{
    let invite = invite.into_inner();
    let username  = req.headers().get(USERNAME).unwrap()
        .to_str().unwrap();
    let user = User::find_by_username(username, &mut *conn.get().unwrap()).unwrap();
    UserManagementService::create_invite(invite.role, &mut *conn.get().unwrap(), user).expect
    ("Error creating invite");
    HttpResponse::Ok()
}

#[get("/invites")]
pub async fn get_invites(req: HttpRequest, conn: Data<DbPool>)->impl Responder{
    let username  = req.headers().get(USERNAME).unwrap()
        .to_str().unwrap();
    let user = User::find_by_username(username, &mut *conn.get().unwrap()).unwrap();
    let invites = UserManagementService::get_invites(user, &mut *conn.get().unwrap()).expect
    ("Error getting invites");
    HttpResponse::Ok().json(invites)
}

#[get("/users/invites/{invite_id}")]
pub async fn get_invite(conn: Data<DbPool>, invite_id: web::Path<String>)->
    impl Responder{
    match UserManagementService::get_invite(invite_id.into_inner(), &mut *conn.get().unwrap()){
        Ok(invite) => HttpResponse::Ok().json(invite),
        Err(e) => HttpResponse::BadRequest().body(e.to_string())
    }
}