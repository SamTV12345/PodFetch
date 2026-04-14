use crate::app_state::AppState;
use crate::auth::require_equal_user;
use crate::auth_middleware::AuthFilter;
use crate::gpodder::{
    ensure_session_user, map_gpodder_error, require_password_match, require_present_header_value,
};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::session::Session;
use sha256::digest;

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
    log::info!("GPodder login attempt for user '{username}'");

    // If cookie is already set, return it
    if let Some(cookie) = jar.get("sessionid") {
        let session = cookie.value();
        log::debug!("GPodder login: existing sessionid cookie found for user '{username}'");
        if let Ok(Some(unwrapped_session)) = state.session_service.find_by_session_id(session) {
            log::info!("GPodder login: reusing existing session for user '{username}'");
            let user_cookie = create_session_cookie(unwrapped_session);
            return Ok((user_cookie, StatusCode::OK));
        }
        log::debug!("GPodder login: existing session invalid/expired for user '{username}'");
    }

    match state.environment.reverse_proxy {
        true => {
            log::debug!("GPodder login: using reverse proxy auth for user '{username}'");
            handle_proxy_auth(
                req,
                &username,
                state.user_auth_service.as_ref(),
                state.environment.as_ref(),
                state.session_service.as_ref(),
            )
        }
        false => {
            log::debug!("GPodder login: using basic auth for user '{username}'");
            handle_gpodder_basic_auth(
                req,
                &username,
                state.user_auth_service.as_ref(),
                state.environment.as_ref(),
                state.session_service.as_ref(),
            )
        }
    }
}

#[utoipa::path(
post,
path="/api/2/auth/{username}/logout.json",
responses(
(status = 200, description = "Logs out the user and removes the session.")),
tag="gpodder"
)]
pub async fn logout(
    State(state): State<AppState>,
    Path(username): Path<String>,
    jar: CookieJar,
) -> Result<StatusCode, CustomError> {
    // Verify the session belongs to the requested user
    if let Some(cookie) = jar.get("sessionid")
        && let Ok(Some(session)) = state.session_service.find_by_session_id(cookie.value())
    {
        ensure_session_user::<CustomError>(&session.username, &username)
            .map_err(map_gpodder_error)?;
        state.session_service.delete_by_user_id(session.user_id)?;
        return Ok(StatusCode::OK);
    }
    Err(CustomErrorInner::Forbidden(Warning).into())
}

fn handle_proxy_auth(
    rq: axum::extract::Request,
    username: &str,
    user_auth_service: &crate::services::user_auth::service::UserAuthService,
    environment: &EnvironmentService,
    session_service: &crate::services::session::service::SessionService,
) -> Result<(CookieJar, StatusCode), CustomError> {
    let config = environment.reverse_proxy_config.clone().unwrap();
    let header_name = &config.header_name;
    let authorization_header = rq
        .headers()
        .get(header_name.as_str())
        .and_then(|header| header.to_str().ok());

    if authorization_header.is_none() {
        log::warn!(
            "GPodder proxy auth: missing header '{header_name}' for user '{username}'"
        );
    }

    let auth_val = require_present_header_value::<CustomError>(authorization_header)
        .map_err(map_gpodder_error)?;

    if auth_val != username {
        log::warn!(
            "GPodder proxy auth: header value '{auth_val}' does not match URL username '{username}'"
        );
    }
    ensure_session_user::<CustomError>(&auth_val, username).map_err(map_gpodder_error)?;

    match user_auth_service.find_by_username(&auth_val) {
        Ok(user) => {
            log::info!("GPodder proxy auth: session created for user '{username}'");
            let session = session_service.create_session(user.username, user.id)?;
            let user_cookie = create_session_cookie(session);
            Ok((user_cookie, StatusCode::OK))
        }
        Err(e) => {
            if config.auto_sign_up {
                log::info!(
                    "GPodder proxy auth: user '{username}' not found, auto-signing up"
                );
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
                log::error!(
                    "GPodder proxy auth: user '{username}' not found and auto_sign_up is disabled: {e}"
                );
                Err(CustomErrorInner::Forbidden(Warning).into())
            }
        }
    }
}

