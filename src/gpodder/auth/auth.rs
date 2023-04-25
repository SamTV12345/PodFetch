use std::io::Error;
use std::sync::{Mutex, MutexGuard};
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::Data;
use sha256::digest;
use crate::{DbPool, extract_basic_auth};
use crate::models::user::User;
use actix_web::{post};
use crate::mutex::LockResultExt;
use crate::service::environment_service::EnvironmentService;
use awc::cookie::{Cookie, SameSite};
use diesel::SqliteConnection;
use crate::models::session::Session;

#[post("/auth/{username}/login.json")]
pub async fn login(username:web::Path<String>, rq: HttpRequest, conn:Data<DbPool>,
                   env_service: Data<Mutex<EnvironmentService>>)
    ->impl
Responder {
    let env = env_service.lock().ignore_poison();

    match rq.clone().cookie("sessionid") {
        Some(cookie) => {
            let session = cookie.value();
            let opt_session = Session::find_by_session_id(session, &mut conn.get().unwrap());
                if opt_session.is_ok(){
                    let user_cookie = create_session_cookie(opt_session.unwrap(), env);
                    return HttpResponse::Ok().cookie(user_cookie).finish();
                }
        }
        None=>{}
    }

    let authorization = rq.headers().get("Authorization").unwrap().to_str().unwrap();
    let unwrapped_username = username.into_inner();
    let (username_basic, password) = basic_auth_login(authorization.to_string());
    if username_basic != unwrapped_username {
        return HttpResponse::Unauthorized().finish();
    }
    if unwrapped_username == env.username && password == env.password {
        return HttpResponse::Ok().finish();
    } else {
        match User::find_by_username(&unwrapped_username, &mut conn.get().unwrap()) {
            Some(user) => {
                if user.clone().password.unwrap()== digest(password) {
                    let session = Session::new(user.username);
                    Session::insert_session(&session, &mut conn.get().unwrap()).expect("Error inserting session");
                    let user_cookie = create_session_cookie(session, env);
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

fn create_session_cookie(session: Session, env: MutexGuard<EnvironmentService>) -> Cookie<'static> {
    let secure = env.server_url.starts_with("https");
    let user_cookie = Cookie::build("sessionid", session.session_id)
        .http_only(true).secure
    (secure).same_site
    (SameSite::Strict).path("/api").finish();
    user_cookie
}

pub fn basic_auth_login(rq: String) -> (String, String) {
    let (u,p) = extract_basic_auth(rq.as_str());

    return (u.to_string(),p.to_string())
}

pub async fn auth_checker(conn: &mut SqliteConnection, session: Option<String>, username: String)
    ->Result<(),
    Error>{
    return match session {
        Some(session) => {
            let session = Session::find_by_session_id(&session, conn).unwrap();
            if session.username != username {
                return Err(Error::new(std::io::ErrorKind::Other, "User and session not matching"))
            }
            Ok(())
        }
        None => {
            Err(Error::new(std::io::ErrorKind::Other, "No session"))
        }
    }
}

pub fn extract_from_http_request(rq: HttpRequest)->Option<String>{
    rq.cookie("sessionid")
        .map(|cookie|cookie.value().to_string())
}