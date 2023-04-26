use rocket::{
    http::{
        uri::{error::PathError as RPathError, fmt::Path, Segments},
        Status,
    },
    request::{self, FromRequest, FromSegments},
    Request,
};

use crate::CONFIG;

pub struct PathGuard(pub String);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PathError {
    BadLen,
    Other(RPathError),
}

impl<'r> FromSegments<'r> for PathGuard {
    type Error = PathError;
    fn from_segments(segments: Segments<'r, Path>) -> Result<Self, Self::Error> {
        if segments.len() == 0 {
            return Err(PathError::BadLen);
        }
        match segments.to_path_buf(false) {
            Ok(path) => Ok(PathGuard(path.to_string_lossy().to_string())),
            Err(e) => Err(PathError::Other(e)),
        }
    }
}

pub struct TokenGuard;
#[derive(Debug)]
pub enum TokenError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TokenGuard {
    type Error = TokenError;
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let token = CONFIG.token.as_ref();
        if token.is_none() {
            debug!("Token not set");
            return request::Outcome::Success(TokenGuard);
        }
        if let Some(token_result) = request.query_value::<String>("token") {
            match token_result {
                Ok(query_token) => {
                    if token == Some(&query_token) {
                        return request::Outcome::Success(TokenGuard);
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            }
            return request::Outcome::Failure((Status::NotFound, TokenError::Invalid));
        }
        request::Outcome::Failure((Status::NotFound, TokenError::Missing))
    }
}
