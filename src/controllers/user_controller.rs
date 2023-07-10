use std::sync::Mutex;
use actix_web::{HttpRequest, HttpResponse, post, get, put, Responder, web, delete};
use actix_web::web::Data;
use crate::constants::constants::{Role, USERNAME};
use crate::DbPool;
use crate::models::user::User;
use crate::mutex::LockResultExt;
use crate::service::environment_service::EnvironmentService;
use crate::service::user_management_service::UserManagementService;
use utoipa::ToSchema;
use crate::utils::error::CustomError;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserOnboardingModel{
    invite_id: String,
    username: String,
    password: String
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitePostModel{
    role: Role,
    explicit_consent: bool
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRoleUpdateModel{
    role: Role,
    explicit_consent: bool
}

#[utoipa::path(
context_path="/api/v1",
request_body = UserOnboardingModel,
responses(
(status = 200, description = "Creates a user (admin)")),
tag="info"
)]
#[post("/users/")]
pub async fn onboard_user(user_onboarding: web::Json<UserOnboardingModel>, conn: Data<DbPool>)->
                                                                                              Result<HttpResponse, CustomError>{
    let user_to_onboard = user_onboarding.into_inner();

    let res = UserManagementService::onboard_user(user_to_onboard.username, user_to_onboard
        .password,
                                                  user_to_onboard.invite_id, &mut *conn.get()
            .unwrap())?;

        Ok(HttpResponse::Ok().json(User::map_to_dto(res)))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all users", body= Vec<UserOnboardingModel>)),
tag="info"
)]
#[get("")]
pub async fn get_users(conn: Data<DbPool>, requester: Option<web::ReqData<User>>)->
                                                                                 Result<HttpResponse, CustomError>{

    let res = UserManagementService::get_users(requester.unwrap().into_inner(),&mut *conn.get()
        .unwrap())?;

    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets a user by username", body = Option<User>)),
tag="info"
)]
#[get("/users/{username}")]
pub async fn get_user(req: HttpRequest, conn: Data<DbPool>)-> Result<HttpResponse, CustomError>{
    let username = get_user_from_request(req);
    let user = User::find_by_username(&username, &mut *conn.get().unwrap())?;
    Ok(HttpResponse::Ok().json(User::map_to_dto(user)))

}

#[utoipa::path(
context_path="/api/v1",
request_body = UserOnboardingModel,
responses(
(status = 200, description = "Updates the role of a user", body = Option<User>)),
tag="info"
)]
#[put("/{username}/role")]
pub async fn update_role(role: web::Json<UserRoleUpdateModel>, conn: Data<DbPool>, username:
web::Path<String>, requester: Option<web::ReqData<User>>)
    -> Result<HttpResponse, CustomError>{

    if !requester.unwrap().is_admin(){
        return Err(CustomError::Forbidden)
    }
    let mut user_to_update = User::find_by_username(&username, &mut *conn.get().unwrap())?;

    // Update to his/her designated role
    user_to_update.role = role.role.to_string();
    user_to_update.explicit_consent = role.explicit_consent;

    let res = UserManagementService::update_user(user_to_update, &mut *conn.get().unwrap())?;

    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
context_path="/api/v1",
request_body=InvitePostModel,
responses(
(status = 200, description = "Creates an invite", body = Invite,)),
tag="info"
)]
#[post("/invites")]
pub async fn create_invite(invite: web::Json<InvitePostModel>, conn:
Data<DbPool>, requester: Option<web::ReqData<User>>)
    ->impl
Responder{
    let invite = invite.into_inner();

    let created_invite = UserManagementService::create_invite(invite.role, invite
        .explicit_consent,&mut *conn.get()
        .unwrap(), requester.unwrap().into_inner()).expect("Error creating invite");
    HttpResponse::Ok().json(created_invite)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all invites", body = Vec<Invite>)),
tag="info"
)]
#[get("/invites")]
pub async fn get_invites(conn: Data<DbPool>, requester: Option<web::ReqData<User>>)->impl Responder{
    if !requester.unwrap().is_admin(){
        return HttpResponse::Forbidden().body("You are not authorized to perform this action")
    }

    let invites = UserManagementService::get_invites( &mut *conn.get().unwrap());

    if invites.is_err(){
        return HttpResponse::BadRequest().body(invites.err().unwrap().name().clone())
    }

    HttpResponse::Ok().json(invites.unwrap())
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets a specific invite", body = Option<Invite>)),
tag="info"
)]
#[get("/users/invites/{invite_id}")]
pub async fn get_invite(conn: Data<DbPool>, invite_id: web::Path<String>)->
    impl Responder{
    match UserManagementService::get_invite(invite_id.into_inner(), &mut *conn.get().unwrap()){
        Ok(invite) => HttpResponse::Ok().json(invite),
        Err(e) => HttpResponse::BadRequest().body(e.to_string())
    }
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Deletes a user by username")),
tag="info"
)]
#[delete("/{username}")]
pub async fn delete_user(conn:Data<DbPool>, username: web::Path<String>,  requester: Option<web::ReqData<User>>)->impl
Responder{
    if !requester.unwrap().is_admin(){
        return HttpResponse::Forbidden().body("You are not authorized to perform this action")
    }

    let user_to_delete = User::find_by_username(&username, &mut *conn.get().unwrap()).unwrap();
    return match UserManagementService::delete_user(user_to_delete, &mut *conn.get().unwrap())
    {
        Ok(_) => HttpResponse::Ok().into(),
        Err(e) => HttpResponse::BadRequest().body(e.to_string())
    };
}

#[utoipa::path(
context_path="/api/v1",
tag="info",
responses(
(status = 200, description = "Gets an invite by id", body = Option<Invite>)))]
#[get("/invites/{invite_id}/link")]
pub async fn get_invite_link(conn: Data<DbPool>, invite_id: web::Path<String>,
                             environment_service: Data<Mutex<EnvironmentService>>, requester: Option<web::ReqData<User>>)->
    impl Responder{
    if !requester.unwrap().is_admin(){
        return HttpResponse::Forbidden().body("You are not authorized to perform this action")
    }
    let environment_service = environment_service.lock().ignore_poison();


    match UserManagementService::get_invite_link(invite_id.into_inner(),
                                                 environment_service,&mut *conn.get()
        .unwrap()){
        Ok(invite) => HttpResponse::Ok().json(invite),
        Err(e) => HttpResponse::BadRequest().body(e.to_string())
    }
}

#[utoipa::path(
context_path="/api/v1",
tag="info",
responses(
(status = 200, description = "Deletes an invite by id")))]
#[delete("/invites/{invite_id}")]
pub async fn delete_invite(conn:Data<DbPool>, invite_id: web::Path<String>,requester: Option<web::ReqData<User>>)->impl Responder{
    if !requester.unwrap().is_admin(){
        return HttpResponse::Forbidden().body("You are not authorized to perform this action")
    }

    return match UserManagementService::delete_invite(invite_id.into_inner(), &mut *conn.get().unwrap())
    {
        Ok(_) => HttpResponse::Ok().into(),
        Err(e) => HttpResponse::BadRequest().body(e.to_string())
    };
}

fn get_user_from_request(req: HttpRequest)->String{
    req.clone().headers().get(USERNAME).unwrap().to_str().unwrap().to_string()
}