use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt::Display;

#[derive(Debug)]
pub struct CustomError {
    reason: String,
    status: StatusCode,
}

impl CustomError {
    pub fn reason(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        CustomError {
            reason,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn new(reason: impl Into<String>, status: StatusCode) -> Self {
        let reason = reason.into();
        CustomError { reason, status }
    }
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        (self.status, self.reason).into_response()
    }
}