fn handle_gpodder_basic_auth(
    rq: axum::extract::Request,
    username: &str,
    user_auth_service: &crate::services::user_auth::service::UserAuthService,
    environment: &EnvironmentService,
    session_service: &crate::services::session::service::SessionService,
) -> Result<(CookieJar, StatusCode), CustomError> {
    let authorization_header = rq
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    if authorization_header.is_none() {
        log::warn!("GPodder basic auth: no Authorization header for user '{username}'");
    }

    let authorization = require_present_header_value::<CustomError>(authorization_header)
        .map_err(map_gpodder_error)?;

    let (username_basic, password) = AuthFilter::basic_auth_login(&authorization)?;

    if username_basic != username {
        log::warn!(
            "GPodder basic auth: username mismatch - header='{}', url='{username}'",
            username_basic
        );
    }
    require_equal_user::<CustomError>(&username_basic, username)
        .map_err(|_| CustomError::from(CustomErrorInner::Forbidden(Warning)))?;

    if let Some(admin_username) = &environment.username
        && admin_username == username
    {
        log::error!(
            "GPodder basic auth: user '{username}' is the admin user (configured via USERNAME env var). \
             Admin users cannot log in via GPodder API. Please create a separate user for GPodder/Kodi."
        );
        return Err(CustomErrorInner::Conflict(
            "The user you are trying to login is equal to the admin user. Please\
                 use another user to login."
                .to_string(),
            Warning,
        )
        .into());
    }

    match user_auth_service.find_by_username(username) {
        Ok(user) => {
            let password_hash = digest(&password);
            if user.password.as_deref() != Some(password_hash.as_str()) {
                log::warn!(
                    "GPodder basic auth: password mismatch for user '{username}'"
                );
            }
            require_password_match::<CustomError>(user.password.as_deref(), &password_hash)
                .map_err(map_gpodder_error)?;

            let session = session_service.create_session(user.username, user.id)?;
            log::info!("GPodder basic auth: login successful for user '{username}'");
            let user_cookie = create_session_cookie(session);
            Ok((user_cookie, StatusCode::OK))
        }
        Err(e) => {
            log::error!("GPodder basic auth: user '{username}' not found in database: {e}");
            Err(e)
        }
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

pub fn get_auth_router() -> utoipa_axum::router::OpenApiRouter<AppState> {
    use utoipa_axum::routes;
    utoipa_axum::router::OpenApiRouter::new()
        .routes(routes!(login))
        .routes(routes!(logout))
}

#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::gpodder_api::auth::authentication::create_session_cookie;
    use podfetch_domain::session::Session;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_create_session_cookie() {
        let session = Session::new("test".to_string(), 1);
        let cookie = create_session_cookie(session.clone());

        let cookie = cookie.get("sessionid").unwrap();

        assert_eq!(cookie.name(), "sessionid");
        assert_eq!(cookie.value(), session.session_id);
    }

    use crate::device::DeviceResponse;
    use crate::gpodder_api::auth::test_support::tests::create_auth_gpodder;
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::device_test_builder::tests::DevicePostTestDataBuilder;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;

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

    #[tokio::test]
    #[serial]
    async fn test_logout() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();

        create_auth_gpodder(&mut server, &user).await;

        // Verify we're authenticated
        let response = server
            .test_server
            .get(&format!("/api/2/devices/{}", user.username))
            .await;
        assert_eq!(response.status_code(), 200);

        // Logout
        let response = server
            .test_server
            .post(&format!("/api/2/auth/{}/logout.json", user.username))
            .await;
        assert_eq!(response.status_code(), 200);

        // Clear all auth so we only rely on the (now-invalidated) cookie
        server.test_server.clear_headers();

        // After logout, session is gone — request should fail
        let response = server
            .test_server
            .get(&format!("/api/2/devices/{}", user.username))
            .await;
        assert_eq!(response.status_code(), 403);
    }
}
