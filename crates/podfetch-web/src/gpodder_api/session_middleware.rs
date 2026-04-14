use crate::app_state::AppState;
use crate::auth_middleware::AuthFilter;
use crate::gpodder::{
    extract_session_cookie_value, map_gpodder_error, require_password_match,
    require_present_header_value,
};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use common_infrastructure::error::CustomError;
use common_infrastructure::error::CustomErrorInner;
use common_infrastructure::error::ErrorSeverity::Warning;
use podfetch_domain::session::Session;
use sha256::digest;

pub async fn handle_cookie_session(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, CustomError> {
    // Try cookie-based session first
    if let Some(cookie_value) = extract_session_cookie_value(req.headers())
        && let Ok(Some(session)) = state.session_service.find_by_session_id(&cookie_value)
    {
        req.extensions_mut().insert(session);
        return Ok(next.run(req).await);
    }

    // Fall back to Basic Auth
    if let Some(auth_header) = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        && auth_header.starts_with("Basic ")
    {
        let session = if state.environment.reverse_proxy {
            let config = state.environment.reverse_proxy_config.clone().unwrap();
            let proxy_header = req
                .headers()
                .get(&config.header_name)
                .and_then(|h| h.to_str().ok());
            let auth_val = require_present_header_value::<CustomError>(proxy_header)
                .map_err(map_gpodder_error)?;
            let user = state
                .user_auth_service
                .find_by_username(&auth_val)
                .map_err(|_| CustomError::from(CustomErrorInner::Forbidden(Warning)))?;
            Session::new(user.username, user.id)
        } else {
            let (username, password) = AuthFilter::basic_auth_login(auth_header)?;
            let user = state
                .user_auth_service
                .find_by_username(&username)
                .map_err(|_| CustomError::from(CustomErrorInner::Forbidden(Warning)))?;
            require_password_match::<CustomError>(user.password.as_deref(), &digest(password))
                .map_err(map_gpodder_error)?;
            Session::new(user.username, user.id)
        };

        req.extensions_mut().insert(session);
        return Ok(next.run(req).await);
    }

    // No valid auth found
    Err(CustomErrorInner::Forbidden(Warning).into())
}
