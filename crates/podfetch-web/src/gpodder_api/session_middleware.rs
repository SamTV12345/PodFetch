use crate::app_state::AppState;
use common_infrastructure::error::CustomError;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use crate::gpodder::{
    extract_session_cookie_value, map_gpodder_error, require_active_session, require_session_cookie,
};

pub async fn handle_cookie_session(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, CustomError> {
    let session_cookie = extract_session_cookie_value(req.headers());
    let extracted_cookie = require_session_cookie::<CustomError>(session_cookie).map_err(map_gpodder_error)?;

    let session = state.session_service.find_by_session_id(&extracted_cookie)?;
    let session = require_active_session::<_, CustomError>(session).map_err(map_gpodder_error)?;

    req.extensions_mut().insert(session);

    Ok(next.run(req).await)
}

