use std::{convert::Infallible, sync::Arc};

use byte_unit::Byte;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs;
use warp::{
    http::{Response, StatusCode},
    path::Tail,
    reject, Filter, Rejection, Reply,
};

use crate::config::Config;
use crate::util;

#[derive(Deserialize, Serialize)]
struct GhQuery {
    token: String,
}

pub fn routes(
    client: Client,
    config: Arc<Config>,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    gh_route(client.clone(), config.clone()).recover(handle_rejection)
}

fn gh_route(
    client: Client,
    config: Arc<Config>,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let route;
    if config.token.is_empty() {
        route = with_config(config.clone()).boxed();
    } else {
        route = token_guard(config.clone()).boxed();
    }
    route
        .and(warp::get())
        .and(path_guard())
        .and(with_client(client))
        .and_then(get_gh)
}

fn token_guard(
    config: Arc<Config>,
) -> impl Filter<Extract = (Arc<Config>,), Error = Rejection> + Clone {
    warp::any()
        .and(warp::query::<GhQuery>())
        .and_then(move |q: GhQuery| {
            let config = config.clone();
            async move {
                if !config.token.is_empty() && config.token != q.token {
                    return Err(reject::not_found());
                }
                Ok(config)
            }
        })
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

fn with_client(client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_config(
    config: Arc<Config>,
) -> impl Filter<Extract = (Arc<Config>,), Error = Infallible> + Clone {
    warp::any().map(move || config.clone())
}

async fn get_gh(
    config: Arc<Config>,
    gh_path: String,
    client: Client,
) -> Result<impl Reply, Rejection> {
    let file_str = gh_path.replace("/", "_");
    let filepath = config.cache_path.join(&file_str);
    let typepath = util::typepath(&filepath);
    if filepath.exists() {
        debug!("{file_str} is exists");
        match fs::read(&filepath).await {
            Ok(content) => {
                let content_type = util::content_type_typepath(&typepath).await;
                return Ok(Response::builder()
                    .header("content-type", content_type)
                    .body(content)
                    .into_response());
            }
            Err(e) => {
                error!("{file_str}: {e}");
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
            return Err(reject::custom(InternalServerError));
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
    // let data = content.;
    let data: Vec<u8> = content.to_vec();
    if is_success {
        if let Some(content_length) = content_length_option {
            if content_length <= config.file_max {
                fs::write(&filepath, &data).await.ok();
                fs::write(&typepath, &content_type).await.ok();
            } else {
                warn!(
                    "{gh_path} content-length:{} > {}",
                    Byte::from_bytes(content_length)
                        .get_appropriate_unit(true)
                        .to_string(),
                    Byte::from_bytes(config.file_max)
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
        .body(data);
    Ok(response.into_response())
}

#[derive(Debug)]
struct InternalServerError;

impl reject::Reject for InternalServerError {}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.is_not_found() || err.find::<reject::InvalidQuery>().is_some() {
        return Ok(warp::reply::with_status(
            StatusCode::NOT_FOUND.canonical_reason().unwrap_or_default(),
            StatusCode::NOT_FOUND,
        ));
    } else if let Some(InternalServerError) = err.find() {
        return Ok(warp::reply::with_status(
            StatusCode::INTERNAL_SERVER_ERROR
                .canonical_reason()
                .unwrap_or_default(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }
    Err(err)
}
