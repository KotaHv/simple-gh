use std::sync::Arc;
use std::task::{Context, Poll};

use async_trait::async_trait;
use axum::{
    body::Full,
    extract::{FromRequestParts, State},
    http::{self, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::future::BoxFuture;
use reqwest::Client;
use serde::Deserialize;
use tokio::fs;
use tower::{Layer, Service};

use crate::config::CONFIG;
use crate::error::CustomError;
use crate::util;

pub fn routes() -> Router<Arc<Client>> {
    let mut get_gh = get(get_gh);
    if CONFIG.token.is_some() {
        debug!("TokenLayer");
        get_gh = get_gh.route_layer(TokenLayer);
    }
    Router::new().route("/*gh_path", get(get_gh))
}

struct Request {
    url: String,
    client: Arc<Client>,
}
type RequestOutput = Result<reqwest::Response, CustomError>;
impl Request {
    fn new(client: Arc<Client>, gh_path: &str) -> Self {
        Request {
            url: format!("https://raw.githubusercontent.com/{gh_path}"),
            client,
        }
    }
    async fn get(&self) -> RequestOutput {
        Request::result(self.client.get(&self.url).send().await)
    }

    async fn head(&self) -> RequestOutput {
        Request::result(self.client.head(&self.url).send().await)
    }

    fn result(res: Result<reqwest::Response, reqwest::Error>) -> RequestOutput {
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{:#?}", e);
                Err(CustomError::reason(e.to_string()))
            }
        }
    }
}

struct GHPath(String);
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

#[derive(Clone)]
struct TokenLayer;

impl<S> Layer<S> for TokenLayer {
    type Service = TokenMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TokenMiddleware { inner }
    }
}

#[derive(Clone)]
struct TokenMiddleware<S> {
    inner: S,
}

#[derive(Debug, Deserialize)]
struct GHParams {
    token: String,
}

impl<S, T> Service<http::Request<T>> for TokenMiddleware<S>
where
    S: Service<http::Request<T>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: http::Request<T>) -> Self::Future {
        let mut is_ok = false;
        if let Some(params) = request.uri().query() {
            if let Ok(params) = serde_urlencoded::from_str::<GHParams>(params) {
                if Some(params.token) == CONFIG.token {
                    is_ok = true;
                }
            }
        }
        let future = self.inner.call(request);
        Box::pin(async move {
            match is_ok {
                true => {
                    let response: Response = future.await?;
                    Ok(response)
                }
                false => Ok(StatusCode::NOT_FOUND.into_response()),
            }
        })
    }
}

async fn get_gh(
    GHPath(gh_path): GHPath,
    State(client): State<Arc<Client>>,
) -> Result<Response, CustomError> {
    let filepath = gh_path.replace("/", "_");
    let filepath = CONFIG.cache.path.join(filepath);
    let typepath = util::typepath(&filepath);
    match fs::read(&filepath).await {
        Ok(content) => {
            debug!("{filepath:?} is exists");
            let content_type = util::content_type_typepath(&typepath).await;
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", content_type)
                .body(Full::from(content))
                .unwrap()
                .into_response());
        }
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                error!("{filepath:?}: {e}")
            }
        }
    }
    let req = Request::new(client, &gh_path);
    let res = req.head().await?;
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
    let res = req.get().await?;
    let content = res.bytes().await.unwrap();
    if is_success {
        if let Ok(_) = fs::write(&filepath, &content).await {
            fs::write(&typepath, &content_type).await.ok();
        }
    }
    // (status_code, [("content-type", content_type)], content).into_response()
    Ok(Response::builder()
        .status(status_code)
        .header("content-type", content_type)
        .body(Full::from(content))
        .unwrap()
        .into_response())
}
