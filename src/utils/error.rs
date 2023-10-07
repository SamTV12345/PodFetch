

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

pub fn map_io_error(e: std::io::Error, path: Option<String>) -> CustomError {
    error!("IO error: {} for path {}", e, path.unwrap_or("".to_string()));
    match e.kind() {
        std::io::ErrorKind::NotFound => CustomError::NotFound,
        std::io::ErrorKind::PermissionDenied => CustomError::Forbidden,
        _ => CustomError::Unknown,
    }
}

pub fn map_io_extra_error(e: fs_extra::error::Error, path: Option<String>) ->CustomError {
    error!("IO extra error: {} for path {}", e, path.unwrap_or("".to_string()));
    CustomError::Unknown
}

pub fn map_db_error(e: diesel::result::Error) -> CustomError {
    error!("Database error: {}", e);
    match e {
        diesel::result::Error::InvalidCString(_) => CustomError::NotFound,
        diesel::result::Error::DatabaseError(_, _) => CustomError::DatabaseError(e),
        _ => CustomError::Unknown,
    }
}

pub fn map_r2d2_error(e: r2d2::Error) -> CustomError {
    error!("R2D2 error: {}", e);
    CustomError::Unknown
}

pub fn map_reqwest_error(e: reqwest::Error) -> CustomError {
    error!("Error during reqwest: {}",e);

    CustomError::BadRequest("Error requesting resource from server".to_string())
}


#[derive(Serialize)]
struct ErrorResponse {
    pub code: u16,
    pub error: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use crate::utils::error::{CustomError, map_db_error, map_io_error};
    use actix_web::http::StatusCode;
    use actix_web::ResponseError;
    use diesel::result::Error;
    use std::io::ErrorKind;

    #[test]
    fn test_map_io_error() {
        let io_error = std::io::Error::new(ErrorKind::NotFound, "File not found");
        let custom_error = map_io_error(io_error, None);
        assert_eq!(custom_error.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(custom_error.to_string(), "Requested file was not found");
    }

    #[test]
    fn test_map_db_error() {
        let db_error = Error::NotFound;
        let custom_error = map_db_error(db_error);
        assert_eq!(custom_error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(custom_error.to_string(), "Unknown Internal Error");
    }

    #[test]
    fn test_custom_error() {
        let custom_error = CustomError::NotFound;
        assert_eq!(custom_error.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(custom_error.to_string(), "Requested file was not found");
    }

    #[test]
    fn test_custom_conflict_message() {
        let custom_error = CustomError::Conflict("An error occured".to_string());
        assert_eq!(custom_error.status_code(), StatusCode::CONFLICT);
        assert_eq!(custom_error.to_string(), "The following error occurred: An error occured");
    }
}