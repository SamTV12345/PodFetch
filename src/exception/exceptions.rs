use std::fmt::{Display, Formatter};
use actix_web::{error, HttpResponse};
use actix_web::http::StatusCode;

pub trait PodFetchErrorTrait {
    fn new(name: &'static str, status_code: StatusCode) -> Self;
    fn name(&self) -> &'static str;
    fn status_code(&self) -> StatusCode;
}

#[derive(Debug)]
pub struct PodFetchError {
    name: &'static str,
    status_code: StatusCode
}

impl Display for PodFetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "The following error occurred: {}", self.name)
    }
}

impl PodFetchErrorTrait for PodFetchError {
    fn new(name: &'static str, status_code: StatusCode) -> Self {
        PodFetchError {
            name,
            status_code
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn status_code(&self) -> StatusCode {
        self.status_code
    }
}

// Use default implementation for `error_response()` method
impl error::ResponseError for PodFetchError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code)
            .body(self.name)
    }
}

impl PodFetchError{
    pub fn podcast_already_exists() -> PodFetchError {
        PodFetchError::new("Podcast already exists", StatusCode::BAD_REQUEST)
    }
}