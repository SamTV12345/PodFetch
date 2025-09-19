use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use log::{debug, error, info, warn};
use s3::error::S3Error;
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::convert::{Infallible, Into};
use std::error::Error;
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use thiserror::Error;

pub struct CustomError {
    pub inner: CustomErrorInner,
    pub backtrace: Box<Backtrace>,
    pub error_severity: ErrorSeverity,
}

pub struct ApiError {
    pub status: StatusCode,
    pub value: ApiErrorValue,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorValue {
    pub error_code: String,
    pub arguments: HashMap<String, String>,
}

impl ApiError {
    pub fn updating_admin_not_allowed(username: &str) -> Self {
        let mut args = HashMap::new();
        args.insert("username".into(), username.into());
        ApiError {
            value: ApiErrorValue {
                error_code: "UPDATE_OF_ADMIN_NOT_ALLOWED".into(),
                arguments: args.clone(),
            },
            status: StatusCode::BAD_REQUEST,
        }
    }
}

pub enum ErrorType {
    CustomErrorType(CustomError),
    ApiErrorType(ApiError),
}

impl IntoResponse for ErrorType {
    fn into_response(self) -> Response {
        match self {
            ErrorType::CustomErrorType(ce) => ce.into_response(),
            ErrorType::ApiErrorType(ae) => {
                let body = serde_json::to_string(&ae.value)
                    .unwrap_or_else(|_| "{\"error\":\"Serialization error\"}".to_string());
                (ae.status, body).into_response()
            }
        }
    }
}

impl From<CustomError> for ErrorType {
    fn from(value: CustomError) -> Self {
        ErrorType::CustomErrorType(value)
    }
}

impl From<CustomErrorInner> for ErrorType {
    fn from(value: CustomErrorInner) -> Self {
        ErrorType::CustomErrorType(value.into())
    }
}

impl From<ApiError> for ErrorType {
    fn from(value: ApiError) -> Self {
        ErrorType::ApiErrorType(value)
    }
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Initial error: {:}", self.inner)?;
        writeln!(f, "Error context:")?;
        writeln!(f, "{:}", self.backtrace)
    }
}

impl From<CustomError> for Infallible {
    fn from(val: CustomError) -> Self {
        panic!("{}", val)
    }
}

impl From<CustomErrorInner> for CustomError {
    fn from(inner: CustomErrorInner) -> Self {
        let error_severity = match &inner {
            CustomErrorInner::NotFound(sev)
            | CustomErrorInner::Forbidden(sev)
            | CustomErrorInner::Unknown(sev) => sev.clone(),
            CustomErrorInner::DatabaseError(_, sev)
            | CustomErrorInner::Conflict(_, sev)
            | CustomErrorInner::BadRequest(_, sev)
            | CustomErrorInner::UnAuthorized(_, sev) => sev.clone(),
        };
        let backtrace = Box::new(Backtrace::force_capture());
        Self {
            inner,
            backtrace,
            error_severity: error_severity.clone(),
        }
    }
}

pub trait ResultExt: Sized {
    type T;
    fn unwrap_or_backtrace(self) -> Self::T {
        self.expect_or_backtrace("ResultExt::unwrap_or_backtrace found Err")
    }
    fn expect_or_backtrace(self, msg: &str) -> Self::T;
}

impl<T> ResultExt for Result<T, CustomError> {
    type T = T;
    fn expect_or_backtrace(self, msg: &str) -> T {
        match self {
            Ok(ok) => ok,
            Err(bterr) => {
                eprintln!("Error occurred{msg}");
                eprintln!();
                eprintln!("{bterr:}");
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

impl From<Infallible> for CustomError {
    fn from(_: Infallible) -> Self {
        CustomErrorInner::Unknown(ErrorSeverity::Error).into()
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
                eprintln!("{msg}");
                eprintln!();
                eprintln!("{bterr:}");
                panic!("{}", msg);
            }
        }
    }
}

impl Debug for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Error for CustomError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.inner)
    }
}

impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        let status = match &self.inner {
            CustomErrorInner::NotFound(_) => StatusCode::NOT_FOUND,
            CustomErrorInner::Forbidden(_) => StatusCode::FORBIDDEN,
            CustomErrorInner::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomErrorInner::DatabaseError(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomErrorInner::Conflict(_, _) => StatusCode::CONFLICT,
            CustomErrorInner::BadRequest(_, _) => StatusCode::BAD_REQUEST,
            CustomErrorInner::UnAuthorized(_, _) => StatusCode::UNAUTHORIZED,
        };

        (status, self.error_response().message).into_response()
    }
}

