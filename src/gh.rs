use std::convert::Infallible;

use byte_unit::Byte;
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

async fn get_gh(gh_path: String, client: Client) -> Result<Box<dyn Reply>, Rejection> {
    let file_str = gh_path.replace("/", "_");
    let filepath = CONFIG.cache_path.join(&file_str);
    let typepath = util::typepath(&filepath);
    match fs::read(&filepath).await {
        Ok(content) => {
            debug!("{file_str} is exists");
            let content_type = util::content_type_typepath(&typepath).await;
            let res = Response::builder()
                .header("content-type", content_type)
                .body(content);
            return Ok(Box::new(res));
        }
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                error!("{file_str}: {e}")
            }
        }
    }
    let res = match client
        .get(format!("https://raw.githubusercontent.com/{gh_path}"))
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            error!("{gh_path}: {e:?}");
            return Err(reject::custom(RequestError(e.to_string())));
        }
    };
    let is_success = res.status().is_success();
    let status_code = StatusCode::from_u16(res.status().as_u16()).unwrap();
    let mut content_type = "application/octet-stream".to_string();
    if let Some(ct) = res.headers().get(reqwest::header::CONTENT_TYPE) {
        if let Ok(ct) = ct.to_str() {
            content_type = ct.to_string();
        }
    }
    let content_length_option = res.content_length();
    let content = res.bytes().await.unwrap();
    if is_success {
        if let Some(content_length) = content_length_option {
            if content_length <= CONFIG.file_max {
                fs::write(&filepath, &content).await.ok();
                fs::write(&typepath, &content_type).await.ok();
            } else {
                warn!(
                    "{gh_path} content-length:{} > {}",
                    Byte::from_bytes(content_length)
                        .get_appropriate_unit(true)
                        .to_string(),
                    Byte::from_bytes(CONFIG.file_max)
                        .get_appropriate_unit(true)
                        .to_string()
                );
            }
        } else {
            warn!("{gh_path} content-length is None");
        }
    }
    let response = Response::builder()
        .header("content-type", content_type)
        .status(status_code)
        .body(content);
    Ok(Box::new(response))
}

#[derive(Debug)]
struct RequestError(String);

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
    } else if let Some(RequestError(reason)) = err.find() {
        return Ok(warp::reply::with_status(
            reason.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }
    Err(err)
}
