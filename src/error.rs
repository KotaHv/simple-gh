use axum::{
    body::boxed,
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

impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(self.status)
            .body(boxed(self.reason))
            .unwrap()
    }
}