impl CustomError {
    fn error_response(&self) -> ErrorResponse {
        let status_code = match &self.inner {
            CustomErrorInner::NotFound(_) => 404,
            CustomErrorInner::Forbidden(_) => 403,
            CustomErrorInner::UnAuthorized(_, _) => 401,
            CustomErrorInner::BadRequest(_, _) => 404,
            CustomErrorInner::Conflict(_, _) => 409,
            CustomErrorInner::Unknown(_) => 500,
            CustomErrorInner::DatabaseError(_, _) => 500,
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
        match self.error_severity {
            ErrorSeverity::Critical | ErrorSeverity::Error => {
                error!("Error {}: {} with error", self.inner, self.backtrace);
            }
            ErrorSeverity::Warning => {
                warn!("Warning {}: {} with error", self.inner, self.backtrace);
            }
            ErrorSeverity::Info => {
                info!("Info {}: {} with error", self.inner, self.backtrace);
            }
            ErrorSeverity::Debug => {
                debug!("Debug {}: {} with error", self.inner, self.backtrace);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Debug,
    Critical,
}

#[derive(Error, Debug)]
pub enum CustomErrorInner {
    #[error("Requested file was not found")]
    NotFound(ErrorSeverity),
    #[error("You are forbidden to access requested file.")]
    Forbidden(ErrorSeverity),
    #[error("You are not authorized to access this resource. {0}")]
    UnAuthorized(String, ErrorSeverity),
    #[error("Bad Request: {0}")]
    BadRequest(String, ErrorSeverity),
    #[error("The following error occurred: {0}")]
    Conflict(String, ErrorSeverity),
    #[error("Unknown Internal Error")]
    Unknown(ErrorSeverity),
    #[error("Unknown Internal Error")]
    DatabaseError(diesel::result::Error, ErrorSeverity),
}

impl CustomErrorInner {
    pub fn name(&self) -> String {
        match self {
            Self::NotFound(_) => "NotFound".to_string(),
            Self::Forbidden(_) => "Forbidden".to_string(),
            Self::Unknown(_) => "Unknown".to_string(),
            Self::DatabaseError(_, _) => "DatabaseError".to_string(),
            Self::Conflict(e, _) => e.to_string(),
            Self::BadRequest(e, _) => e.to_string(),
            Self::UnAuthorized(_, _) => "UnAuthorized".to_string(),
        }
    }
}

pub fn map_io_error(
    e: std::io::Error,
    path: Option<String>,
    error_severity: ErrorSeverity,
) -> CustomError {
    error!(
        "IO error: {} for path {}",
        e,
        path.unwrap_or("".to_string())
    );
    match e.kind() {
        std::io::ErrorKind::NotFound => {
            CustomError::from(CustomErrorInner::NotFound(error_severity))
        }
        std::io::ErrorKind::PermissionDenied => {
            CustomError::from(CustomErrorInner::Forbidden(error_severity))
        }
        _ => CustomError::from(CustomErrorInner::Unknown(error_severity)),
    }
}

pub fn map_s3_error(error: S3Error, error_severity: ErrorSeverity) -> CustomError {
    log::info!("S3 error: {error}");
    CustomErrorInner::Unknown(error_severity).into()
}

pub fn map_io_extra_error(
    e: fs_extra::error::Error,
    path: Option<String>,
    severity: ErrorSeverity,
) -> CustomError {
    error!(
        "IO extra error: {} for path {}",
        e,
        path.unwrap_or("".to_string())
    );
    CustomError::from(CustomErrorInner::Unknown(severity))
}

pub fn map_db_error(e: diesel::result::Error, severity: ErrorSeverity) -> CustomError {
    error!("Database error: {e}");
    match e {
        diesel::result::Error::InvalidCString(_) => {
            CustomError::from(CustomErrorInner::NotFound(severity))
        }
        diesel::result::Error::DatabaseError(_, _) => {
            CustomError::from(CustomErrorInner::DatabaseError(e, severity))
        }
        _ => CustomError::from(CustomErrorInner::Unknown(severity)),
    }
}

pub fn map_r2d2_error(e: r2d2::Error, error_severity: ErrorSeverity) -> CustomError {
    error!("R2D2 error: {e}");
    CustomError::from(CustomErrorInner::Unknown(error_severity))
}

pub fn map_reqwest_error(e: reqwest::Error) -> CustomError {
    error!("Error during reqwest: {e}");

    CustomErrorInner::BadRequest(
        "Error requesting resource from server".to_string(),
        ErrorSeverity::Warning,
    )
    .into()
}

#[derive(Serialize)]
struct ErrorResponse {
    pub code: u16,
    pub error: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use crate::utils::error::{CustomErrorInner, ErrorSeverity, map_db_error, map_io_error};

    use diesel::result::Error;
    use serial_test::serial;
    use std::io::ErrorKind;

    #[test]
    #[serial]
    fn test_map_io_error() {
        let io_error = std::io::Error::new(ErrorKind::NotFound, "File not found");
        let custom_error = map_io_error(io_error, None, ErrorSeverity::Error);
        assert!(
            custom_error
                .to_string()
                .contains("Requested file was not found")
        );
    }

    #[test]
    #[serial]
    fn test_map_db_error() {
        let db_error = Error::NotFound;
        let custom_error = map_db_error(db_error, ErrorSeverity::Warning);
        assert!(custom_error.to_string().starts_with("Initial error"));
    }

    #[test]
    #[serial]
    fn test_custom_error() {
        let custom_error = CustomErrorInner::NotFound;
        assert_eq!(
            custom_error(ErrorSeverity::Error).to_string(),
            "Requested file was not found"
        );
    }

    #[test]
    #[serial]
    fn test_custom_conflict_message() {
        let custom_error =
            CustomErrorInner::Conflict("An error occured".to_string(), ErrorSeverity::Warning);
        assert_eq!(
            custom_error.to_string(),
            "The following error occurred: An error occured"
        );
    }
}
