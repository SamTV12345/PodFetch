use log::error;
use s3::error::S3Error;
use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub struct BacktraceError {
    pub inner: CustomErrorInner,
    pub backtrace: Box<Backtrace>,
}

impl Display for BacktraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Initial error: {:}", self.inner)?;
        writeln!(f, "Error context:")?;
        writeln!(f, "{:}", self.backtrace)
    }
}

impl From<CustomErrorInner> for BacktraceError {
    fn from(inner: CustomErrorInner) -> Self {
        let backtrace = Box::new(Backtrace::force_capture());
        Self { inner, backtrace }
    }
}

pub trait ResultExt: Sized {
    type T;
    fn unwrap_or_backtrace(self) -> Self::T {
        self.expect_or_backtrace("ResultExt::unwrap_or_backtrace found Err")
    }
    fn expect_or_backtrace(self, msg: &str) -> Self::T;
}

impl<T> ResultExt for Result<T, BacktraceError> {
    type T = T;
    fn expect_or_backtrace(self, msg: &str) -> T {
        match self {
            Ok(ok) => ok,
            Err(bterr) => {
                eprintln!("{}", msg);
                eprintln!();
                eprintln!("{:}", bterr);
                panic!("{}", msg);
            }
        }
    }
}

pub struct DynBacktraceError {
    inner: Box<dyn Error + Send + Sync + 'static>,
    backtrace: Box<Backtrace>,
}

impl<E: Error + Send + Sync + 'static> From<E> for DynBacktraceError {
    fn from(inner: E) -> Self {
        let backtrace = Box::new(Backtrace::force_capture());
        Self {
            inner: Box::new(inner),
            backtrace,
        }
    }
}

impl Deref for DynBacktraceError {
    type Target = dyn Error + Send + Sync + 'static;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl DerefMut for DynBacktraceError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

impl Display for DynBacktraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Initial error: {:}", self.inner)?;
        writeln!(f, "Error context:")?;
        writeln!(f, "{:}", self.backtrace)
    }
}

impl Debug for DynBacktraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl ResultExt for Result<(), DynBacktraceError> {
    type T = ();
    fn expect_or_backtrace(self, msg: &str) {
        match self {
            Ok(()) => (),
            Err(bterr) => {
                eprintln!("{}", msg);
                eprintln!();
                eprintln!("{:}", bterr);
                panic!("{}", msg);
            }
        }
    }
}

impl Debug for BacktraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Error for BacktraceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.inner)
    }
}

pub(crate) type CustomError = BacktraceError;

impl IntoResponse  for CustomError {
    fn into_response(self) -> Response {
        let status = match &self.inner {
            CustomErrorInner::NotFound => StatusCode::NOT_FOUND,
            CustomErrorInner::Forbidden => StatusCode::FORBIDDEN,
            CustomErrorInner::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            CustomErrorInner::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomErrorInner::Conflict(e) => StatusCode::CONFLICT,
            CustomErrorInner::BadRequest(e) => StatusCode::BAD_REQUEST,
            CustomErrorInner::UnAuthorized(e) => StatusCode::UNAUTHORIZED,
        };

        (status, self.error_response().message).into_response()
    }
}


impl CustomError {
    fn error_response(&self) -> ErrorResponse {
        let status_code = match &self.inner {
            CustomErrorInner::NotFound => {404}
            CustomErrorInner::Forbidden => {403}
            CustomErrorInner::UnAuthorized(_) => {401}
            CustomErrorInner::BadRequest(_) => {404}
            CustomErrorInner::Conflict(_) => {409}
            CustomErrorInner::Unknown => {500}
            CustomErrorInner::DatabaseError(_) => {500}
        };
        ErrorResponse {
            code: status_code as u16,
            message: self.inner.to_string(),
            error: self.inner.name(),
        }
    }
}

impl Drop for CustomError {
    fn drop(&mut self) {
        error!(
            "Error {}: {} with error",
            self.inner.to_string(),
            self.backtrace
        );
    }
}

