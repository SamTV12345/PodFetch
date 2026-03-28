use base64::Engine;
use base64::engine::general_purpose;
use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub enum AuthControllerError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("{0}")]
    Service(E),
}

pub fn parse_basic_auth<Err: Display>(
    auth_header: &str,
) -> Result<(String, String), AuthControllerError<Err>> {
    let parts = auth_header.split(' ').collect::<Vec<&str>>();
    if parts.len() != 2 || parts[0] != "Basic" {
        return Err(AuthControllerError::Forbidden);
    }

    let decoded = general_purpose::STANDARD
        .decode(parts[1])
        .map_err(|_| AuthControllerError::Forbidden)?;
    let decoded = String::from_utf8(decoded).map_err(|_| AuthControllerError::Forbidden)?;
    let auth = decoded.split(':').collect::<Vec<&str>>();
    if auth.len() != 2 {
        return Err(AuthControllerError::Forbidden);
    }

    Ok((auth[0].to_string(), auth[1].to_string()))
}

pub fn require_equal_user<Err: Display>(
    expected: &str,
    actual: &str,
) -> Result<(), AuthControllerError<Err>> {
    if expected == actual {
        Ok(())
    } else {
        Err(AuthControllerError::Forbidden)
    }
}
