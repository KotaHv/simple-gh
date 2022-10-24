use byte_unit::Byte;
use rocket::{
    http::{
        uri::{error::PathError as RPathError, fmt::Path, Segments},
        ContentType, Status,
    },
    request::{self, FromRequest, FromSegments},
    tokio::fs::{self, write},
    Request, Route, State,
};

use crate::config::Config;
use crate::util;

pub fn routes() -> Vec<Route> {
    routes![get_gh]
}

#[get("/<github_path..>")]
async fn get_gh(
    github_path: PathGuard,
    client: &State<reqwest::Client>,
    config: &State<Config>,
    _token: Token,
) -> Result<(Status, (ContentType, Vec<u8>)), Status> {
    let github_path = github_path.0;
    let file_str = github_path.clone().replace("/", "_");
    let filepath = config.cache_path.join(&file_str);
    let typepath = util::typepath(&filepath);
    if filepath.exists() {
        debug!("{file_str} is exists");
        match fs::read(&filepath).await {
            Ok(content) => {
                let content_type = util::content_type_typepath(&typepath).await;
                return Ok((Status::Ok, (content_type, content)));
            }
            Err(e) => {
                error!("{file_str}: {e}");
            }
        }
    }

    let res = match client
        .get(format!("https://raw.githubusercontent.com/{}", github_path))
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            error!("{github_path}: {e}");
            return Err(Status::InternalServerError);
        }
    };

    let is_success = res.status().is_success();
    let status_code = Status::new(res.status().as_u16());
    let content_type = util::content_type_reqwest(&res);
    let content_length_option = res.content_length();
    let content: bytes::Bytes = res.bytes().await.unwrap();
    let data: Vec<u8> = content.to_vec();
    if is_success {
        if let Some(content_length) = content_length_option {
            if content_length <= config.file_max {
                write(&filepath, &data).await.ok();
                write(&typepath, &content_type.to_string()).await.ok();
            } else {
                warn!(
                    "{file_str} content-length:{} > {}",
                    Byte::from_bytes(content_length)
                        .get_appropriate_unit(true)
                        .to_string(),
                    Byte::from_bytes(config.file_max)
                        .get_appropriate_unit(true)
                        .to_string()
                );
            }
        } else {
            warn!("{file_str} content-length is None");
        }
    }
    Ok((status_code, (content_type, data)))
}

struct PathGuard(String);

#[derive(Debug, PartialEq, Eq, Clone)]
enum PathError {
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

struct Token();
#[derive(Debug)]
enum TokenError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = TokenError;
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let config = request.guard::<&State<Config>>().await.unwrap();
        let token = &config.token;
        if token == "" {
            debug!("Token not set");
            return request::Outcome::Success(Token());
        }
        if let Some(token_result) = request.query_value::<String>("token") {
            match token_result {
                Ok(query_token) => {
                    if token == &query_token {
                        return request::Outcome::Success(Token());
                    }
                }
                Err(e) => {
                    error!(target: "token", "{e}");
                }
            }
            return request::Outcome::Failure((Status::NotFound, TokenError::Invalid));
        }
        request::Outcome::Failure((Status::NotFound, TokenError::Missing))
    }
}
