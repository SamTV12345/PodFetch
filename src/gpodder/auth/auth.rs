use std::collections::HashMap;
use std::sync::Mutex;
use actix_web::dev::ServiceRequest;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::Data;
use sha256::digest;
use crate::{DbPool, extract_basic_auth, validator};
use crate::models::user::User;
use actix_web::{post};
use utoipa::openapi::HeaderBuilder;
use uuid::Uuid;
use crate::mutex::LockResultExt;
use crate::service::environment_service::EnvironmentService;
use actix_session::Session;
use awc::cookie::{Cookie, SameSite};

#[post("/auth/{username}/login.json")]
pub async fn login(username:web::Path<String>, rq: HttpRequest, conn:Data<DbPool>,
                   env_service: Data<Mutex<EnvironmentService>>, session:Data<Mutex<HashMap<String,
        String>>>)
    ->impl
Responder {
    let authorization = rq.headers().get("Authorization").unwrap().to_str().unwrap();
    let unwrapped_username = username.into_inner();
    let (username_basic, password) = basic_auth_login(authorization.to_string());
    let env = env_service.lock().ignore_poison();
    if username_basic != unwrapped_username {
        return HttpResponse::Unauthorized().finish();
    }

    if unwrapped_username == env.username && password == env.password {
        return HttpResponse::Ok().finish();
    } else {
        match User::find_by_username(&unwrapped_username, &mut conn.get().unwrap()) {
            Some(user) => {
                if user.clone().password.unwrap()== digest(password) {
                    let token = Uuid::new_v4().to_string();
                    session.lock().ignore_poison().insert(token.clone(), user.username);
                    let user_cookie = Cookie::build("sessionid", token)
                        .http_only(true).secure
                    (false).same_site
                    (SameSite::Strict).path("/api").finish();
                    HttpResponse::Ok().cookie(user_cookie).finish()
                } else {
                    HttpResponse::Unauthorized().finish()
                }
            }
            None => {
                return  HttpResponse::Unauthorized().finish()
            }
        }
    }
}

pub fn basic_auth_login(rq: String) -> (String, String) {
    let (u,p) = extract_basic_auth(rq.as_str());

    return (u.to_string(),p.to_string())
}

