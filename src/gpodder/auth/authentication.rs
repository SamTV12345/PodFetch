use crate::auth_middleware::AuthFilter;
use crate::models::session::Session;
use crate::models::user::User;

use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::utils::error::{CustomError, CustomErrorInner};
use axum::extract::Path;
use axum::http::StatusCode;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use sha256::digest;


#[utoipa::path(
post,
path="/auth/{username}/login.json",
responses(
(status = 200, description = "Logs in the user and returns a session cookie.")),
tag="gpodder"
)]
pub async fn login(
    Path(username): Path<String>,
    jar: CookieJar,
    req: axum::extract::Request,
) -> Result<(CookieJar, StatusCode), CustomError> {
    // If cookie is already set, return it
    if let Some(cookie) = jar.get("sessionid") {
        let session = cookie.value();
        let opt_session = Session::find_by_session_id(session);
        if let Ok(unwrapped_session) = opt_session {
            let user_cookie = create_session_cookie(unwrapped_session);
            return Ok((user_cookie, StatusCode::OK));
        }
    }

    match ENVIRONMENT_SERVICE.reverse_proxy {
        true => handle_proxy_auth(req, &username),
        false => handle_gpodder_basic_auth(req, &username),
    }
}

fn handle_proxy_auth(rq: axum::extract::Request, username: &str) -> Result<(CookieJar, StatusCode),
    CustomError> {
    let config = ENVIRONMENT_SERVICE.reverse_proxy_config.clone().unwrap();
    let opt_authorization = rq.headers().get(config.header_name);
    match opt_authorization {
        Some(auth) => {
            let auth_val = auth.to_str().unwrap();

            // Block if auth and user is different
            if auth_val != username {
                log::error!("Error: Username and auth header are different");
                return Err(CustomErrorInner::Forbidden.into());
            }

            match User::find_by_username(auth_val) {
                Ok(user) => {
                    let session = Session::new(user.username);
                    Session::insert_session(&session)?;
                    let user_cookie = create_session_cookie(session);
                    Ok((user_cookie, StatusCode::OK))
                }
                Err(e) => {
                    if config.auto_sign_up {
                        User::insert_user(&mut User {
                            id: 0,
                            username: username.to_string(),
                            role: "user".to_string(),
                            password: None,
                            explicit_consent: false,
                            created_at: chrono::Utc::now().naive_utc(),
                            api_key: None,
                        })
                        .expect("Error inserting user on auto registering");
                        handle_proxy_auth(rq, username)
                    } else {
                        log::error!("Error finding user by username: {}", e);
                        Err(CustomErrorInner::Forbidden.into())
                    }
                }
            }
        }
        None => Err(CustomErrorInner::Forbidden.into()),
    }
}

fn handle_gpodder_basic_auth(
    rq: axum::extract::Request,
    username: &str,
) -> Result<(CookieJar, StatusCode), CustomError> {
    let opt_authorization = rq.headers().get("Authorization");

    if opt_authorization.is_none() {
        return Err(CustomErrorInner::Forbidden.into());
    }

    let authorization = opt_authorization.unwrap().to_str().unwrap();

    let (username_basic, password) = AuthFilter::basic_auth_login(authorization)?;
    if username_basic != username {
        return Err(CustomErrorInner::Forbidden.into());
    }

    if let Some(admin_username) = &ENVIRONMENT_SERVICE.username {
        if admin_username == username {
            return Err(CustomErrorInner::Conflict(
                "The user you are trying to login is equal to the admin user. Please\
                 use another user to login."
                    .to_string(),
            )
            .into());
        }
    }

    let user = User::find_by_username(username)?;
    match user.password {
        Some(p) => {
            if p == digest(password) {
                let session = Session::new(user.username);
                Session::insert_session(&session).expect("Error inserting session");
                let user_cookie = create_session_cookie(session);
                Ok((user_cookie, StatusCode::OK))
            } else {
                Err(CustomErrorInner::Forbidden.into())
            }
        }
        None => Err(CustomErrorInner::Forbidden.into()),
    }
}

fn create_session_cookie(session: Session) -> CookieJar {

    CookieJar::new().add(
    Cookie::build(("sessionid", session.session_id))
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .path("/api"))
}

#[cfg(test)]
mod tests {
    use crate::gpodder::auth::authentication::create_session_cookie;
    use crate::models::session::Session;

    #[test]
    fn test_create_session_cookie() {
        let session = Session::new("test".to_string());
        let cookie = create_session_cookie(session.clone());

        let cookie = cookie.get("sessionid").unwrap();

        assert_eq!(cookie.name(), "sessionid");
        assert_eq!(cookie.value(), session.session_id);
    }
}
