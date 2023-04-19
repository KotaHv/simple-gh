use actix_web::{
    guard::{Guard, GuardContext},
    http::StatusCode,
    web, HttpResponse, Responder, Scope,
};
use reqwest::Client;
use serde::Deserialize;
use tokio::fs;

use crate::config::CONFIG;
use crate::error::CustomError;
use crate::util;

pub fn routes(path: &str) -> Scope {
    let mut get_gh = web::resource("/{gh_path:.*}")
        .guard(PathGuard)
        .route(web::get().to(get_gh));
    if CONFIG.token.is_some() {
        get_gh = get_gh.guard(TokenGuard)
    }
    web::scope(path).service(get_gh)
}

struct Request {
    url: String,
    client: web::Data<Client>,
}

impl Request {
    fn new(client: web::Data<Client>, gh_path: &str) -> Self {
        Request {
            url: format!("https://raw.githubusercontent.com/{gh_path}"),
            client,
        }
    }
    async fn get(&self) -> Result<reqwest::Response, CustomError> {
        match self.client.get(&self.url).send().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{}: {:?}", self.url, e);
                return Err(CustomError::reason(e.to_string()));
            }
        }
    }

    async fn head(&self) -> Result<reqwest::Response, CustomError> {
        match self.client.head(&self.url).send().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{}: {:?}", self.url, e);
                return Err(CustomError::reason(e.to_string()));
            }
        }
    }
}

async fn get_gh(
    gh_path: web::Path<String>,
    client: web::Data<Client>,
) -> Result<impl Responder, CustomError> {
    let filepath = gh_path.replace("/", "_");
    let filepath = CONFIG.cache.path.join(filepath);
    let typepath = util::typepath(&filepath);
    match fs::read(&filepath).await {
        Ok(content) => {
            debug!("{filepath:?} is exists");
            let content_type = util::content_type_typepath(&typepath).await;
            return Ok(HttpResponse::build(StatusCode::OK)
                .append_header(("content-type", content_type))
                .body(content));
        }
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                error!("{filepath:?}: {e}")
            }
        }
    }
    let req = Request::new(client, &gh_path);
    let res = match req.head().await {
        Ok(res) => res,
        Err(re) => return Err(re),
    };
    match res.content_length() {
        Some(content_length) => {
            if content_length > CONFIG.file_max {
                let reason = format!(
                    "file size: {} > {}",
                    byte_unit::Byte::from_bytes(content_length)
                        .get_appropriate_unit(true)
                        .to_string(),
                    byte_unit::Byte::from_bytes(CONFIG.file_max)
                        .get_appropriate_unit(true)
                        .to_string()
                );
                return Err(CustomError::new(reason, StatusCode::PAYLOAD_TOO_LARGE));
            }
        }
        None => return Err(CustomError::reason(format!("{:#?}", res.headers()))),
    }
    let status_code = res.status();
    let is_success = status_code.is_success();
    let content_type = match res.headers().get(reqwest::header::CONTENT_TYPE) {
        Some(ct) => ct.to_str().unwrap(),
        None => "application/octet-stream",
    };
    let res = match req.get().await {
        Ok(res) => res,
        Err(re) => return Err(re),
    };
    let content = res.bytes().await.unwrap();
    if is_success {
        fs::write(&filepath, &content).await.ok();
        fs::write(&typepath, &content_type).await.ok();
    }
    Ok(HttpResponse::build(status_code)
        .append_header(("content-type", content_type))
        .body(content))
}

struct PathGuard;

impl Guard for PathGuard {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        let len = ctx.head().uri.path().split('/').count();
        if len >= 5 {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Deserialize)]
struct GHParams {
    token: String,
}

struct TokenGuard;

impl Guard for TokenGuard {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        if let Some(params) = ctx.head().uri.query() {
            if let Ok(params) = web::Query::<GHParams>::from_query(params) {
                if Some(&params.token) == CONFIG.token.as_ref() {
                    return true;
                }
            }
        }
        false
    }
}
