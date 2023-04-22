use std::sync::Arc;

use axum::{
    body::Full,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use reqwest::Client;
use tokio::fs;

use crate::config::CONFIG;
use crate::util;

struct Request {
    url: String,
    client: Arc<Client>,
}

impl Request {
    fn new(client: Arc<Client>, gh_path: &str) -> Self {
        Request {
            url: format!("https://raw.githubusercontent.com/{gh_path}"),
            client,
        }
    }
    async fn get(&self) -> Result<reqwest::Response, reqwest::Error> {
        match self.client.get(&self.url).send().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{}: {:?}", self.url, e);
                Err(e)
            }
        }
    }

    async fn head(&self) -> Result<reqwest::Response, reqwest::Error> {
        match self.client.head(&self.url).send().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{}: {:?}", self.url, e);
                return Err(e);
            }
        }
    }
}

pub fn routes() -> Router<Arc<Client>> {
    Router::new().route("/*gh_path", get(get_gh))
}

async fn get_gh(Path(gh_path): Path<String>, State(client): State<Arc<Client>>) -> Response {
    let filepath = gh_path.replace("/", "_");
    let filepath = CONFIG.cache.path.join(filepath);
    let typepath = util::typepath(&filepath);
    match fs::read(&filepath).await {
        Ok(content) => {
            debug!("{filepath:?} is exists");
            let content_type = util::content_type_typepath(&typepath).await;
            return Response::builder()
                .status(StatusCode::OK)
                .header("content-type", content_type)
                .body(Full::from(content))
                .unwrap()
                .into_response();
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
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
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
                return (StatusCode::PAYLOAD_TOO_LARGE, reason).into_response();
            }
        }
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{:#?}", res.headers()),
            )
                .into_response()
        }
    }
    let status_code = res.status();
    let is_success = status_code.is_success();
    let content_type = match res.headers().get(reqwest::header::CONTENT_TYPE) {
        Some(ct) => ct.to_str().unwrap(),
        None => "application/octet-stream",
    };
    let res = match req.get().await {
        Ok(res) => res,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let content = res.bytes().await.unwrap();
    if is_success {
        fs::write(&filepath, &content).await.ok();
        fs::write(&typepath, &content_type).await.ok();
    }
    // (status_code, [("content-type", content_type)], content).into_response()
    Response::builder()
        .status(status_code)
        .header("content-type", content_type)
        .body(Full::from(content))
        .unwrap()
        .into_response()
}
