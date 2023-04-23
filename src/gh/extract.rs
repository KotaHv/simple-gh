use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

pub struct GHPath(pub String);
#[async_trait]
impl<S> FromRequestParts<S> for GHPath {
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let mut path = parts.uri.path();
        path = path.trim_start_matches("/").trim_end_matches("/");
        match path.split("/").count() {
            0..=2 => Err(StatusCode::NOT_FOUND),
            _ => Ok(GHPath(path.to_string())),
        }
    }
}
