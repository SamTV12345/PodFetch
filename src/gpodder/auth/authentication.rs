use crate::auth_middleware::AuthFilter;
use crate::models::session::Session;
use crate::models::user::User;

use crate::utils::error::{map_r2d2_error, CustomError};
use crate::DbPool;
use actix_web::post;
use actix_web::web::Data;
use actix_web::{web, HttpRequest, HttpResponse};
use awc::cookie::{Cookie, SameSite};
use sha256::digest;
use std::ops::DerefMut;

use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::service::environment_service::EnvironmentService;

#[post("/auth/{username}/login.json")]
pub async fn login(
    username: web::Path<String>,
    rq: HttpRequest,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    let env = ENVIRONMENT_SERVICE.get().unwrap();

    // If cookie is already set, return it
    if let Some(cookie) = rq.clone().cookie("sessionid") {
        let session = cookie.value();
        let opt_session =
            Session::find_by_session_id(session, conn.get().map_err(map_r2d2_error)?.deref_mut());
        if let Ok(unwrapped_session) = opt_session {
            let user_cookie = create_session_cookie(unwrapped_session);
            return Ok(HttpResponse::Ok().cookie(user_cookie).finish());
        }
    }

    match env.reverse_proxy {
        true => handle_proxy_auth(rq, username, conn, env),
        false => handle_gpodder_basic_auth(rq, username, conn, env),
    }
}

fn handle_proxy_auth(
    rq: HttpRequest,
    username: web::Path<String>,
    conn: Data<DbPool>,
    env: &EnvironmentService,
) -> Result<HttpResponse, CustomError> {
    let config = env.reverse_proxy_config.clone().unwrap();
    let opt_authorization = rq.headers().get(config.header_name);
    return match opt_authorization {
        Some(auth) => {
            let auth_val = auth.to_str().unwrap();

            // Block if auth and user is different
            if auth_val != username.into_inner() {
                return Err(CustomError::Forbidden);
            }

            return match User::find_by_username(
                auth_val,
                conn.get().map_err(map_r2d2_error)?.deref_mut(),
            ) {
                Ok(user) => {
                    let session = Session::new(user.username);
                    Session::insert_session(
                        &session,
                        conn.get().map_err(map_r2d2_error)?.deref_mut(),
                    )?;
                    let user_cookie = create_session_cookie(session);
                    Ok(HttpResponse::Ok().cookie(user_cookie).finish())
                }
                Err(e) => {
                    log::error!("Error finding user by username: {}", e);
                    Err(CustomError::Forbidden)
                }
            };
        }
        None => Err(CustomError::Forbidden),
    };
}

fn handle_gpodder_basic_auth(
    rq: HttpRequest,
    username: web::Path<String>,
    conn: Data<DbPool>,
    env: &EnvironmentService,
) -> Result<HttpResponse, CustomError> {
    let opt_authorization = rq.headers().get("Authorization");

    if opt_authorization.is_none() {
        return Err(CustomError::Forbidden);
    }

    let authorization = opt_authorization.unwrap().to_str().unwrap();

    let unwrapped_username = username.into_inner();
    let (username_basic, password) = AuthFilter::basic_auth_login(authorization.to_string());
    if username_basic != unwrapped_username {
        return Err(CustomError::Forbidden);
    }

    if let Some(admin_username) = &env.username {
        if admin_username == &unwrapped_username {
            return Err(CustomError::Conflict(
                "The user you are trying to login is equal to the admin user. Please\
                 use another user to login."
                    .to_string(),
            ));
        }
    }

    let user = User::find_by_username(
        &unwrapped_username,
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;
    return match user.password {
        Some(p) => {
            if p == digest(password) {
                let session = Session::new(user.username);
                Session::insert_session(&session, conn.get().map_err(map_r2d2_error)?.deref_mut())
                    .expect("Error inserting session");
                let user_cookie = create_session_cookie(session);
                Ok(HttpResponse::Ok().cookie(user_cookie).finish())
            } else {
                Err(CustomError::Forbidden)
            }
        }
        None => Err(CustomError::Forbidden),
    };
}

fn create_session_cookie(session: Session) -> Cookie<'static> {
    let user_cookie = Cookie::build("sessionid", session.session_id)
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .path("/api")
        .finish();
    user_cookie
}

#[cfg(test)]
mod tests {
    use crate::gpodder::auth::authentication::create_session_cookie;
    use crate::models::session::Session;

    #[test]
    fn test_create_session_cookie() {
        let session = Session::new("test".to_string());
        let cookie = create_session_cookie(session.clone());

        assert_eq!(cookie.name(), "sessionid");
        assert_eq!(cookie.value(), session.session_id);
    }
}
