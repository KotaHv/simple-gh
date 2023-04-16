use std::{convert::Infallible, str::FromStr};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs;
use warp::{
    http::{Response, StatusCode},
    path::Tail,
    reject, Filter, Rejection, Reply,
};

use crate::config::CONFIG;
use crate::util;

#[derive(Deserialize, Serialize)]
struct GhQuery {
    token: String,
}

pub fn routes(
    client: Client,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    gh_route(client.clone()).recover(handle_rejection)
}

fn gh_route(
    client: Client,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let route = match CONFIG.token {
        Some(_) => token_guard().boxed(),
        None => warp::any().boxed(),
    };
    route
        .and(warp::get())
        .and(path_guard())
        .and(with_client(client))
        .and_then(get_gh)
}

fn path_guard() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::path::tail().and_then(|tail: Tail| async move {
        let gh_path = tail.as_str();
        if gh_path.replace("/", "").len() == 0 {
            return Err(reject::not_found());
        }
        Ok(gh_path.to_string())
    })
}

fn token_guard() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::any()
        .and(warp::query::<GhQuery>())
        .and_then(|q: GhQuery| async move {
            if Some(q.token) != CONFIG.token {
                return Err(reject::not_found());
            }
            Ok(())
        })
        .untuple_one()
}

fn with_client(client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

struct Request {
    url: String,
    client: Client,
}

impl Request {
    fn new(client: Client, gh_path: &str) -> Self {
        Request {
            url: format!("https://raw.githubusercontent.com/{gh_path}"),
            client,
        }
    }
    async fn get(&self) -> Result<reqwest::Response, RequestError> {
        match self.client.get(&self.url).send().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{}: {:?}", self.url, e);
                return Err(RequestError {
                    reason: e.to_string(),
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                });
            }
        }
    }

    async fn head(&self) -> Result<reqwest::Response, RequestError> {
        match self.client.head(&self.url).send().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{}: {:?}", self.url, e);
                return Err(RequestError {
                    reason: e.to_string(),
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                });
            }
        }
    }
}

async fn get_gh(gh_path: String, client: Client) -> Result<Box<dyn Reply>, Rejection> {
    let filepath = gh_path.replace("/", "_");
    let filepath = CONFIG.cache.path.join(filepath);
    let typepath = util::typepath(&filepath);
    match fs::read(&filepath).await {
        Ok(content) => {
            debug!("{filepath:?} is exists");
            let content_type = util::content_type_typepath(&typepath).await;
            let res = Response::builder()
                .header("content-type", content_type)
                .body(content);
            return Ok(Box::new(res));
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
        Err(re) => return Err(reject::custom(re)),
    };
    match res.content_length() {
        Some(content_length) => {
            if content_length > CONFIG.file_max {
                return Err(reject::custom(RequestError {
                    reason: format!("{} > {}", content_length, CONFIG.file_max),
                    status: StatusCode::PAYLOAD_TOO_LARGE,
                }));
            }
        }
        None => {
            return Err(reject::custom(RequestError {
                reason: format!("{:#?}", res.headers()),
                status: StatusCode::INTERNAL_SERVER_ERROR,
            }))
        }
    }
    let status_code = StatusCode::from_str(res.status().as_str()).unwrap();
    let content_type = match res.headers().get(reqwest::header::CONTENT_TYPE) {
        Some(ct) => ct.to_str().unwrap(),
        None => "application/octet-stream",
    };
    let is_success = status_code.is_success();
    let res = match req.get().await {
        Ok(res) => res,
        Err(re) => return Err(reject::custom(re)),
    };
    let content = res.bytes().await.unwrap();
    if is_success {
        fs::write(&filepath, &content).await.ok();
        fs::write(&typepath, &content_type).await.ok();
    }
    let response = Response::builder()
        .header("content-type", content_type)
        .status(status_code)
        .body(content);
    Ok(Box::new(response))
}

#[derive(Debug)]
struct RequestError {
    reason: String,
    status: StatusCode,
}

impl reject::Reject for RequestError {}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.is_not_found() || err.find::<reject::InvalidQuery>().is_some() {
        return Ok(warp::reply::with_status(
            StatusCode::NOT_FOUND
                .canonical_reason()
                .unwrap_or_default()
                .to_string(),
            StatusCode::NOT_FOUND,
        ));
    } else if let Some(RequestError { reason, status }) = err.find() {
        return Ok(warp::reply::with_status(
            reason.to_string(),
            status.to_owned(),
        ));
    }
    Err(err)
}
