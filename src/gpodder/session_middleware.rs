use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::models::session::Session;
use axum_extra::extract::cookie::CookieJar;
use std::convert::Infallible;

use crate::utils::error::ErrorSeverity::{Critical, Warning};
use crate::utils::error::{map_db_error, CustomError, CustomErrorInner};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use diesel::ExpressionMethods;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};

pub async fn handle_cookie_session(
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

    use crate::adapters::persistence::dbconfig::schema::sessions::dsl::*;
    let session = sessions
        .filter(session_id.eq(extracted_cookie))
        .first::<Session>(&mut get_connection())
        .optional()
        .map_err(|e| map_db_error(e, Critical))
        .map_err(<CustomError as Into<Infallible>>::into)?;
    if session.is_none() {
        let inner_error = CustomErrorInner::Forbidden(Warning);
        let error: CustomError = inner_error.into();
        return Err(error);
    }

    req.extensions_mut().insert(session.unwrap());

    Ok(next.run(req).await)
}
