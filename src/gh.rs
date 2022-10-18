use std::path::PathBuf;

use byte_unit::Byte;
use rocket::{
    fs::NamedFile,
    http::{ContentType, Status},
    request::{self, FromRequest},
    tokio::fs::write,
    Request, Route, State,
};

use crate::config::Config;
use crate::util;

pub fn routes() -> Vec<Route> {
    routes![get_gh]
}

#[derive(Responder)]
enum GhResponse {
    File(Option<NamedFile>),
    Spider(Vec<u8>, ContentType),
}

#[get("/<github_path..>")]
async fn get_gh(
    github_path: PathBuf,
    client: &State<reqwest::Client>,
    config: &State<Config>,
    _token: Token,
) -> (Status, Option<GhResponse>) {
    let mut file_str = github_path.to_str().unwrap().to_string();
    if file_str.replace("/", "").len() == 0 {
        return (Status::NotFound, None);
    }
    file_str = file_str.replace("/", "_");
    let filepath = config.cache_path.join(&file_str);
    if filepath.exists() {
        debug!("{file_str} is exists");
        return (
            Status::Ok,
            Some(GhResponse::File(NamedFile::open(&filepath).await.ok())),
        );
    }
    let res = client
        .get(format!(
            "https://raw.githubusercontent.com/{}",
            github_path.to_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    let is_success = res.status().is_success();
    let status_code = Status::new(res.status().as_u16());
    let content_type_result = util::content_type(&res);
    let content_length_option = res.content_length();
    let content: bytes::Bytes = res.bytes().await.unwrap();
    let data: Vec<u8> = content.to_vec();
    if is_success {
        if let Some(content_length) = content_length_option {
            if content_length <= config.file_max {
                write(&filepath, &data).await.ok();
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
    match content_type_result {
        Ok(content_type) => (
            status_code,
            Some(GhResponse::Spider(
                data,
                ContentType::new(content_type.0, content_type.1),
            )),
        ),
        Err(content_type_string) => {
            warn!("path={github_path:?}, content-type={content_type_string}");
            (
                status_code,
                Some(GhResponse::Spider(
                    data,
                    ContentType::new(content_type_string, ""),
                )),
            )
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
        match request.query_value::<String>("token") {
            Some(token_result) => match token_result {
                Ok(query_token) => {
                    if token == &query_token {
                        request::Outcome::Success(Token())
                    } else {
                        request::Outcome::Failure((Status::NotFound, TokenError::Invalid))
                    }
                }
                Err(e) => {
                    error!(target: "token", "{e}");
                    request::Outcome::Failure((Status::NotFound, TokenError::Invalid))
                }
            },
            None => request::Outcome::Failure((Status::NotFound, TokenError::Missing)),
        }
    }
}
