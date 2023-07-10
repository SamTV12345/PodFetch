

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use log::error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Requested file was not found")]
    NotFound,
    #[error("You are forbidden to access requested file.")]
    Forbidden,
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("The following error occurred: {0}")]
    Conflict(String),
    #[error("Unknown Internal Error")]
    Unknown,
    #[error("Unknown Internal Error")]
    DatabaseError(diesel::result::Error),
}
impl CustomError {
    pub fn name(&self) -> String {
        match self {
            Self::NotFound => "NotFound".to_string(),
            Self::Forbidden => "Forbidden".to_string(),
            Self::Unknown => "Unknown".to_string(),
            Self::DatabaseError(_) => "DatabaseError".to_string(),
            Self::Conflict(e) => e.to_string(),
            Self::BadRequest(e)=>e.to_string()
        }
    }
}

impl ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound  => StatusCode::NOT_FOUND,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::BadRequest(_)=>StatusCode::BAD_REQUEST
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error: self.name(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}

pub fn map_io_error(e: std::io::Error) -> CustomError {
    match e.kind() {
        std::io::ErrorKind::NotFound => CustomError::NotFound,
        std::io::ErrorKind::PermissionDenied => CustomError::Forbidden,
        _ => CustomError::Unknown,
    }
}

pub fn map_io_extra_error(e: fs_extra::error::Error) ->CustomError {
    match e.kind {
        fs_extra::error::ErrorKind::NotFound => CustomError::NotFound,
        fs_extra::error::ErrorKind::PermissionDenied => CustomError::Forbidden,
        _ => CustomError::Unknown,
    }
}

pub fn map_db_error(e: diesel::result::Error) -> CustomError {
    error!("Database error: {}", e);
    match e {
        diesel::result::Error::InvalidCString(_) => CustomError::NotFound,
        diesel::result::Error::DatabaseError(_, _) => CustomError::DatabaseError(e),
        _ => CustomError::Unknown,
    }
}

pub fn map_reqwest_error(e: reqwest::Error) -> CustomError {
    error!("Error during reqwest: {}",e);

    return CustomError::BadRequest("Error requesting resource from server".to_string())
}


#[derive(Serialize)]
struct ErrorResponse {
    pub code: u16,
    pub error: String,
    pub message: String,
}
