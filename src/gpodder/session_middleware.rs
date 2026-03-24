use crate::app_state::AppState;
use axum_extra::extract::cookie::CookieJar;
use std::convert::Infallible;

use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;

pub async fn handle_cookie_session(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, CustomError> {
    let jar = CookieJar::from_headers(req.headers());
    let cookie = jar.get("sessionid");
    if cookie.is_none() {
        let inner_error = CustomErrorInner::Forbidden(Warning);
        let error: CustomError = inner_error.into();
        return Err(error);
    }
    let binding = cookie.unwrap();
    let extracted_cookie = binding.value();

    let session = state
        .session_service
        .find_by_session_id(extracted_cookie)
        .map_err(<CustomError as Into<Infallible>>::into)?;
    if session.is_none() {
        let inner_error = CustomErrorInner::Forbidden(Warning);
        let error: CustomError = inner_error.into();
        return Err(error);
    }

    req.extensions_mut().insert(session.unwrap());

    Ok(next.run(req).await)
}

