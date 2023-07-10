use std::sync::{Mutex};
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::web::Data;
use sha256::digest;
use crate::{DbPool};
use crate::models::user::User;
use actix_web::{post};
use crate::mutex::LockResultExt;
use crate::service::environment_service::EnvironmentService;
use awc::cookie::{Cookie, SameSite};
use crate::auth_middleware::AuthFilter;
use crate::models::session::Session;
use crate::utils::error::CustomError;

#[post("/auth/{username}/login.json")]
pub async fn login(username:web::Path<String>, rq: HttpRequest, conn:Data<DbPool>,
                   env_service: Data<Mutex<EnvironmentService>>)
    -> Result<HttpResponse, CustomError> {
    let env = env_service.lock().ignore_poison();
    let conn = &mut conn.get().unwrap();
    match rq.clone().cookie("sessionid") {
        Some(cookie) => {
            let session = cookie.value();
            let opt_session = Session::find_by_session_id(session, conn);
                if opt_session.is_ok(){
                    let user_cookie = create_session_cookie(opt_session.unwrap());
                    return Ok(HttpResponse::Ok().cookie(user_cookie).finish());
                }
        }
        None=>{}
    }

    let authorization = rq.headers().get("Authorization").unwrap().to_str().unwrap();
    let unwrapped_username = username.into_inner();
    let (username_basic, password) = AuthFilter::basic_auth_login(authorization.to_string());
    if username_basic != unwrapped_username {
        return Err(CustomError::Forbidden)
    }
    if unwrapped_username == env.username && password == env.password {
        return Ok(HttpResponse::Ok().finish());
    } else {
        let user =  User::find_by_username(&unwrapped_username, conn)?;
        if user.clone().password.unwrap()== digest(password) {
                    let session = Session::new(user.username);
                    Session::insert_session(&session, conn).expect("Error inserting session");
                    let user_cookie = create_session_cookie(session);
                    Ok(HttpResponse::Ok().cookie(user_cookie).finish())
        } else {
                    return  Err(CustomError::Forbidden)
         }
        }
}

fn create_session_cookie(session: Session) -> Cookie<'static> {
    let user_cookie = Cookie::build("sessionid", session.session_id)
        .http_only(true)
        .secure(false)
        .same_site
    (SameSite::Strict).path("/api").finish();
    user_cookie
}
