use rocket::{
    http::{
        uri::{error::PathError as RPathError, fmt::Path, Segments},
        Status,
    },
    request::{FromRequest, FromSegments, Outcome},
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
        if segments.len() < 3 {
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
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = CONFIG.token.as_ref();
        if let Some(token) = token {
            if let Some(Ok(token_query)) = request.query_value::<String>("token") {
                if token != &token_query {
                    return Outcome::Failure((Status::NotFound, TokenError::Invalid));
                }
            } else {
                return Outcome::Failure((Status::NotFound, TokenError::Missing));
            }
        }
        Outcome::Success(TokenGuard)
    }
}
