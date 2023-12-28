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

#[post("/auth/{username}/login.json")]
pub async fn login(
    username: web::Path<String>,
    rq: HttpRequest,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    let env = ENVIRONMENT_SERVICE.get().unwrap();
    if let Some(cookie) = rq.clone().cookie("sessionid") {
        let session = cookie.value();
        let opt_session =
            Session::find_by_session_id(session, conn.get().map_err(map_r2d2_error)?.deref_mut());
        if let Ok(unwrapped_session) = opt_session {
            let user_cookie = create_session_cookie(unwrapped_session);
            return Ok(HttpResponse::Ok().cookie(user_cookie).finish());
        }
    }

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
    if unwrapped_username == env.username && password == env.password {
        Ok(HttpResponse::Ok().finish())
    } else {
        let user = User::find_by_username(
            &unwrapped_username,
            conn.get().map_err(map_r2d2_error)?.deref_mut(),
        )?;
        if user.clone().password.unwrap() == digest(password) {
            let session = Session::new(user.username);
            Session::insert_session(&session, conn.get().map_err(map_r2d2_error)?.deref_mut())
                .expect("Error inserting session");
            let user_cookie = create_session_cookie(session);
            Ok(HttpResponse::Ok().cookie(user_cookie).finish())
        } else {
            Err(CustomError::Forbidden)
        }
    }
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
