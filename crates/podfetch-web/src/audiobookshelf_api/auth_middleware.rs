//! Bearer-token middleware for the audiobookshelf-compatible API.
//!
//! Accepts the token via `Authorization: Bearer <api_key>` or `?token=<api_key>`
//! (the audiobookshelf apps fall back to the query parameter for `<audio>`-tag
//! requests that can't set headers).
//!
//! Successful auth inserts a `User` into the request extensions.

use crate::app_state::AppState;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};

pub async fn audiobookshelf_bearer_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, CustomError> {
    let token = extract_token(&request).ok_or_else(|| {
        CustomError::from(CustomErrorInner::UnAuthorized(
            "Missing bearer token".to_string(),
            Debug,
        ))
    })?;

    let user = state
        .audiobookshelf_login_service
        .require_user_for_token(&token)?;

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

fn extract_token(request: &Request) -> Option<String> {
    if let Some(header) = request.headers().get("Authorization")
        && let Ok(value) = header.to_str()
        && let Some(token) = value.strip_prefix("Bearer ")
    {
        let token = token.trim();
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }

    let query = request.uri().query()?;
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("token=") {
            let decoded = percent_decode(value);
            if !decoded.is_empty() {
                return Some(decoded);
            }
        }
    }
    None
}

fn percent_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hi = chars.next();
            let lo = chars.next();
            if let (Some(hi), Some(lo)) = (hi, lo)
                && let (Some(hi), Some(lo)) = (hi.to_digit(16), lo.to_digit(16))
            {
                out.push(((hi * 16 + lo) as u8) as char);
                continue;
            }
            return String::new();
        } else if c == '+' {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

/// Used by handler functions to read the authenticated user out of the request
/// extensions populated by the middleware above.
pub fn require_authenticated_user(
    request: &Request,
) -> Result<podfetch_domain::user::User, CustomError> {
    request
        .extensions()
        .get::<podfetch_domain::user::User>()
        .cloned()
        .ok_or_else(|| {
            CustomError::from(CustomErrorInner::UnAuthorized(
                "Not authenticated".to_string(),
                Debug,
            ))
        })
}

#[allow(dead_code)]
pub use require_authenticated_user as require_user;

// Extractor wrapper so handlers can declare `AuthenticatedUser` as an extractor.
pub struct AuthenticatedUser(pub podfetch_domain::user::User);

impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = CustomError;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let user = parts
            .extensions
            .get::<podfetch_domain::user::User>()
            .cloned();
        async move {
            user.map(AuthenticatedUser).ok_or_else(|| {
                CustomError::from(CustomErrorInner::UnAuthorized(
                    "Not authenticated".to_string(),
                    Debug,
                ))
            })
        }
    }
}