#[derive(Error, Debug)]
pub enum CustomErrorInner {
    #[error("Requested file was not found")]
    NotFound,
    #[error("You are forbidden to access requested file.")]
    Forbidden,
    #[error("You are not authorized to access this resource. {0}")]
    UnAuthorized(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("The following error occurred: {0}")]
    Conflict(String),
    #[error("Unknown Internal Error")]
    Unknown,
    #[error("Unknown Internal Error")]
    DatabaseError(diesel::result::Error),
}

impl CustomErrorInner {
    pub fn name(&self) -> String {
        match self {
            Self::NotFound => "NotFound".to_string(),
            Self::Forbidden => "Forbidden".to_string(),
            Self::Unknown => "Unknown".to_string(),
            Self::DatabaseError(_) => "DatabaseError".to_string(),
            Self::Conflict(e) => e.to_string(),
            Self::BadRequest(e) => e.to_string(),
            Self::UnAuthorized(_) => "UnAuthorized".to_string(),
        }
    }
}

pub fn map_io_error(e: std::io::Error, path: Option<String>) -> CustomError {
    error!(
        "IO error: {} for path {}",
        e,
        path.unwrap_or("".to_string())
    );
    match e.kind() {
        std::io::ErrorKind::NotFound => CustomError::from(CustomErrorInner::NotFound),
        std::io::ErrorKind::PermissionDenied => CustomError::from(CustomErrorInner::Forbidden),
        _ => CustomError::from(CustomErrorInner::Unknown),
    }
}

pub fn map_s3_error(error: S3Error) -> CustomError {
    log::info!("S3 error: {}", error);
    CustomErrorInner::Unknown.into()
}

pub fn map_io_extra_error(e: fs_extra::error::Error, path: Option<String>) -> CustomError {
    error!(
        "IO extra error: {} for path {}",
        e,
        path.unwrap_or("".to_string())
    );
    CustomError::from(CustomErrorInner::Unknown)
}

pub fn map_db_error(e: diesel::result::Error) -> CustomError {
    error!("Database error: {}", e);
    match e {
        diesel::result::Error::InvalidCString(_) => CustomError::from(CustomErrorInner::NotFound),
        diesel::result::Error::DatabaseError(_, _) => {
            CustomError::from(CustomErrorInner::DatabaseError(e))
        }
        _ => CustomError::from(CustomErrorInner::Unknown),
    }
}

pub fn map_r2d2_error(e: r2d2::Error) -> CustomError {
    error!("R2D2 error: {}", e);
    CustomError::from(CustomErrorInner::Unknown)
}

pub fn map_reqwest_error(e: reqwest::Error) -> CustomError {
    error!("Error during reqwest: {}", e);

    CustomErrorInner::BadRequest("Error requesting resource from server".to_string()).into()
}

#[derive(Serialize)]
struct ErrorResponse {
    pub code: u16,
    pub error: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use crate::utils::error::{map_db_error, map_io_error, CustomErrorInner};

    use diesel::result::Error;
    use std::io::ErrorKind;

    #[test]
    fn test_map_io_error() {
        let io_error = std::io::Error::new(ErrorKind::NotFound, "File not found");
        let custom_error = map_io_error(io_error, None);
        assert!(custom_error
            .to_string()
            .contains("Requested file was not found"));
    }

    #[test]
    fn test_map_db_error() {
        let db_error = Error::NotFound;
        let custom_error = map_db_error(db_error);
        assert!(custom_error.to_string().starts_with("Initial error"));
    }

    #[test]
    fn test_custom_error() {
        let custom_error = CustomErrorInner::NotFound;
        assert_eq!(custom_error.to_string(), "Requested file was not found");
    }

    #[test]
    fn test_custom_conflict_message() {
        let custom_error = CustomErrorInner::Conflict("An error occured".to_string());
        assert_eq!(
            custom_error.to_string(),
            "The following error occurred: An error occured"
        );
    }
}
