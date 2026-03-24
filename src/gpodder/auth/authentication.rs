use crate::app_state::AppState;
use crate::auth_middleware::AuthFilter;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use common_infrastructure::config::EnvironmentService;
use podfetch_domain::session::Session;
use podfetch_web::auth::require_equal_user;
use podfetch_web::gpodder::{GpodderControllerError, ensure_session_user};
use sha256::digest;

fn map_gpodder_error(error: GpodderControllerError<CustomError>) -> CustomError {
    match error {
        GpodderControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        GpodderControllerError::BadRequest(message) => {
            CustomErrorInner::BadRequest(message, Warning).into()
        }
        GpodderControllerError::Service(error) => error,
    }
}

#[utoipa::path(
post,
path="/api/2/auth/{username}/login.json",
responses(
(status = 200, description = "Logs in the user and returns a session cookie.")),
tag="gpodder"
)]
pub async fn login(
    State(state): State<AppState>,
    Path(username): Path<String>,
    jar: CookieJar,
    req: axum::extract::Request,
) -> Result<(CookieJar, StatusCode), CustomError> {
    // If cookie is already set, return it
    if let Some(cookie) = jar.get("sessionid") {
        let session = cookie.value();
        if let Ok(Some(unwrapped_session)) = state.session_service.find_by_session_id(session) {
            let user_cookie = create_session_cookie(unwrapped_session);
            return Ok((user_cookie, StatusCode::OK));
        }
    }

    match state.environment.reverse_proxy {
        true => handle_proxy_auth(
            req,
            &username,
            state.user_auth_service.as_ref(),
            state.environment.as_ref(),
            state.session_service.as_ref(),
        ),
        false => handle_gpodder_basic_auth(
            req,
            &username,
            state.user_auth_service.as_ref(),
            state.environment.as_ref(),
            state.session_service.as_ref(),
        ),
    }
}

fn handle_proxy_auth(
    rq: axum::extract::Request,
    username: &str,
    user_auth_service: &crate::application::services::user_auth::service::UserAuthService,
    environment: &EnvironmentService,
    session_service: &crate::application::services::session::service::SessionService,
) -> Result<(CookieJar, StatusCode), CustomError> {
    let config = environment.reverse_proxy_config.clone().unwrap();
    let opt_authorization = rq.headers().get(config.header_name);
    match opt_authorization {
        Some(auth) => {
            let auth_val = auth.to_str().unwrap();

            ensure_session_user::<CustomError>(auth_val, username).map_err(map_gpodder_error)?;

            match user_auth_service.find_by_username(auth_val) {
                Ok(user) => {
                    let session = session_service.create_session(user.username)?;
                    let user_cookie = create_session_cookie(session);
                    Ok((user_cookie, StatusCode::OK))
                }
                Err(e) => {
                    if config.auto_sign_up {
                        user_auth_service
                            .create_user(username.to_string(), "user".to_string(), None, false)
                            .expect("Error inserting user on auto registering");
                        handle_proxy_auth(
                            rq,
                            username,
                            user_auth_service,
                            environment,
                            session_service,
                        )
                    } else {
                        log::error!("Error finding user by username: {e}");
                        Err(CustomErrorInner::Forbidden(Warning).into())
                    }
                }
            }
        }
        None => Err(CustomErrorInner::Forbidden(Warning).into()),
    }
}

fn handle_gpodder_basic_auth(
    rq: axum::extract::Request,
    username: &str,
    user_auth_service: &crate::application::services::user_auth::service::UserAuthService,
    environment: &EnvironmentService,
    session_service: &crate::application::services::session::service::SessionService,
) -> Result<(CookieJar, StatusCode), CustomError> {
    let opt_authorization = rq.headers().get("Authorization");

    if opt_authorization.is_none() {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    let authorization = opt_authorization.unwrap().to_str().unwrap();

    let (username_basic, password) = AuthFilter::basic_auth_login(authorization)?;
    require_equal_user::<CustomError>(&username_basic, username)
        .map_err(|_| CustomError::from(CustomErrorInner::Forbidden(Warning)))?;

    if let Some(admin_username) = &environment.username
        && admin_username == username
    {
        return Err(CustomErrorInner::Conflict(
            "The user you are trying to login is equal to the admin user. Please\
                 use another user to login."
                .to_string(),
            Warning,
        )
        .into());
    }

    let user = user_auth_service.find_by_username(username)?;
    match user.password {
        Some(p) => {
            if p == digest(password) {
                let session = session_service.create_session(user.username)?;
                let user_cookie = create_session_cookie(session);
                Ok((user_cookie, StatusCode::OK))
            } else {
                Err(CustomErrorInner::Forbidden(Warning).into())
            }
        }
        None => Err(CustomErrorInner::Forbidden(Warning).into()),
    }
}

fn create_session_cookie(session: Session) -> CookieJar {
    CookieJar::new().add(
        Cookie::build(("sessionid", session.session_id))
            .http_only(true)
            .secure(false)
            .same_site(SameSite::Strict)
            .path("/api"),
    )
}

#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::gpodder::auth::authentication::create_session_cookie;
    use podfetch_domain::session::Session;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_create_session_cookie() {
        let session = Session::new("test".to_string());
        let cookie = create_session_cookie(session.clone());

        let cookie = cookie.get("sessionid").unwrap();

        assert_eq!(cookie.name(), "sessionid");
        assert_eq!(cookie.value(), session.session_id);
    }

    use crate::commands::startup::tests::handle_test_startup;
    use crate::gpodder::auth::test_support::tests::create_auth_gpodder;
    use crate::test_utils::test_builder::device_test_builder::tests::DevicePostTestDataBuilder;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use podfetch_web::device::DeviceResponse;

    fn app_state() -> AppState {
        AppState::new()
    }

    #[tokio::test]
    #[serial]
    async fn test_login() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .expect("TODO: panic message");

        create_auth_gpodder(&mut server, &user).await;

        let response = server
            .test_server
            .get(&format!("/api/2/devices/{}", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.json::<Vec<DeviceResponse>>().len(), 0);

        // create device
        let device_post = DevicePostTestDataBuilder::new().build();
        let created_response = server
            .test_server
            .post(&format!(
                "/api/2/devices/{}/{}",
                user.username, device_post.caption
            ))
            .json(&device_post)
            .await;
        assert_eq!(created_response.status_code(), 200);

        // get devices
        let response = server
            .test_server
            .get(&format!("/api/2/devices/{}", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.json::<Vec<DeviceResponse>>().len(), 1);
    }
}


