use std::fmt::Display;

use actix_web::{error, http::StatusCode};

#[derive(Debug)]
pub struct CustomError {
    reason: String,
    status: StatusCode,
}

impl CustomError {
    pub fn reason(reason: String) -> Self {
        CustomError {
            reason,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn new(reason: String, status: StatusCode) -> Self {
        CustomError { reason, status }
    }
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl error::ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        self.status
    }
}
